
use compiler;

fn main() {
    let lexer = compiler::lexer::Lexer::new(std::io::stdin());
    println!("REPL!");
}

fn main_loop() {
    loop {
        print!("ready> ");
    }
}

fn handle_definition() {}
