use std::collections::HashMap;
use lexer::{Lexer, Token};

trait ExprAST {}

struct NumberExprAST {
    Val: f64,
}
impl ExprAST for NumberExprAST {}

struct VariableExprAST {
    Name: String,
}
impl ExprAST for VariableExprAST {}

struct BinaryExprAST {
    Op: char,
    LHS: Box<ExprAST>,
    RHS: Box<ExprAST>,
}
impl ExprAST for BinaryExprAST {}

struct CallExprAST {
    Callee: String,
    Args: Vec<Box<ExprAST>>,
}
impl ExprAST for CallExprAST {}

struct PrototypeAST {
    Name: String,
    Args: Vec<String>,
}
impl ExprAST for PrototypeAST {}

struct FunctionAST {
    Proto: Box<PrototypeAST>,
    Body: Box<ExprAST>,
}
impl ExprAST for FunctionAST {}

pub struct Parser<'a> {
    precedence: HashMap<char, i64>,
    lexer: Lexer<'a>,
    current_token: Token,
}

impl<'a> Parser<'a> {
    pub fn new(l: Lexer<'a>) -> Self {
        let mut precedence = HashMap::new();
        precedence.insert('<', 10);
        precedence.insert('+', 20);
        precedence.insert('-', 20);
        precedence.insert('*', 40);
        Parser {
            precedence: precedence,
            lexer: l,
            current_token: Token::Identifier,
        }
    }

    fn get_next_token(&mut self) -> Token {
        self.current_token = self.lexer.next_token();
        self.current_token.clone()
    }

    fn parse_number_expr(&mut self) -> Box<NumberExprAST> {
        let result = Box::new(NumberExprAST { Val: self.lexer.numeric_value });
        self.get_next_token();
        result
    }

    fn parse_paren_expr(&mut self) -> Box<ExprAST> {
        // eat the (
        self.get_next_token();
        let v = self.parse_expression();
        // eat the )
        self.get_next_token();
        v
    }

    // ::= identifier
    // ::= identifier '(' expression* ')'
    fn parse_identifier_expr(&mut self) -> Box<ExprAST> {
        let id_name = self.lexer.identifier.clone();
        self.get_next_token();
        // simple variable ref
        if self.current_token == Token::Punctuation && self.lexer.current_char == '(' {
            return Box::new(VariableExprAST { Name: id_name });
        }

        self.get_next_token();
        let mut args = Vec::new();
        loop {
            let arg = self.parse_expression();
            args.push(arg);

            if self.current_token == Token::Punctuation && self.lexer.current_char == ')' {
                break;
            }

            if self.lexer.current_char != ',' {
                panic!("Expected ) or , in argument list")
            }
            self.get_next_token();
        }

        self.get_next_token();

        Box::new(CallExprAST {
            Callee: id_name,
            Args: args,
        })
    }

    fn parse_primary(&mut self) -> Box<ExprAST> {
        match self.current_token {
            Token::Identifier => self.parse_identifier_expr(),
            Token::Number => self.parse_number_expr(),
            Token::Punctuation => {
                if self.lexer.current_char == '(' {
                    self.parse_paren_expr()
                } else {
                    panic!("unknown token when expecting an expression")
                }
            }
            _ => panic!("unknown token when expecting an expression"),
        }
    }

    fn get_token_precedence(&mut self) -> i64 {
        *self.precedence.get(&self.lexer.current_char).unwrap_or(&-1)
    }

    fn parse_binop_rhs(
        &mut self,
        expression_precedence: i64,
        mut LHS: Box<ExprAST>,
    ) -> Box<ExprAST> {
        loop {
            let token_precedence = self.get_token_precedence();
            if token_precedence < expression_precedence {
                return LHS;
            }
            let binop = self.lexer.current_char;
            self.get_next_token();
            let mut RHS = self.parse_primary();

            let next_precedence = self.get_token_precedence();
            if token_precedence < next_precedence {
                RHS = self.parse_binop_rhs(token_precedence + 1, RHS);
            }

            LHS = Box::new(BinaryExprAST {
                Op: binop,
                LHS: LHS,
                RHS: RHS,
            });
        }
    }

    fn parse_expression(&mut self) -> Box<ExprAST> {
        let LHS = self.parse_primary();
        self.parse_binop_rhs(0, LHS)
    }

    fn parse_prototype(&mut self) -> Box<PrototypeAST> {
        if self.current_token != Token::Identifier {
            panic!("Expected function name in prototype");
        }
        let function_name = self.lexer.identifier.clone();

        self.get_next_token();
        if self.lexer.current_char != '(' {
            panic!("Expected ( in prototype");
        }

        let mut argnames = Vec::new();
        while self.get_next_token() == Token::Identifier {
            argnames.push(self.lexer.identifier.clone());
        }

        if self.lexer.current_char != ')' {
            panic!("Expected ) in prototype")
        }

        // eat the )
        self.get_next_token();

        Box::new(PrototypeAST {
            Name: function_name,
            Args: argnames,
        })
    }

    fn parse_definition(&mut self) -> Box<FunctionAST> {
        // Eat the def
        self.get_next_token();
        let proto = self.parse_prototype();

        let e = self.parse_expression();
        Box::new(FunctionAST {
            Proto: proto,
            Body: e,
        })
    }

    fn parse_top_level_expr(&mut self) -> Box<FunctionAST> {
        let e = self.parse_expression();
        let proto = Box::new(PrototypeAST {
            Name: "__anon_expr".to_owned(),
            Args: Vec::new(),
        });
        Box::new(FunctionAST {
            Proto: proto,
            Body: e,
        })
    }

    fn parse_extern(&mut self) -> Box<PrototypeAST> {
        self.get_next_token();
        self.parse_prototype()
    }

    fn handle_definition(&mut self) {
        let expr = self.parse_definition();
        println!("Parsed function definition");
    }

    fn handle_extern(&mut self) {
        let expr = self.parse_extern();
        println!("Parsed extern");
    }

    fn handle_top_level_expression(&mut self) {
        let expr = self.parse_top_level_expr();
        println!("Parsed top-level expression");
    }

    pub fn run(&mut self) {
        self.get_next_token();
        loop {
            match self.current_token {
                Token::EOF => return,
                Token::Definition => self.handle_definition(),
                Token::Extern => self.handle_extern(),
                _ => self.handle_top_level_expression(),
            }
        }
    }
}
