// TODO: Remove all the unreachable!() and handle missing semicolons
// TODO: codegen
// TODO: print function
// TODO: assignment
// TODO: memory allocation

use lexer::{Lexer, Token};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    reason: String,
}

impl Error for ParseError {
    fn description(&self) -> &str {
        &self.reason
    }
    fn cause(&self) -> Option<&Error> {
        None
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.reason)
    }
}

#[derive(Debug)]
pub enum ExprNode {
    Number(f64),
    Variable(String),
    BinaryOperation(char, Box<ExprNode>, Box<ExprNode>),
    FunctionCall(String, Vec<Box<ExprNode>>),
    IfElse(Box<ExprNode>, Box<ExprNode>, Box<ExprNode>),
    ForLoop(
        Box<ExprNode>,
        Box<ExprNode>,
        Option<Box<ExprNode>>,
        Box<ExprNode>,
    ),
    Assignment(String, 
}

impl ExprNode {
    pub fn codegen(&self, code: &mut String) {
        match *self {
            ExprNode::Number(value) => {}
            _ => {}
        }
    }
}

#[derive(Debug)]
struct Prototype {
    name: String,
    arguments: Vec<(String, Type)>,
    return_type: Type,
}

#[derive(Debug)]
struct Function {
    prot: Prototype,
    body: Vec<Box<ExprNode>>,
}

pub struct Parser<'a> {
    top_level_functions: Vec<Function>,
    lexer: Lexer<'a>,
    current_token: Token,
    output: String,
}


fn precedence(c: char) -> i64 {
    match c {
        '<' | '>' => 10,
        '+' | '-' => 20,
        '*' | '/' => 40,
        _ => -1,
    }
}

impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Self {
        Parser {
            top_level_functions: Vec::new(),
            lexer: lex,
            current_token: Token::EOL,
            output: String::new(),
        }
    }

    fn get_next_token(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn parse_number_expr(&mut self) -> Result<Box<ExprNode>, ParseError> {
        if let Token::Number(value) = self.current_token {
            // Consume the number token
            self.get_next_token();
            Ok(Box::new(ExprNode::Number(value)))
        } else {
            Err(self.error("Not a number token"))
        }
    }

    fn parse_paren_expr(&mut self) -> Result<Box<ExprNode>, ParseError> {
        if let Token::OpenParen = self.current_token {
            // Eat the (
            // TODO: Both of these get_next_token need to match
            // to ensure they are correct and return errors if not
            self.get_next_token();

            let expr = self.parse_expression();
            // Eat the )
            self.get_next_token();
            expr
        } else {
            Err(self.error("Error parsing paren expression"))
        }
    }

    fn parse_identifier_expr(&mut self) -> Result<Box<ExprNode>, ParseError> {
        let id_name = match self.current_token {
            Token::Identifier(ref name) => name.clone(),
            _ => return Err(self.error("Expected valid identifier")),
        };

        // We could have encountered a function call or just a variable access.
        // First we check for a simple variable access
        self.get_next_token();
        match self.current_token {
            Token::OpenParen => {}, // Continue on if this is a function call
            _ => return Ok(Box::new(ExprNode::Variable(id_name))),
        }

        self.get_next_token();
        let mut args = Vec::new();
        loop {
            if let Ok(arg) = self.parse_expression() {
                args.push(arg);
            }

            match self.current_token {
                Token::Comma => {},
                Token::CloseParen => break,
                _ => return Err(self.error("Expected ) or , in argument list")),
            }

            self.get_next_token();
        }

        // Eat the )
        self.get_next_token();

        Ok(Box::new(ExprNode::FunctionCall(id_name, args)))
    }

    fn parse_primary(&mut self) -> Result<Box<ExprNode>, ParseError> {
        match self.current_token {
            Token::Identifier(..) => self.parse_identifier_expr(),
            Token::Number(..) => self.parse_number_expr(),
            Token::OpenParen => self.parse_paren_expr(),
            Token::If => self.parse_if_expr(),
            Token::For => self.parse_for_expr(),
            _ => Err(self.error("Unknown token when expecting an expression")),
        }
    }

    // TODO check this function
    fn parse_binop_rhs(
        &mut self,
        expression_precedence: i64,
        mut lhs: Box<ExprNode>,
    ) -> Result<Box<ExprNode>, ParseError> {
        loop {
            let binop = match self.current_token {
                Token::Operator(op) => op,
                _ => unreachable!(),
            };
            let token_precedence = precedence(binop);
            if token_precedence < expression_precedence {
                return Ok(lhs);
            }

            self.get_next_token();
            let mut rhs = self.parse_primary()?;

            let next_precedence = if let Token::Operator(c) = self.current_token {
                precedence(c)
            } else {
                unreachable!()
            };

            if token_precedence < next_precedence {
                rhs = self.parse_binop_rhs(token_precedence + 1, rhs)?;
            }

            // Constant folding
            lhs = if let ExprNode::Number(l) = *lhs {
                if let ExprNode::Number(r) = *rhs {
                    let value = match binop {
                        '+' => l + r,
                        '-' => l - r,
                        '*' => l * r,
                        '<' => if l < r {
                            1.0
                        } else {
                            0.0
                        },
                        _ => unreachable!("Unimplemented binary operation"),
                    };
                    Box::new(ExprNode::Number(value))
                } else {
                    Box::new(ExprNode::BinaryOperation(binop, lhs, rhs))
                }
            } else {
                Box::new(ExprNode::BinaryOperation(binop, lhs, rhs))
            };
        }
    }

    fn parse_expression(&mut self) -> Result<Box<ExprNode>, ParseError> {
        let lhs = self.parse_primary()?;
        self.parse_binop_rhs(0, lhs)
    }

    fn parse_prototype(&mut self) -> Result<Box<Prototype>, ParseError> {
        if let Token::Identifier(function_name) = self.current_token.clone() {
            self.get_next_token();
            match self.current_token {
                Token::OpenParen => {},
                _ => return Err(self.error("Expected ( in prototype")),
            }

            let mut argnames = Vec::new();
            self.get_next_token();
            while let Token::Identifier(id) = self.current_token.clone() {
                argnames.push(id);
                self.get_next_token();
                if let Token::Comma = self.current_token {
                    self.get_next_token();
                }
            }

            match self.current_token {
                Token::CloseParen => {},
                _ => return Err(self.error("Espected ) in prototype")),
            }
            // eat the )
            self.get_next_token();

            Ok(Box::new(Prototype {
                name: function_name,
                args: argnames,
            }))
        } else {
            Err(self.error("Expected identifier after def"))
        }
    }

    fn parse_definition(&mut self) -> Result<Box<Function>, ParseError> {
        // Eat the def
        self.get_next_token();
        let proto = self.parse_prototype()?;

        let body = self.parse_expression()?;
        Ok(Box::new(Function {
            proto: proto,
            body: body,
        }))
    }

    fn parse_if_expr(&mut self) -> Result<Box<ExprNode>, ParseError> {
        self.get_next_token();

        let condition = self.parse_expression()?;
        match self.current_token {
            Token::Then => {}
            _ => return Err(self.error("Expected then")),
        }

        let then_expr = self.parse_expression()?;
        match self.current_token {
            Token::Else => {}
            _ => return Err(self.error("Expected else")),
        }

        let else_expr = self.parse_expression()?;
        Ok(Box::new(ExprNode::IfElse(condition, then_expr, else_expr)))
    }

    fn parse_for_expr(&mut self) -> Result<Box<ExprNode>, ParseError> {
        println!("Parsing loop");
        self.get_next_token();
        let id_name = match self.current_token {
            Token::Identifier(ref name) => name.clone(),
            _ => return Err(self.error("Expected identifier after for")),
        };

        println!("Parsing =");
        self.get_next_token();
        match self.current_token {
            Token::Equals => {}
            _ => return Err(self.error("Expected = after for")),
        };

        println!("Parsing start");
        self.get_next_token();
        let start = self.parse_expression()?;
        match self.current_token {
            Token::Comma => {}
            _ => return Err(self.error("expected , after start value")),
        };

        println!("Parsing end");
        self.get_next_token();
        let end = self.parse_expression()?;

        println!("Parsing step");
        // Somehow the step is optional
        let step = match self.current_token {
            Token::Comma => Some(self.parse_expression()?),
            _ => None,
        };

        println!("Parsing in");
        match self.current_token {
            Token::In => {}
            _ => return Err(self.error("expected 'in' after for")),
        };

        let body = self.parse_expression()?;

        Ok(Box::new(ExprNode::ForLoop(start, end, step, body)))
    }

    fn parse_top_level_expr(&mut self) -> Result<Box<Function>, ParseError> {
        let body = self.parse_expression()?;
        let proto = Box::new(Prototype {
            name: String::from("__anon_expr"),
            args: Vec::new(),
        });
        Ok(Box::new(Function {
            proto: proto,
            body: body,
        }))
    }

    fn parse_extern(&mut self) -> Result<Box<Prototype>, ParseError> {
        // eat the extern
        self.get_next_token();
        self.parse_prototype()
    }

    fn error(&self, message: &str) -> ParseError {
        ParseError {
            reason: format!("{} on line {}", message, self.lexer.line),
        }
    }

    fn handle_definition(&mut self) {
        match self.parse_definition() {
            Ok(def) => {}
            Err(e) => self.output.push_str(&format!("{}\n", e)),
        };
    }

    fn handle_extern(&mut self) {
        match self.parse_extern() {
            Ok(..) => {}
            Err(e) => self.output.push_str(&format!("{}\n", e)),
        };
    }

    fn handle_top_level_expression(&mut self) {
        match self.parse_top_level_expr() {
            Ok(..) => {}
            Err(e) => self.output.push_str(&format!("{}\n", e)),
        };
    }

    // Find every top-level function definition, launch a new parser for each one
    pub fn run(&mut self) -> Result<Vec<ExprNode>, ParseError> {
        // Need to prime the lexer
        self.get_next_token();
        loop {
            match self.current_token {
                Token::EOL => self.get_next_token(),
                Token::EOF => break,
                Token::Definition => {
                    toplevel_symbols.push(self.parse_prototype());
                    
                },
            }
        }
        print!("{}", self.output);
    }
}
