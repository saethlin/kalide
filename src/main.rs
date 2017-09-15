extern crate llvm_sys;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

fn main() {
    let tests = [
        "extern sin(a);",
        "def foo(x y)\nx+foo(y, 4.0);",
        "def foo(x y) x+y; y;",
    ];

    for test in &tests {
        println!("Parsing\n{}\n", test);
        let lexer = Lexer::new(test);
        let mut parser = Parser::new(lexer);
        parser.run();
        println!('\n');
    }
}
