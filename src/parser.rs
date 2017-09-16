// TODO: Remove Parser::current_token. I'm certain we can avoid this by just passing tokens to some of the parsing functions
// TODO: Remove all the unreachable!() and handle missing semicolons
// TODO: codegen
// TODO: output

#![allow(unused)]
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use lexer::{Lexer, Token};

use llvm_sys::LLVMRealPredicate;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

use std::rc::Rc;
use std::ffi::{CString, CStr};

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

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
enum ExprNode {
    Number(f64),
    Variable(CString),
    BinaryOperation(char, Box<ExprNode>, Box<ExprNode>),
    FunctionCall(CString, Vec<Box<ExprNode>>),
    IfElse(Box<ExprNode>, Box<ExprNode>, Box<ExprNode>),
    ForLoop(Box<ExprNode>, Box<ExprNode>, Option<Box<ExprNode>>, Box<ExprNode>),
}

#[derive(Debug)]
struct Prototype {
    name: CString,
    args: Vec<CString>,
}

#[derive(Debug)]
struct Function {
    proto: Box<Prototype>,
    body: Box<ExprNode>,
}

/*
impl ExprAST for NumberExprAST {
    fn codegen(&self) -> LLVMValueRef {
        LLVMConstReal(LLVMDoubleType(), self.val)
    }
}
*/
/*
impl<'a> ExprAST for VariableExprAST<'a> {
    fn codegen(&self) -> LLVMValueRef {
        *self.parser.names.get(&self.name).unwrap()
    }
}
*/
/*
impl<'a> ExprAST for BinaryExprAST<'a> {
    fn codegen(&self) -> LLVMValueRef {
        let l = self.lhs.codegen();
        let r = self.rhs.codegen();

        match self.op {
            '+' => {
                LLVMBuildFAdd(
                    self.parser.builder,
                    l,
                    r,
                    CStr::from_bytes_with_nul(b"addtmp\0").unwrap().as_ptr(),
                )
            }
            '-' => {
                LLVMBuildFSub(
                    self.parser.builder,
                    l,
                    r,
                    CStr::from_bytes_with_nul_unchecked(b"subtmp\0").as_ptr(),
                )
            }
            '*' => {
                LLVMBuildFMul(
                    self.parser.builder,
                    l,
                    r,
                    CStr::from_bytes_with_nul_unchecked(b"multmp\0").as_ptr(),
                )
            }
            '<' => {
                // Convert to boolean here apparently?
                LLVMBuildFCmp(
                    self.parser.builder,
                    LLVMRealPredicate::LLVMRealORD,
                    l,
                    r,
                    CStr::from_bytes_with_nul_unchecked(b"cmptmp\0").as_ptr(),
                )
            }
            _ => unreachable!(),            
        }
    }
}
*/
/*
impl<'a> ExprAST for CallExprAST<'a> {
    fn codegen(&self) -> LLVMValueRef {
        let llvm_callee = LLVMGetNamedFunction(self.parser.module, self.callee.as_ptr());
        // use LLVMFunctionType to validate the function

        let llvm_args: Vec<_> = self.args.iter().map(|arg| arg.codegen()).collect();
        LLVMBuildCall(
            self.parser.builder,
            llvm_callee,
            llvm_args.as_mut_ptr(),
            llvm_args.len() as u32,
            self.callee.as_ptr(),
        )
    }
}
*/
/*
impl<'a> ExprAST for Prototype<'a> {
    fn codegen(&self) -> LLVMValueRef {
        let arg_types: Vec<_> = self.args
            .iter()
            .map(|_| LLVMDoubleTypeInContext(self.parser.context))
            .collect();
        let fn_type = LLVMFunctionType(
            LLVMDoubleTypeInContext(self.parser.context),
            arg_types.as_mut_ptr(),
            arg_types.len() as u32,
            LLVM_FALSE,
        );
        LLVMAddFunction(self.parser.module, self.name.as_ptr(), fn_type)
    }
}
*/
/*
impl<'a> Function<'a> {
    fn codegen(&mut self) {
        let the_function = LLVMGetNamedFunction(self.parser.module, self.proto.name.as_ptr());
        let basic_block = LLVMGetInsertBlock(self.parser.builder);
        LLVMInsertIntoBuilderWithName(
            self.parser.builder,
            LLVMBasicBlockAsValue(basic_block),
            CStr::from_bytes_with_nul_unchecked(b"entry\0").as_ptr(),
        );
        self.parser.names.clear();

    }
}
*/

pub struct Parser<'a> {
    precedence: HashMap<char, i64>,
    lexer: Lexer<'a>,
    current_token: Token,
    output: String,

    // codegen stuff
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    names: HashMap<CString, LLVMValueRef>,
}


impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Self {
        unsafe {
            let mut precedence = HashMap::with_capacity(4);
            precedence.insert('<', 10);
            precedence.insert('+', 20);
            precedence.insert('-', 20);
            precedence.insert('*', 40);
            let context = LLVMContextCreate();
            let builder = LLVMCreateBuilderInContext(context);
            let module = LLVMModuleCreateWithNameInContext(
                CStr::from_bytes_with_nul(b"Kalide Compiler\0")
                    .unwrap()
                    .as_ptr(),
                context,
            );
            Parser {
                precedence: precedence, // TODO this is global _data_
                lexer: lex,
                output: String::new(),
                current_token: Token::Identifier("".to_owned()),
                context: context,
                builder: builder,
                module: module,
                names: HashMap::new(),
            }
        }
    }

    fn get_next_token(&mut self) -> Token {
        self.current_token = self.lexer.next_token();
        self.current_token.clone()
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
        if let Token::Punctuation('(') = self.current_token {
            // Eat the (
            // TODO: Both of these get_next_token need to match to ensure they are correct and return errors if not
            self.get_next_token();

            let v = self.parse_expression();
            // Eat the )
            self.get_next_token();
            v
        } else {
            Err(self.error("Error parsing paren expression"))
        }
    }

    // ::= identifier
    // ::= identifier '(' expression* ')'
    fn parse_identifier_expr(&mut self) -> Result<Box<ExprNode>, ParseError> {
        let id_name = match self.current_token {
            Token::Identifier(ref name) => name.clone(),
            _ => return Err(self.error("Expected valid identifier")),
        };

        // eat the identifier
        // simple variable ref
        if let Token::Punctuation(c) = self.get_next_token() {
            if c != '(' {
                return Ok(Box::new(ExprNode::Variable(
                    CString::new(id_name.as_bytes()).unwrap(),
                )));
            }
        }

        self.get_next_token();
        let mut args = Vec::new();
        loop {
            if let Ok(arg) = self.parse_expression() {
                args.push(arg);
            }

            if let Token::Punctuation(')') = self.current_token {
                break;
            }

            if let Token::Punctuation(c) = self.current_token {
                if c != ',' {
                    return Err(self.error(&format!(
                        "Expected ) or , in argument list. Found {:?}",
                        self.current_token
                    )));
                }
            }

            self.get_next_token();
        }

        // Eat the )
        self.get_next_token();

        Ok(Box::new(ExprNode::FunctionCall(
            CString::new(id_name.as_bytes()).unwrap(),
            args,
        )))
    }

    fn parse_primary(&mut self) -> Result<Box<ExprNode>, ParseError> {
        match self.current_token {
            Token::Identifier(..) => self.parse_identifier_expr(),
            Token::Number(..) => self.parse_number_expr(),
            Token::Punctuation('(') => self.parse_paren_expr(),
            Token::If => self.parse_if_expr(),
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
                Token::Punctuation(op) => op,
                _ => unreachable!(),
            };
            let token_precedence = *self.precedence.get(&binop).unwrap_or(&-1);
            if token_precedence < expression_precedence {
                return Ok(lhs);
            }

            self.get_next_token();
            let mut rhs = self.parse_primary()?;

            let next_precedence = if let Token::Punctuation(c) = self.current_token {
                *self.precedence.get(&c).unwrap_or(&-1)
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
                        '<' => {
                            match l < r {
                                true => 1.0,
                                false => 0.0,
                            }
                        }
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
            if let Token::Punctuation(c) = self.get_next_token() {
                if c != '(' {
                    return Err(self.error("Expected ( in prototype"));
                }
            }

            let mut argnames = Vec::new();
            self.get_next_token();
            while let Token::Identifier(id) = self.current_token.clone() {
                argnames.push(CString::new(id.as_bytes()).unwrap());
                self.get_next_token();
                if let Token::Punctuation(',') = self.current_token {
                    self.get_next_token();
                }

            }

            if let Token::Punctuation(c) = self.current_token {
                if c != ')' {
                    return Err(self.error("Expected ) in prototype"));
                }
            }
            // eat the )
            self.get_next_token();

            Ok(Box::new(Prototype {
                name: CString::new(function_name.as_bytes()).unwrap(),
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
        self.get_next_token();
        let id_name = match self.current_token {
            Token::Identifier(ref name) => name.clone(),
            _ => return Err(self.error("Expected for")),
        };

        self.get_next_token();
        match self.current_token {
            Token::Punctuation('=') => {}
            _ => return Err(self.error("Expected for")),
        };

        self.get_next_token();
        let start = self.parse_expression()?;
        match self.current_token {
            Token::Punctuation(',') => {}
            _ => return Err(self.error("expected , after start value")),
        };

        self.get_next_token();
        let end = self.parse_expression()?;

        // Somehow the step is optional
        let step = match self.current_token {
            Token::Punctuation(',') => Some(self.parse_expression()?),
            _ => None,
        };

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
            name: CStr::from_bytes_with_nul(b"__anon_expr\0")
                .unwrap()
                .to_owned(),
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
        ParseError { reason: format!("{} on line {}", message, self.lexer.line) }
    }

    fn handle_definition(&mut self) {
        match self.parse_definition() {
            Ok(def) => println!("{:#?}", def),
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

    pub fn run(&mut self) -> Result<(), ParseError> {
        self.get_next_token();
        loop {
            match self.current_token {
                Token::Punctuation(';') => {
                    self.get_next_token();
                } // ignores line endings???
                Token::EOF => return Ok(()),
                Token::Definition => self.handle_definition(),
                Token::Extern => self.handle_extern(),
                _ => self.handle_top_level_expression(),
            }
        }
        print!("{}", self.output);
    }
}
