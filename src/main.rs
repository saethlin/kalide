extern crate llvm_sys;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

fn main() {
    let mut big_test = String::new();
    for _ in 0..100000 {
        big_test.extend("def foo(x, y)\nx+foo(y, 4.0);\n".chars());
    }

    Parser::new(Lexer::new(&big_test)).run().unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_extern() {
        Parser::new(Lexer::new("extern sin(a);")).run().unwrap()
    }

    #[test]
    fn parse_prototype() {
        Parser::new(Lexer::new("def foo(x, y);")).run().unwrap()
    }

    #[test]
    fn parse_function() {
        Parser::new(Lexer::new("def foo(x) x;")).run().unwrap()
    }

    #[test]
    fn parse_number() {
        Parser::new(Lexer::new("4.0;")).run().unwrap()
    }

    #[test]
    fn parse_binop() {
        Parser::new(Lexer::new("4.0 + 2.0;")).run().unwrap()
    }

}