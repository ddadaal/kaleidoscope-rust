use crate::parser::nodes::Expression;
use crate::parser::nodes::Program;
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::values::AnyValueEnum;
use inkwell::values::BasicValueEnum;
use inkwell::values::FloatValue;
use inkwell::{values::PointerValue, FloatPredicate};
use std::collections::HashMap;

struct CodegenContext<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    named_values: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    /// Generate code of an expression
    /// All expressions have return value of float
    fn codegen_expr(&self, expr: &Expression) -> Result<FloatValue<'ctx>, String> {
        match expr {
            Expression::NumberExpr(num) => Ok(self.context.f64_type().const_float(*num)),
            Expression::VariableExpr(ref var) => self
                .named_values
                .get(var)
                .map(|x| self.builder.build_load(*x, var).into_float_value())
                .ok_or(format!("Unknown variable name: {}", var)),
            Expression::BinaryExpr(op, left, right) => {
                let lhs = self.codegen_expr(left)?;
                let rhs = self.codegen_expr(right)?;
                match op {
                    '+' => Ok(self.builder.build_float_add(lhs, rhs, "tmpadd")),
                    '-' => Ok(self.builder.build_float_sub(lhs, rhs, "tmpsub")),
                    '*' => Ok(self.builder.build_float_mul(lhs, rhs, "tmpmul")),
                    '/' => Ok(self.builder.build_float_div(lhs, rhs, "tmpdiv")),
                    '<' => Ok({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::ULT,
                            lhs,
                            rhs,
                            "tmpcmp",
                        );

                        self.builder.build_unsigned_int_to_float(
                            cmp,
                            self.context.f64_type(),
                            "tmpbool",
                        )
                    }),
                    '>' => Ok({
                        let cmp = self.builder.build_float_compare(
                            FloatPredicate::ULT,
                            rhs,
                            lhs,
                            "tmpcmp",
                        );

                        self.builder.build_unsigned_int_to_float(
                            cmp,
                            self.context.f64_type(),
                            "tmpbool",
                        )
                    }),
                    _ => Err(format!("Unknown binary op {}", op)),
                }
            }
            Expression::CallExpr(name, args) => {
                // Get function
                let func = self
                    .module
                    .get_function(name)
                    .ok_or(format!("Unknown function: {}", name))?;

                // validate args len
                if args.len() != func.count_params() as usize {
                    return Err(format!(
                        "Unmatched arg number. Function expects {} but the input has {}.",
                        args.len(),
                        func.count_params()
                    ));
                }

                // Parse args
                let mut parsed_args: Vec<BasicValueEnum> = Vec::with_capacity(args.len());

                for arg in args {
                    parsed_args.push(self.codegen_expr(arg)?.into());
                }

                self.builder
                    .build_call(func, parsed_args.as_slice(), "tmpcall")
                    .try_as_basic_value()
                    .left()
                    .map(|x| x.into_float_value())
                    .ok_or("Invalid call.".into())
            }
        }
    }
}
