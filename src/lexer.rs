use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EOF,
    Definition,
    If,
    Then,
    Else,
    For,
    In,
    Identifier(String),
    Number(f64),
    Operator(char),
    EOL,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Colon,
    Comma,
    Equals,
}

pub struct Lexer<'a> {
    pub line: usize,
    buffer: String,
    getchar: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer {
            line: 1,
            buffer: String::with_capacity(4),
            getchar: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.buffer.clear();

        while let Some(c) = self.getchar.next() {
            if c.is_whitespace() {
                if c == '\n' {
                    self.line += 1;
                }
                continue;
            }
            // Comments
            if c == '#' {
                self.getchar.find(|&c| c == '\n');
                continue;
            }
            // Identifier
            if c.is_alphabetic() {
                self.buffer.push(c);
                while let Some(&c) = self.getchar.peek() {
                    if c.is_alphabetic() || c.is_numeric() {
                        self.buffer.push(c);
                        self.getchar.next();
                    } else {
                        break;
                    }
                }
                return match self.buffer.as_ref() {
                    "def" => Token::Definition,
                    "if" => Token::If,
                    "then" => Token::Then,
                    "else" => Token::Else,
                    "for" => Token::For,
                    "in" => Token::In,
                    _ => Token::Identifier(self.buffer.clone()),
                };
            // Numeric literal
            } else if c.is_numeric() || c == '.' {
                self.buffer.push(c);
                while let Some(&c) = self.getchar.peek() {
                    if c.is_numeric() || c == '.' {
                        self.buffer.push(c);
                        self.getchar.next();
                    } else {
                        break;
                    }
                }
                return match self.buffer.parse() {
                    Ok(number) => Token::Number(number),
                    Err(..) => panic!("Invalid numeric literal {}", self.buffer),
                };
            } else {
                return match c {
                    '(' => Token::OpenParen,
                    ')' => Token::CloseParen,
                    ';' => Token::EOL,
                    '{' => Token::OpenBrace,
                    '}' => Token::CloseBrace,
                    ':' => Token::Colon,
                    ',' => Token::Comma,
                    '=' => Token::Equals,
                    '>' => Token::Operator('>'),
                    '<' => Token::Operator('<'),
                    '+' => Token::Operator('+'),
                    '-' => Token::Operator('-'),
                    '*' => Token::Operator('*'),
                    '/' => Token::Operator('/'),
                    _ => unreachable!(&format!("Encountered unknown symbol {}", c)),
                };
            }
        }
        Token::EOF
    }
}
