use std::str::Chars;

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    EOF,
    Definition,
    Extern,
    Identifier(String),
    Number(f64),
    Punctuation(char),
}

pub struct Lexer<'a> {
    buffer: String,
    getchar: Chars<'a>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Lexer {
        Lexer {
            buffer: String::new(),
            getchar: input.chars(),
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.buffer.clear();

        while let Some(c) = self.getchar.next() {
            if c.is_whitespace() {
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
                while let Some(c) = self.getchar.next() {
                    if c.is_alphabetic() || c.is_numeric() {
                        self.buffer.push(c);
                    } else {
                        break;
                    }
                }
                return match self.buffer.as_str() {
                    "def" => Token::Definition,
                    "extern" => Token::Extern,
                    _ => Token::Identifier(self.buffer.clone()),
                };
            // Numeric literal
            } else if c.is_numeric() || c == '.' {
                self.buffer.push(c);
                while let Some(c) = self.getchar.next() {
                    if c.is_numeric() || c == '.' {
                        self.buffer.push(c)
                    } else {
                        break;
                    }
                }
                return match self.buffer.parse() {
                    Ok(number) => Token::Number(number),
                    Err(..) => panic!("Invalid numeric literal {}", self.buffer),
                };
            } else {
                return Token::Punctuation(c);
            }
        }
        Token::EOF
    }
}
