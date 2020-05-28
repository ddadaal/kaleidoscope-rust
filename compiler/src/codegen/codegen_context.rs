use crate::parser::nodes::Expression;
use crate::parser::nodes::{Function, Prototype};
use inkwell::builder::Builder;
use inkwell::context::Context;
use inkwell::module::Module;
use inkwell::types::BasicTypeEnum;
use inkwell::values::AnyValueEnum;
use inkwell::values::BasicValueEnum;
use inkwell::values::FloatValue;
use inkwell::{
    values::{BasicValue, FunctionValue, PointerValue},
    FloatPredicate,
};
use std::collections::HashMap;

pub struct CodegenContext<'ctx> {
    context: &'ctx Context,
    module: Module<'ctx>,
    builder: Builder<'ctx>,
    named_values: HashMap<String, PointerValue<'ctx>>,
}

impl<'ctx> CodegenContext<'ctx> {
    pub fn new(context: &'ctx Context, module_name: &str) -> Self {
        CodegenContext {
            context,
            module: context.create_module(module_name),
            builder: context.create_builder(),
            named_values: HashMap::new(),
        }
    }

    /// Generate code of an expression
    /// All expressions have return value of float
    pub fn compile_expr(&self, expr: &Expression) -> Result<FloatValue<'ctx>, String> {
        match expr {
            Expression::NumberExpr(num) => Ok(self.context.f64_type().const_float(*num)),
            Expression::VariableExpr(ref var) => self
                .named_values
                .get(var)
                .map(|x| self.builder.build_load(*x, var).into_float_value())
                .ok_or(format!("Unknown variable name: {}", var)),
            Expression::BinaryExpr(op, left, right) => {
                let lhs = self.compile_expr(left)?;
                let rhs = self.compile_expr(right)?;
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
                    parsed_args.push(self.compile_expr(arg)?.into());
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

    /// Generate code of proto, convert a function prototype to a FunctionValue
    pub fn compile_proto(&self, proto: &Prototype) -> Result<FunctionValue<'ctx>, String> {
        let ret_type = self.context.f64_type();
        let arg_types: Vec<BasicTypeEnum> = vec![ret_type.into(); proto.args.len()];
        let arg_types_slice = arg_types.as_slice();

        let fn_type = self.context.f64_type().fn_type(arg_types_slice, false);
        let fn_val = self.module.add_function(&proto.name, fn_type, None);

        // set argument names
        for (i, arg) in fn_val.get_param_iter().enumerate() {
            arg.into_float_value().set_name(&proto.args[i]);
        }

        Ok(fn_val)
    }

    /// Creates a new stack allocation instruction
    pub fn create_entry_block_alloca(
        &self,
        fun_val: &FunctionValue,
        name: &str,
    ) -> PointerValue<'ctx> {
        let builder = self.context.create_builder();

        let entry = fun_val.get_first_basic_block().unwrap();

        match entry.get_first_instruction() {
            Some(first_instr) => builder.position_before(&first_instr),
            None => builder.position_at_end(entry),
        }

        builder.build_alloca(self.context.f64_type(), name)
    }

    pub fn compile_func(&mut self, func: &Function) -> Result<FunctionValue, String> {
        // if the FunctionValue does not exist, compile it.
        let fun_val = match self.module.get_function(&func.prototype.name) {
            Some(func) => func,
            None => self.compile_proto(&func.prototype)?,
        };

        let basic_block = self.context.append_basic_block(fun_val, "entry");
        self.builder.position_at_end(basic_block);

        // record the functioin arguments in the named_values
        self.named_values.clear();
        for (i, arg) in fun_val.get_param_iter().enumerate() {
            let arg_name = &func.prototype.args[i];
            let alloca = self.create_entry_block_alloca(&fun_val, arg_name);

            self.builder.build_store(alloca, arg);

            self.named_values.insert(arg_name.into(), alloca);
        }

        let body = self.compile_expr(&func.body)?;
        self.builder.build_return(Some(&body));

        if fun_val.verify(true) {
            Ok(fun_val)
        } else {
            unsafe {
                fun_val.delete();
            }
            Err(format!(
                "Generated function {} verification failed.",
                func.prototype.name
            ))
        }
    }
}

pub fn create_inkwell_context() -> Context {
    return Context::create();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_proto() {
        let context = Context::create();
        let cc = CodegenContext::new(&context, "test");

        let test_name = "test_func";

        let proto = Prototype {
            name: test_name.into(),
            args: vec!["arg1".into(), "arg2".into()],
        };

        let compiled_proto = cc.compile_proto(&proto).unwrap();

        println!("{:?}", compiled_proto);
        assert_eq!(
            test_name.to_string(),
            compiled_proto.get_name().to_str().unwrap()
        );
    }
}
