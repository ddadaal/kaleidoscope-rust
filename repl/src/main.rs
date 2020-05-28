use compiler;
use compiler::{codegen::codegen_context::CodegenContext, parser::parser::ParseError};
use std::{
    error::Error,
    io::{Read, Stdin, stdout, Write},
};

struct StdinIterator(Stdin);

impl Iterator for StdinIterator {
    type Item = char;
    fn next(&mut self) -> Option<Self::Item> {
        let mut character = [0];
        match self.0.read(&mut character) {
            Ok(x) if x > 0 => Some(character[0] as char),
            _ => None,
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let stdin_wrapper = StdinIterator(std::io::stdin());
    let lexer = compiler::lexer::Lexer::new(stdin_wrapper);
    let tokens = lexer.take_while(|x| x.is_ok()).map(|x| x.unwrap());
    let mut parser = compiler::parser::parser::Parser::new(tokens);

    let context = compiler::codegen::codegen_context::create_inkwell_context();
    let mut cc = CodegenContext::new(&context, "test");

    loop {
        print!("ready> ");
        stdout().flush()?;
        match parser.parse() {
            Ok(node) => match node {
                compiler::parser::nodes::ASTNode::ExternNode(proto) => {
                    match cc.compile_proto(&proto) {
                        Ok(fun_value) => println!("Read extern: {}", fun_value.print_to_string()),
                        Err(err) => println!("Err parsing extern: {}", err),
                    }
                }
                compiler::parser::nodes::ASTNode::FunctionNode(func) => {
                    match cc.compile_func(&func) {
                        Ok(fun_value) => println!("Read function: {}", fun_value.print_to_string()),
                        Err(err) => println!("Err parsing function: {}", err),
                    }
                }
                compiler::parser::nodes::ASTNode::EOF => break,
                compiler::parser::nodes::ASTNode::Delimiter => continue,
            },
            Err(err) => println!("Err parsing node: {:?}", err),
        }
    }

    Ok(())
}
