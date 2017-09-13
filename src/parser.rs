#![allow(unused)]
use std::collections::HashMap;
use lexer::{Lexer, Token};

use llvm_sys::LLVMRealPredicate;
use llvm_sys::core::*;
use llvm_sys::prelude::*;

use std::rc::Rc;
use std::ffi::{CString, CStr};

const LLVM_FALSE: LLVMBool = 0;
const LLVM_TRUE: LLVMBool = 1;

trait ExprAST {
    fn codegen(&self) -> LLVMValueRef;
}

struct NumberExprAST {
    val: f64,
}
impl ExprAST for NumberExprAST {
    fn codegen(&self) -> LLVMValueRef {
        LLVMConstReal(LLVMDoubleType(), self.val)
    }
}

struct VariableExprAST<'a> {
    parser: Rc<Parser<'a>>,
    name: CString,
}
impl<'a> ExprAST for VariableExprAST<'a> {
    fn codegen(&self) -> LLVMValueRef {
        *self.parser.names.get(&self.name).unwrap()
    }
}

struct BinaryExprAST<'a> {
    parser: Rc<Parser<'a>>,
    op: char,
    lhs: Box<ExprAST>,
    rhs: Box<ExprAST>,
}
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

struct CallExprAST<'a> {
    parser: Rc<Parser<'a>>,
    callee: CString,
    args: Vec<Box<ExprAST>>,
}
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

struct PrototypeAST<'a> {
    parser: Rc<Parser<'a>>,
    name: CString,
    args: Vec<CString>,
}
impl<'a> ExprAST for PrototypeAST<'a> {
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

struct FunctionAST<'a> {
    parser: Rc<Parser<'a>>,
    proto: Box<PrototypeAST<'a>>,
    body: Box<ExprAST>,
}
impl<'a> FunctionAST<'a> {
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

pub struct Parser<'a> {
    precedence: HashMap<char, i64>,
    lexer: Lexer<'a>,
    current_token: Token,

    // codegen stuff
    context: LLVMContextRef,
    builder: LLVMBuilderRef,
    module: LLVMModuleRef,
    names: HashMap<CString, LLVMValueRef>,
}

impl<'a> Parser<'a> {
    pub fn new(lex: Lexer<'a>) -> Self {
        let mut precedence = HashMap::new();
        precedence.insert('<', 10);
        precedence.insert('+', 20);
        precedence.insert('-', 20);
        precedence.insert('*', 40);
        let context = LLVMContextCreate();
        let builder = LLVMCreateBuilderInContext(context);
        let module = LLVMModuleCreateWithNameInContext(
            CStr::from_bytes_with_nul_unchecked(b"Kalide Compiler\0").as_ptr(),
            context,
        );
        Parser {
            precedence: precedence,
            lexer: lex,
            current_token: Token::Identifier,
            context: context,
            builder: builder,
            module: module,
            names: HashMap::new(),
        }
    }

    fn get_next_token(&mut self) -> Token {
        self.current_token = self.lexer.next_token();
        self.current_token.clone()
    }

    fn parse_number_expr(&mut self) -> Box<NumberExprAST> {
        println!("parse_number_expr");
        let result = Box::new(NumberExprAST { val: self.lexer.numeric_value });
        self.get_next_token();
        result
    }

    fn parse_paren_expr(&mut self) -> Box<ExprAST> {
        println!("parse_paren_expr");
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
        println!("parse_identifier_expr");
        let id_name = self.lexer.identifier();
        self.get_next_token();
        // simple variable ref
        if self.current_token == Token::Punctuation && self.lexer.current_char != '(' {
            return Box::new(VariableExprAST {
                parser: Rc::new(*self),
                name: id_name,
            });
        }

        println!("found call to function {:?}", id_name);
        self.get_next_token();
        let mut args = Vec::new();
        loop {
            let arg = self.parse_expression();
            args.push(arg);

            if self.current_token == Token::Punctuation && self.lexer.current_char == ')' {
                break;
            }

            if self.current_token == Token::Punctuation && self.lexer.current_char != ',' {
                panic!("Expected ) or , in argument list")
            }
            self.get_next_token();
        }

        // Eat the )
        self.get_next_token();

        Box::new(CallExprAST {
            parser: Rc::new(*self),
            callee: id_name,
            args: args,
        })
    }

    fn parse_primary(&mut self) -> Box<ExprAST> {
        println!("parse_primary");
        match self.current_token {
            Token::Identifier => self.parse_identifier_expr(),
            Token::Number => self.parse_number_expr(),
            Token::Punctuation => {
                if self.lexer.current_char == '(' {
                    self.parse_paren_expr()
                } else {
                    panic!("Expected (, found {}", self.lexer.current_char);
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
        mut lhs: Box<ExprAST>,
    ) -> Box<ExprAST> {
        println!("parse_binop_rhs");
        loop {
            println!("{}", self.lexer.current_char);
            let token_precedence = self.get_token_precedence();
            if token_precedence < expression_precedence {
                return lhs;
            }
            let binop = self.lexer.current_char;
            self.get_next_token();

            let mut rhs = self.parse_primary();

            let next_precedence = self.get_token_precedence();
            if token_precedence < next_precedence {
                rhs = self.parse_binop_rhs(token_precedence + 1, rhs);
            }

            lhs = Box::new(BinaryExprAST {
                parser: Rc::new(*self),
                op: binop,
                lhs: lhs,
                rhs: rhs,
            });
        }
    }

    fn parse_expression(&mut self) -> Box<ExprAST> {
        println!("parse_expression");
        let lhs = self.parse_primary();
        self.parse_binop_rhs(0, lhs)
    }

    fn parse_prototype(&mut self) -> Box<PrototypeAST> {
        println!("parse_prototype");
        if self.current_token != Token::Identifier {
            panic!("Expected function name in prototype");
        }
        let function_name = self.lexer.identifier();

        self.get_next_token();
        if self.lexer.current_char != '(' {
            panic!("Expected ( in prototype");
        }

        let mut argnames = Vec::new();
        while self.get_next_token() == Token::Identifier {
            argnames.push(self.lexer.identifier());
        }

        if self.lexer.current_char != ')' {
            panic!("Expected ) in prototype")
        }

        // eat the )
        self.get_next_token();

        Box::new(PrototypeAST {
            parser: Rc::new(*self),
            name: function_name,
            args: argnames,
        })
    }

    fn parse_definition(&mut self) -> Box<FunctionAST> {
        println!("parse_definition");
        // Eat the def
        self.get_next_token();
        let proto = self.parse_prototype();

        let e = self.parse_expression();
        Box::new(FunctionAST {
            parser: Rc::new(*self),
            proto: proto,
            body: e,
        })
    }

    fn parse_top_level_expr(&mut self) -> Box<FunctionAST> {
        println!("parse_top_level_expr");
        let e = self.parse_expression();
        let proto = Box::new(PrototypeAST {
            parser: Rc::new(*self),
            name: CStr::from_bytes_with_nul_unchecked(b"__anon_expr\0").to_owned(),
            args: Vec::new(),
        });
        Box::new(FunctionAST {
            parser: Rc::new(*self),
            proto: proto,
            body: e,
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
            // Ignore line endings wtf
            if self.lexer.current_char == ';' && self.current_token == Token::Punctuation {
                self.get_next_token();
                continue;
            }
            match self.current_token {
                Token::EOF => return,
                Token::Definition => self.handle_definition(),
                Token::Extern => self.handle_extern(),
                _ => self.handle_top_level_expression(),
            }
        }
    }
}
