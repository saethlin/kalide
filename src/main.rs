#[macro_use]
extern crate bencher;
extern crate smallvec;

use bencher::Bencher;

mod lexer;
use lexer::Lexer;

mod parser;
use parser::Parser;

fn parse_extern(bench: &mut Bencher) {
    bench.iter(|| Parser::new(Lexer::new("extern sin(a);")).run().unwrap())
}

fn parse_prototype(bench: &mut Bencher) {
    bench.iter(|| Parser::new(Lexer::new("def foo(x, y);")).run().unwrap())
}

fn parse_function(bench: &mut Bencher) {
    bench.iter(|| {
        Parser::new(Lexer::new("def foo(x, y) x+y;")).run().unwrap()
    })
}

fn parse_number(bench: &mut Bencher) {
    bench.iter(|| Parser::new(Lexer::new("4.0;")).run().unwrap())
}

fn parse_binop(bench: &mut Bencher) {
    bench.iter(|| Parser::new(Lexer::new("4.0 + 2.0;")).run().unwrap())
}

fn lex_function(bench: &mut Bencher) {
    bench.iter(|| {
        let mut lex = Lexer::new("def foo(x, y) x+y;");
        while lex.next_token() != lexer::Token::EOF {}
    })
}

fn parse_loop(bench: &mut Bencher) {
    bench.iter(|| {
        Parser::new(Lexer::new(
            "
extern putchard(char);
def printstar(n)
  for i = 1, i < n, 1.0 in
    putchard(42);  # ascii 42 = '*'

# print 100 '*' characters
printstar(100);",
        )).run()
            .unwrap()
    })
}

benchmark_group!(
    benches,
    parse_extern,
    parse_prototype,
    parse_function,
    parse_number,
    lex_function,
    parse_binop
);
benchmark_main!(benches);

/*
fn main() {
    for _ in 0..10_000 {
        Parser::new(Lexer::new("def foo(x, y) x+y;")).run().unwrap();
    }
}
*/

/*
With smallvec, nodes allocated in a vector
normal String, precedence in a match instead of hashmap
test lex_function    ... bench:         230 ns/iter (+/- 20)
test parse_binop     ... bench:         221 ns/iter (+/- 11)
test parse_extern    ... bench:         236 ns/iter (+/- 25)
test parse_function  ... bench:         454 ns/iter (+/- 51)
test parse_number    ... bench:         149 ns/iter (+/- 14)
test parse_prototype ... bench:         528 ns/iter (+/- 45)
*/
