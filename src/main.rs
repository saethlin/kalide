use std::fs::File;
use std::io::Read;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

fn main() {
    let mut contents = String::new();
    File::open("test")
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let lexer = Lexer::new(&contents);
    let mut parser = Parser::new(lexer);
    parser.run();
}
