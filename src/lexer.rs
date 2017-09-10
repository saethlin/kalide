use std::str::Chars;
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    EOF,
    Definition,
    Extern,
    Identifier,
    Number,
    Punctuation,
}

pub struct Lexer<'a> {
    pub current_char: char,
    pub identifier: String,
    numeric: String,
    pub numeric_value: f64,
    getchar: Peekable<Chars<'a>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer {
            current_char: 0 as char,
            identifier: String::new(),
            numeric: String::new(),
            numeric_value: 0.0,
            getchar: input.chars().peekable(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.identifier.clear();
        self.numeric.clear();

        let mut c = self.getchar.next();
        while c.is_some() && c.unwrap().is_whitespace() {
            c = self.getchar.next();
        }

        match c {
            Some(c) => {
                self.current_char = c;

                // Eat an identifier
                if c.is_alphabetic() {
                    self.identifier.push(c);
                    self.identifier.extend(self.getchar.clone().take_while(
                        |c| c.is_alphabetic() || c.is_numeric(),
                    ));
                    if self.identifier.len() > 1 {
                        self.getchar.nth(self.identifier.len() - 2);
                    }

                // Eat a numeric literal
                } else if c.is_numeric() || c == '.' {
                    self.numeric.push(c);
                    self.numeric.extend(self.getchar.clone().take_while(
                        |c| c.is_numeric() || *c == '.',
                    ));
                    self.numeric_value = self.numeric.parse().unwrap();
                    if self.numeric.len() > 1 {
                        self.getchar.nth(self.numeric.len() - 2);
                    }


                // Eat comments
                } else if c == '#' {
                    self.getchar.position(|c| c == '\n');
                    return self.next_token();
                };

                if self.identifier.as_str() == "def" {
                    Token::Definition
                } else if self.identifier.as_str() == "extern" {
                    Token::Extern
                } else if !self.identifier.is_empty() {
                    Token::Identifier
                } else if !self.numeric.is_empty() {
                    Token::Number
                } else {
                    Token::Punctuation
                }
            }
            None => Token::EOF,
        }
    }
}
