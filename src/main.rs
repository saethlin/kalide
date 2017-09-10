#![allow(unused_variables)]
use std::fs::File;
use std::io::Read;

mod lexer;
use lexer::Lexer;

fn main() {
    let mut contents = String::new();
    File::open("test")
        .unwrap()
        .read_to_string(&mut contents)
        .unwrap();
    let mut lex = Lexer::new(&contents);
    let tok = lex.next_token();
}
