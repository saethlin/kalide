use std::str::Chars;

pub enum Token {
    EOF = -1,
    // commands
    Definition = -2,
    Extern = -3,

    // primary
    Identifier = -4,
    Number = -5,
}

pub struct Lexer<'a> {
    identifier: String,
    numeric: String,
    numeric_value: f64,
    getchar: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer {
            identifier: String::new(),
            numeric: String::new(),
            numeric_value: 0.0,
            getchar: input.chars(),
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
                // Eat an identifier
                if c.is_alphabetic() {
                    self.identifier.extend(self.getchar.clone().take_while(
                        |c| c.is_alphabetic() || c.is_numeric(),
                    ));
                    self.getchar.nth(self.identifier.len());

                // Eat a numeric literal
                } else if c.is_numeric() || c == '.' {
                    self.numeric.extend(self.getchar.clone().take_while(
                        |c| c.is_numeric() || *c == '.',
                    ));
                    self.numeric_value = self.numeric.parse().unwrap();
                    self.getchar.nth(self.numeric.len());

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
                    unreachable!()
                }
            }
            None => Token::EOF,
        }
    }
}
