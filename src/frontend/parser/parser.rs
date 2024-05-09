// Ethan Olchik
// src/parser/parser.rs
// This file contains the implementation of the top-down recursive descent parser

//> Imports

use std::vec;
use codespan_reporting::diagnostic::Label;

use crate::{
    ast::ast::*,
    errors::errors::Error,
    frontend::lexer::token::{Token, TokenType},
    semantic::types::{Type, TypeKind, UserType, UserTypeKind, Modifiers},
    utils::Position,
};

//> Struct Definition

pub struct Parser {
    pub filename: String,
    pub content: String,
    pub tokens: Vec<Token>,
    pub current: usize,
    pub had_error: bool,
    pub panic_mode: bool, // This is used to prevent cascading errors

    pub module: Option<Module>,
}

//> Implementation

impl Parser {
    pub fn new(filename: String, content: String, tokens: Vec<Token>) -> Self {
        Self {
            filename,
            content,
            tokens,
            current: 0,
            had_error: false,
            panic_mode: false,

            module: None,
        }
    }

    pub fn parse(&mut self) -> Module {
        self.expect(
            TokenType::Module,
            format!("Expected module declaration at the beginning of the file, got {:?}", self.peek())
        );

        let mut name = self.expect(
            TokenType::Identifier,
            format!("Expected module name after 'module', got {:?}", self.peek())
        );

        while self.match_token(vec![TokenType::Dot]) {
            let next = self.expect(
                TokenType::Identifier,
                format!("Expected module name after '.', got {:?}", self.peek())
            );

            name.lexeme += format!(".{}", next.lexeme).as_str();
        }

        self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after module name, got {:?}.\nNote: the first module declaration cannot be a module block.", self.peek())
        );

        let mut module = Module {
            name: Identifier { name },
            filename: self.filename.clone(),
            statements: vec![],
            imports: vec![],
        };
        self.module = Some(module.clone());

        while !self.is_at_end() {
            let statement = self.top_level_statement();
            module.statements.push(statement);
        }

        module        
    }

    /// <Strong>Top Level Statements:</strong><br>
    /// Imports, Functions, Structs, Enums, Global Constants
    fn top_level_statement(&mut self) -> Box<Statement> {
        let x: Box<Statement>;
        if self.match_token(vec![TokenType::Import]) {
            x = Box::new(self.import_statement());
        }
        else if self.match_token(vec![TokenType::Func]) {
            x = Box::new(self.function_declaration());
        }
        else if self.match_token(vec![TokenType::Struct]) {
            x = Box::new(self.struct_declaration());
        }
        else if self.match_token(vec![TokenType::Enum]) {
            x = Box::new(self.enum_declaration());
        }
        else if self.match_token(vec![TokenType::Const]) {
            x = Box::new(self.variable_declaration(true));
        } else if self.match_token(vec![TokenType::Export]) {
            x = Box::new(self.export_block());
        }
        else {
            x = Box::new(self.statement());
        }

        if self.had_error {
            self.synchronize();
        }

        x
    }

    fn import_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        let mut path = self.expect(
            TokenType::Identifier,
            format!("Expected module name after 'import', got {:?}", self.peek())
        );

        while self.match_token(vec![TokenType::Dot]) {
            let next = self.expect(
                TokenType::Identifier,
                format!("Expected module name after '.', got {:?}", self.peek())
            );

            path.lexeme += format!(".{}", next.lexeme).as_str();
        }

        let mut alias: Option<String> = None;
        if self.match_token(vec![TokenType::As]) {
            alias = Some(self.expect(
                TokenType::Identifier,
                format!("Expected alias name after 'as', got {:?}", self.peek())
            ).lexeme);
        }

        self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after import statement, got {:?}", self.peek())
        );

        let import = Import {
            path: Expression {
                kind: ExpressionKind::Identifier(Box::new(Identifier {
                    name: path.clone()
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: path.line,
                    start_pos: start.pos,
                    end_pos: path.pos
                }
            },
            alias
        };

        self.add_import(import.clone());

        Statement {
            kind: StatementKind::Import(Box::new(import)),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn add_import(&mut self, import: Import) {
        let mut module = self.module.clone().unwrap();
        module.imports.push(import.clone());
        self.module = Some(module);
    }

    fn function_declaration(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();
        let mut is_method = false;
        let mut struct_name =  Token::new(TokenType::Identifier, String::from(""), 0, 0);
        let mut struct_ref_name = Token::new(TokenType::Identifier, String::from(""), 0, 0);

        if self.match_token(vec![TokenType::LeftParen]) {
            is_method = true;

            struct_name = self.expect(
                TokenType::Identifier,
                format!("Expected struct name, got {:?}", self.peek()).to_string()
            );

            struct_ref_name = self.expect(
                TokenType::Identifier,
                format!("Expected struct reference name, got {:?}", self.peek()).to_string()
            );

            self.expect(
                TokenType::RightParen,
                format!("Expected ')', got {:?}", self.peek()).to_string()
            );
        }

        let name = self.expect(
            TokenType::Identifier,
            format!("Expected function name, got {:?}", self.peek()).to_string()
        );

        self.expect(
            TokenType::LeftParen,
            format!("Expected '(' after function name, got {:?}", self.peek()).to_string()
        );

        let parameters = self.parameters();
        self.expect(
            TokenType::RightParen,
            format!("Expected ')' after parameters, got {:?}", self.peek()).to_string()
        );

        self.expect(
            TokenType::Colon,
            format!("Expected ':' before function type, got {:?}", self.peek()).to_string()
        );

        let type_ = self.type_annotation(true, true);

        let body: Statement;
        if type_.modifiers.is_extern {
            body = Statement {
                kind: StatementKind::Block(Box::new(Block {
                    statements: vec![]
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: start.line,
                    start_pos: start.pos,
                    end_pos: start.pos
                }
            };

            self.expect(
                TokenType::Semicolon,
                format!("Expected ';' after extern function declaration, got {:?}", self.peek()).to_string()
            );
        } else {
            self.expect(
                TokenType::LeftBrace,
                format!("Expected '{{' before function body, got {:?}", self.peek()).to_string()
            );

            body = self.block_statement();
        }

        Statement {
            kind: StatementKind::Function(Box::new(Function {
                name: Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                    
                },
                parameters,
                type_,
                body: Box::new(body),
                is_method,
                struct_name: Some(Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: struct_name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: struct_name.line,
                        start_pos: start.pos,
                        end_pos: struct_name.pos
                    },
                    
                }),
                struct_ref_name: Some(Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: struct_ref_name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: struct_ref_name.line,
                        start_pos: struct_name.pos,
                        end_pos: struct_ref_name.pos
                    },
                    
                })
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }

    }

    fn parameters(&mut self) -> Vec<Variable> {
        let start = self.tokens[self.current].clone();
        let mut parameters = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                let name = self.expect(
                    TokenType::Identifier,
                    format!("Expected parameter name, got {:?}", self.peek()).to_string()
                );

                self.expect(
                    TokenType::Colon,
                    format!("Expected ':' after parameter name, got {:?}", self.peek()).to_string()
                );

                let type_ = self.type_annotation(false, true);
                let mut value: Option<Expression> = None;
                if self.match_token(vec![TokenType::Equals]) {
                    value = Some(self.expression());
                }

                parameters.push(Variable {
                    name: Expression {
                        kind: ExpressionKind::Identifier(Box::new(Identifier {
                            name: name.clone()
                        })),
                        pos: Position {
                            start_line: start.line,
                            end_line: name.line,
                            start_pos: start.pos,
                            end_pos: name.pos
                        },
                        
                    },
                    value,
                    type_: type_.clone(),
                    is_field: false,
                    owner: None
                });

                if !self.match_token(vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        parameters
    }

    fn type_annotation(&mut self, can_have_modifiers: bool, can_have_type: bool) -> Type {
        let mut modifiers = Modifiers::new(
            false,
            false,
            false,
            false,
            false
        );

        loop {
            if self.match_token(vec![TokenType::Export]) {
                self.error(
                    self.previous(),
                    "Can only export symbols in an export block at the end of a source file.".to_string(),
                    None
                );
            }
            if self.match_token(vec![
                TokenType::Pub,
                TokenType::Extern, TokenType::Static, TokenType::Const
            ]) {
                if can_have_modifiers {
                    modifiers.from_tokentype(self.previous().kind);
                } else {
                    self.error(
                        self.previous(),
                        format!("Unexpected modifier {:?}", self.previous().kind).to_string(),
                        None
                    )
                }
            } else {
                break;
            }
        }

        let mut type_ = self.type_(can_have_type);

        type_.modifiers = modifiers;

        type_
    }

    fn type_(&mut self, can_have_type: bool) -> Type {
        let start = self.tokens[self.current-1].clone();
        let mut ident: Token = Token::new(
            TokenType::Identifier,
            String::from(""),
            0,
            0
        );
        let mut kind = TypeKind::Void;
        let mut found_type = false;

        if can_have_type {
            ident = self.expect(
                TokenType::Identifier,
                format!("Expected type name, got {:?}", self.peek()).to_string()
            );

            if self.match_token(vec![TokenType::Dot]) {
                let mut suffix = self.expect(
                    TokenType::Identifier,
                    format!("Expected type name after '.', got {:?}", self.peek()).to_string()
                );

                suffix.lexeme = ident.clone().lexeme + format!(".{}", suffix.lexeme).as_str();

                while self.match_token(vec![TokenType::Dot]) {
                    let x = self.expect(
                        TokenType::Identifier,
                        format!("Expected type name after '.', got {:?}", self.peek()).to_string()
                    );

                    suffix.lexeme += format!(".{}", x.lexeme).as_str();
                }

                found_type = true;
                kind = TypeKind::UserType(UserType {
                    kind: UserTypeKind::Unknown,
                    name: suffix.lexeme
                });
            }

            if self.match_token(vec![TokenType::LeftBracket]) {
                let mut dimension: usize = 1;

                while !self.check(TokenType::RightBracket) {
                    if self.match_token(vec![TokenType::Comma]) {
                        dimension += 1;
                    } else {
                        break;
                    }
                }
                self.expect(
                    TokenType::RightBracket,
                    format!("Expected ']' after array type, got {:?}", self.peek()).to_string()
                );
                
                match kind {
                    TypeKind::Void => {
                        kind = Type::from_string(ident.lexeme.as_str())
                    }
                    _ => {}
                }

                // create array type inclusive of multiple dimensions, e.g. Int32[,] -> TypeKind::Array(TypeKind::Array(TypeKind::Int32))
                return Type::new_array_type(
                    Type::new(
                        kind,
                        Modifiers::new(false, false, false, false, false),
                        Position {
                            start_line: start.line,
                            end_line: ident.line,
                            start_pos: start.pos,
                            end_pos: ident.pos
                        }
                    ),
                    dimension
                );
            } else {
                if !found_type {
                    kind = Type::from_string(ident.lexeme.as_str());
                }
            }
        } else {
            kind = TypeKind::Error;
            if self.peek().kind != TokenType::LeftBrace {
                self.error(
                    self.peek(),
                    format!("Unexpected type name {:?}", self.peek()).to_string(),
                    None
                );
            }
        }
        

        Type::new(
            kind,
            Modifiers::new(false, false, false, false, false),
            Position {
                start_line: start.line,
                end_line: ident.line,
                start_pos: start.pos,
                end_pos: ident.pos
            }
        )
    }

    fn block_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();
        let mut statements: Vec<Box<Statement>> = vec![];

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let statement = self.top_level_statement();
            statements.push(statement);

            if self.panic_mode {
                self.synchronize();
            }
        }

        self.expect(
            TokenType::RightBrace,
            format!("Expected '}}' after block statement, got {:?}", self.peek()).to_string()
        );

        Statement {
            kind: StatementKind::Block(Box::new(Block {
                statements
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn struct_declaration(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        let name = self.expect(
            TokenType::Identifier,
            format!("Expected struct name, got {:?}", self.peek())
        );

        let mut type_ = Type::new(
            TypeKind::UserType(UserType {
                kind: UserTypeKind::Struct,
                name: name.lexeme.clone(),
            }),
            Modifiers::new(false, false, false, false, false),
            Position {
                start_line: name.line,
                end_line: name.line,
                start_pos: name.pos,
                end_pos: name.pos
            }
        );
        
        if self.match_token(vec![TokenType::Colon]) {
            type_.modifiers = self.type_annotation(true, false).modifiers;
        }

        let fields = self.fields(name.lexeme.clone());

        Statement {
            kind: StatementKind::Struct(Box::new(Struct {
                name: Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                    
                },
                fields,
                methods: vec![],
                type_
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn fields(&mut self, owner: String) -> Vec<Variable> {
        let start = self.tokens[self.current].clone();
        let mut fields = Vec::new();

        self.expect(
            TokenType::LeftBrace,
            format!("Expected '{{' before struct fields, got {:?}", self.peek()).to_string()
        );

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let name = self.expect(
                TokenType::Identifier,
                format!("Expected field name, got {:?}", self.peek()).to_string()
            );

            self.expect(
                TokenType::Colon,
                format!("Expected ':' after field name, got {:?}", self.peek()).to_string()
            );

            let type_ = self.type_annotation(true, true);
            let mut value: Option<Expression> = None;
            if self.match_token(vec![TokenType::Equals]) {
                value = Some(self.expression());
            }

            fields.push(Variable {
                name: Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                },
                value,
                type_: type_.clone(),
                is_field: true,
                owner: Some(owner.clone())
            });

            if !self.match_token(vec![TokenType::Comma]) {
                break;
            }
        }

        self.expect(
            TokenType::RightBrace,
            format!("Expected '}}' after enum variants, got {:?}", self.peek()).to_string()
        );

        fields
    }

    fn enum_declaration(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        let name = self.expect(
            TokenType::Identifier,
            format!("Expected enum name, got {:?}", self.peek()).to_string()
        );

        let mut type_ = Type::new(
            TypeKind::UserType(UserType {
                kind: UserTypeKind::Enum,
                name: name.lexeme.clone()
            }),
            Modifiers::new(false, false, false, false, false),
            Position {
                start_line: name.line,
                end_line: name.line,
                start_pos: name.pos,
                end_pos: name.pos
            }
        );

        if self.match_token(vec![TokenType::Colon]) {
            type_.modifiers = self.type_annotation(true, false).modifiers;
        }

        self.expect(
            TokenType::LeftBrace,
            format!("Expected '{{' before enum variants, got {:?}", self.peek()).to_string()
        );

        let variants = self.enum_variants();

        Statement {
            kind: StatementKind::Enum(Box::new(Enum {
                name: Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                    
                },
                variants,
                methods: vec![],
                type_
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn enum_variants(&mut self) -> Vec<Variable> {
        let mut variants = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let start = self.tokens[self.current].clone();
            let name = self.expect(
                TokenType::Identifier,
                format!("Expected variant name, got {:?}", self.peek()).to_string()
            );

            let mut value: Option<Expression> = None;
            if self.match_token(vec![TokenType::Equals]) {
                value = Some(self.expression());
            }

            variants.push(Variable {
                name: Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                    
                },
                value,
                type_: Type::new(
                    TypeKind::UserType(UserType {
                        kind: UserTypeKind::Enum,
                        name: name.lexeme.clone()
                    }),
                    Modifiers::new(false, true, false, true, false),
                    Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    }
                ),
                is_field: false,
                owner: None
            });

            if !self.match_token(vec![TokenType::Comma]) {
                break;
            }
        }

        self.expect(
            TokenType::RightBrace,
            format!("Expected '}}' after enum variants, got {:?}", self.peek()).to_string()
        );

        variants
    }

    fn variable_declaration(&mut self, is_const: bool) -> Statement {
        let start = self.tokens[self.current].clone();
        let name = self.expect(
            TokenType::Identifier,
            format!("Expected variable name, got {:?}", self.peek()).to_string()
        );

        self.expect(
            TokenType::Colon,
            format!("Expected ':' after variable name, got {:?}", self.peek()).to_string()
        );
        let mut type_ = self.type_annotation(true, true);

        type_.modifiers.is_const = is_const;

        let mut value = None;
        if self.match_token(vec![TokenType::Equals]) {
            value = Some(self.expression());
        }

        let semi = self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after variable declaration, got {:?}", self.peek()).to_string()
        );

        Statement {
            kind: StatementKind::Variable(Box::new(Variable {
                name: Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                    
                },
                value,
                type_,
                is_field: false,
                owner: None
            })),
            pos: Position {
                start_line: start.line,
                end_line: semi.line,
                start_pos: start.pos,
                end_pos: semi.pos
            }
        }
    }

    fn export_block(&mut self) -> Statement {
        let mut identifiers: Vec<Identifier> = vec![];
        if self.match_token(vec![TokenType::RightBrace]) {
            todo!()
        } else {
            todo!()
        }
    }

    fn statement(&mut self) -> Statement {
        if self.match_token(vec![TokenType::LeftBrace]) {
            return self.block_statement();
        }

        if self.match_token(vec![TokenType::If]) {
            return self.if_statement();
        }

        if self.match_token(vec![TokenType::While]) {
            return self.while_statement();
        }

        if self.match_token(vec![TokenType::For]) {
            return self.for_statement();
        }

        if self.match_token(vec![TokenType::Return]) {
            return self.return_statement();
        }

        if self.match_token(vec![TokenType::Break]) {
            return self.break_statement();
        }

        if self.match_token(vec![TokenType::Continue]) {
            return self.continue_statement();
        }

        if self.match_token(vec![TokenType::Var, TokenType::Const]) {
            return self.variable_declaration(self.previous().kind == TokenType::Const);
        }

        let statement = self.expression_statement();

        self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after statement, got {:?}", self.peek()).to_string()
        );

        statement
    }

    fn if_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        self.expect(
            TokenType::LeftParen,
            format!("Expected '(' before if-statement condition, got {:?}", self.peek()).to_string()
        );

        let condition = self.expression();

        self.expect(
            TokenType::RightParen,
            format!("Expected ')' after if-statement condition, got {:?}", self.peek()).to_string()
        );

        let then_branch = self.statement();
        let mut else_branch = None;
        if self.match_token(vec![TokenType::Else]) {
            else_branch = Some(Box::new(self.statement()));
        }

        Statement {
            kind: StatementKind::If(Box::new(If {
                condition,
                then_branch: Box::new(then_branch),
                else_branch
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn while_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        self.expect(
            TokenType::LeftParen,
            format!("Expected '(' before while-statement condition, got {:?}", self.peek()).to_string()
        );

        let condition = self.expression();

        self.expect(
            TokenType::RightParen,
            format!("Expected ')' after while-statement condition, got {:?}", self.peek()).to_string()
        );

        let body = self.statement();

        Statement {
            kind: StatementKind::While(Box::new(While {
                condition,
                body: Box::new(body)
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn for_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        self.expect(
            TokenType::LeftParen,
            format!("Expected '(' before for-statement condition, got {:?}", self.peek()).to_string()
        );
        
        let mut initializer = None;
        let mut condition = None;
        let mut increment = None;

        if !self.match_token(vec![TokenType::Semicolon]) {
            if self.match_token(vec![TokenType::Var]) {
                initializer = Some(Box::new(self.variable_declaration(false)));
            } else {
                initializer = Some(Box::new(self.expression_statement()));
            }

            self.expect(
                TokenType::Semicolon,
                format!("Expected ';' after for-statement initializer, got {:?}", self.peek()).to_string()
            );
        }

        if !self.match_token(vec![TokenType::Semicolon]) {
            condition = Some(self.expression());

            self.expect(
                TokenType::Semicolon,
                format!("Expected ';' after for-statement condition, got {:?}", self.peek()).to_string()
            );
        }

        if !self.match_token(vec![TokenType::RightParen]) {
            increment = Some(self.expression());

            self.expect(
                TokenType::RightParen,
                format!("Expected ')' after for-statement increment, got {:?}", self.peek()).to_string()
            );
        }

        let body = self.statement();

        Statement {
            kind: StatementKind::For(Box::new(For {
                initializer,
                condition,
                increment,
                body: Box::new(body)
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn return_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        let mut value = None;
        if !self.check(TokenType::Semicolon) {
            value = Some(self.expression());
        }

        self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after return statement, got {:?}", self.peek()).to_string()
        );

        Statement {
            kind: StatementKind::Return(Box::new(Return {
                value,
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                }
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn break_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after break statement, got {:?}", self.peek()).to_string()
        );

        let pos = Position {
            start_line: start.line,
            end_line: self.previous().line,
            start_pos: start.pos,
            end_pos: self.previous().pos
        };

        Statement {
            kind: StatementKind::Break(Box::new(Break { pos: pos.clone() })),
            pos
        }
    }

    fn continue_statement(&mut self) -> Statement {
        let start = self.tokens[self.current].clone();

        self.expect(
            TokenType::Semicolon,
            format!("Expected ';' after continue statement, got {:?}", self.peek()).to_string()
        );

        let pos = Position {
            start_line: start.line,
            end_line: self.previous().line,
            start_pos: start.pos,
            end_pos: self.previous().pos
        };

        Statement {
            kind: StatementKind::Continue(Box::new(Continue { pos: pos.clone() })),
            pos
        }
    }

    fn expression_statement(&mut self) -> Statement {
        let expression = self.expression();

        Statement {
            kind: StatementKind::Expression(Box::new(expression.clone())),
            pos: Position {
                start_line: expression.pos.start_line,
                end_line: self.previous().line,
                start_pos: expression.pos.start_pos,
                end_pos: self.previous().pos
            }
        }
    }

    fn expression(&mut self) -> Expression {
        self.assignment()
    }

    fn assignment(&mut self) -> Expression {
        let expression = self.or();

        if self.match_token(vec![
            TokenType::Equals,
            TokenType::PlusEquals,
            TokenType::MinusEquals,
            TokenType::StarEquals,
            TokenType::SlashEquals,
            TokenType::AmpersandEquals,
            TokenType::PipeEquals,
            TokenType::CaretEquals,
        ]) {
            let operator = self.previous();
            let value = self.assignment();

            return Expression {
                kind: ExpressionKind::Assignment(Box::new(Assignment {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(value)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn or(&mut self) -> Expression {
        let mut expression = self.and();

        while self.match_token(vec![TokenType::PipePipe]) {
            let operator = self.previous();
            let right = self.and();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn and(&mut self) -> Expression {
        let mut expression = self.equality();

        while self.match_token(vec![TokenType::AmpersandAmpersand]) {
            let operator = self.previous();
            let right = self.equality();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn equality(&mut self) -> Expression {
        let mut expression = self.comparison();

        while self.match_token(vec![TokenType::EqualsEquals, TokenType::BangEquals]) {
            let operator = self.previous();
            let right = self.comparison();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn comparison(&mut self) -> Expression {
        let mut expression = self.term();

        while self.match_token(vec![
            TokenType::Less, TokenType::LessEquals,
            TokenType::Greater, TokenType::GreaterEquals
        ]) {
            let operator = self.previous();
            let right = self.term();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn term(&mut self) -> Expression {
        let mut expression = self.factor();

        while self.match_token(vec![TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn factor(&mut self) -> Expression {
        let mut expression = self.exponent();

        while self.match_token(vec![TokenType::Star, TokenType::Slash]) {
            let operator = self.previous();
            let right = self.exponent();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn exponent(&mut self) -> Expression {
        let mut expression = self.unary();

        while self.match_token(vec![TokenType::StarStar]) {
            let operator = self.previous();
            let right = self.unary();

            expression = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: operator.clone(),
                    left: Box::new(expression.clone()),
                    right: Box::new(right)
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: expression.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: expression.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn unary(&mut self) -> Expression {
        if self.match_token(vec![TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary();

            return Expression {
                kind: ExpressionKind::Unary(Box::new(Unary {
                    operator: operator.clone(),
                    right: Box::new(right.clone()),
                    is_prefix: true
                })),
                pos: Position {
                    start_line: operator.line,
                    end_line: right.pos.end_line,
                    start_pos: operator.pos,
                    end_pos: right.pos.end_pos
                },
                
            }
        }

        if self.check(TokenType::Identifier) {
            if self.peek_next().kind == TokenType::PlusPlus || self.peek_next().kind == TokenType::MinusMinus {
                let name = self.peek();
                self.advance();
                let operator = self.advance();
                return Expression {
                    kind: ExpressionKind::Unary(Box::new(Unary {
                        operator: operator.clone(),
                        right: Box::new(Expression {
                            kind: ExpressionKind::Identifier(Box::new(Identifier {
                                name: name.clone()
                            })),
                            pos: Position {
                                start_line: name.line,
                                end_line: operator.line,
                                start_pos: name.pos,
                                end_pos: operator.pos
                            },
                            
                        }),
                        is_prefix: false
                    })),
                    pos: Position {
                        start_line: name.line,
                        end_line: operator.line,
                        start_pos: name.pos,
                        end_pos: operator.pos
                    },
                    
                }
            }
        }

        self.cast()
    }

    fn cast(&mut self) -> Expression {
        let mut expression = self.property_access();

        while self.match_token(vec![TokenType::As]) {
            let operator = self.previous();
            let type_ = self.type_annotation(true, true);

            expression = Expression {
                kind: ExpressionKind::Cast(Box::new(Cast {
                    operator: operator.clone(),
                    value: Box::new(expression.clone()),
                    type_: type_.clone()
                })),
                pos: Position {
                    start_line: expression.pos.start_line,
                    end_line: type_.pos.end_line,
                    start_pos: expression.pos.start_pos,
                    end_pos: type_.pos.end_pos
                },
                
            }
        }

        expression
    }

    fn property_access(&mut self) -> Expression {
        let expression = self.call();

        if self.match_token(vec![TokenType::Dot]) {
            return self.get(expression);
        } else if self.match_token(vec![TokenType::LeftBracket]) {
            return self.index(expression);
        }

        expression
    }

    fn call(&mut self) -> Expression {
        let mut expression = self.primary();

        loop {
            if self.match_token(vec![TokenType::LeftParen]) {
                expression = self.finish_call(expression);
            } else {
                break;
            }
        }

        expression
    }

    fn finish_call(&mut self, callee: Expression) -> Expression {
        let mut args = Vec::new();

        if !self.check(TokenType::RightParen) {
            loop {
                if args.len() >= 255 {
                    self.error(
                        self.peek(),
                        format!("Cannot have more than 255 arguments, got {:?}", self.peek()).to_string(),
                        None
                    );
                }

                args.push(self.expression());

                if !self.match_token(vec![TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.expect(
            TokenType::RightParen,
            format!("Expected ')' after arguments, got {:?}", self.peek()).to_string()
        );

        let call_expr = Expression {
            kind: ExpressionKind::Call(Box::new(Call {
                callee: Box::new(callee.clone()),
                args,
            })),
            pos: Position {
                start_line: callee.pos.start_line,
                end_line: paren.line,
                start_pos: callee.pos.end_pos,
                end_pos: paren.pos
            },
            
        };

        if self.match_token(vec![TokenType::Dot]) {
            return self.get(call_expr);
        } else if self.match_token(vec![TokenType::LeftBracket]) {
            return self.index(call_expr);
        }

        call_expr
    }

    fn get(&mut self, object: Expression) -> Expression {
        let start = self.tokens[self.current].clone();
        let mut current_object = object;
        let mut current_name = self.finish_get();

        while self.match_token(vec![TokenType::Dot]) {
            let next_name = self.finish_get();
            current_object = Expression {
                kind: ExpressionKind::Get(Box::new(Get {
                    object: Box::new(current_object),
                    name: Box::new(current_name)
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: next_name.pos.end_line,
                    start_pos: start.pos,
                    end_pos: next_name.pos.end_pos
                },
                
            };

            current_name = next_name;
        }

        if self.match_token(TokenType::assignment_operators()) {
            return self.set(current_object, current_name);
        }
        if self.match_token(vec![TokenType::PlusPlus, TokenType::MinusMinus]) {
            return Expression {
                kind: ExpressionKind::Unary(Box::new(Unary {
                    operator: self.previous(),
                    right: Box::new(Expression {
                        kind: ExpressionKind::Get(Box::new(Get {
                            object: Box::new(current_object),
                            name: Box::new(current_name.clone())
                        })),
                        pos: Position {
                            start_line: start.line,
                            end_line: current_name.pos.end_line,
                            start_pos: start.pos,
                            end_pos: current_name.pos.end_pos
                        },
                        
                    }),
                    is_prefix: false
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                },
                
            };
        }

        Expression {
            kind: ExpressionKind::Get(Box::new(Get {
                object: Box::new(current_object),
                name: Box::new(current_name.clone())
            })),
            pos: Position {
                start_line: start.line,
                end_line: current_name.pos.end_line,
                start_pos: start.pos,
                end_pos: current_name.pos.end_pos
            },
            
        }
    }

    fn finish_get(&mut self) -> Expression {
        let start = self.tokens[self.current].clone();
        if self.match_token(vec![TokenType::Identifier]) {
            let name = self.previous();

            if self.match_token(vec![TokenType::LeftParen]) {
                return self.finish_call(Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: self.previous().line,
                        start_pos: start.pos,
                        end_pos: self.previous().pos
                    },
                    
                });
            }

            if self.match_token(vec![TokenType::LeftBracket]) {
                return self.index(Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: name.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: name.line,
                        start_pos: start.pos,
                        end_pos: name.pos
                    },
                    
                });
            }

            if self.match_token(vec![TokenType::LeftBrace]) {
                return self.struct_init(name);
            }

            return Expression {
                kind: ExpressionKind::Identifier(Box::new(Identifier {
                    name: name.clone()
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: name.line,
                    start_pos: start.pos,
                    end_pos: name.pos
                },
                
            }
        } else {
            self.error(
                self.peek(),
                format!("Expected identifier, got {:?}", self.peek()).to_string(),
                None
            );

            Expression {
                kind: ExpressionKind::Error,
                pos: Position {
                    start_line: start.line,
                    end_line: start.line,
                    start_pos: start.pos,
                    end_pos: start.pos
                },
                
            }
        }
    }

    fn set(&mut self, current_object: Expression, current_name: Expression) -> Expression {
        let start = self.tokens[self.current].clone();
        let value = self.expression();

        Expression {
            kind: ExpressionKind::Set(Box::new(Set {
                object: Box::new(current_object),
                name: Box::new(current_name),
                value: Box::new(value.clone())
            })),
            pos: Position {
                start_line: start.line,
                end_line: value.pos.end_line,
                start_pos: start.pos,
                end_pos: value.pos.end_pos
            },
            
        }
    }

    fn index(&mut self, target: Expression) -> Expression {
        let start = self.tokens[self.current].clone();
        let index = self.finish_index();

        Expression {
            kind: ExpressionKind::Index(Box::new(Index {
                target: Box::new(target),
                index: Box::new(index.clone())
            })),
            pos: Position {
                start_line: start.line,
                end_line: index.pos.end_line,
                start_pos: start.pos,
                end_pos: index.pos.end_pos
            },
            
        }
    }

    fn finish_index(&mut self) -> Expression {
        // e.g:
        //  a[0] => 1st element of a,
        //  a[0, 1] => 2nd element of the 1st element of a. (indexing a 2D array)
        //  ...

        let start = self.tokens[self.current].clone();
        let mut index = self.expression();

        while self.match_token(vec![TokenType::Comma]) {
            let right = self.expression();

            index = Expression {
                kind: ExpressionKind::Binary(Box::new(Binary {
                    operator: Token::new(
                        TokenType::Comma,
                        String::from(","),
                        start.line,
                        start.pos
                    ),
                    left: Box::new(index.clone()),
                    right: Box::new(right.clone())
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: right.pos.end_line,
                    start_pos: start.pos,
                    end_pos: right.pos.end_pos
                },
                
            }
        }

        if self.match_token(vec![TokenType::Dot]) {
            return self.get(index);
        }

        self.expect(
            TokenType::RightBracket,
            format!("Expected ']' after index, got {:?}", self.peek()).to_string()
        );

        index
    }

    fn struct_init(&mut self, t: Token) -> Expression {
        let start = self.tokens[self.current].clone();
        let fields = self.finish_struct_init();

        Expression {
            kind: ExpressionKind::StructInit(Box::new(StructInit {
                name: t,
                fields
            })),
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            },
            
        }
    }

    fn finish_struct_init(&mut self) -> Vec<(Token, Expression)> {
        let mut fields = Vec::new();

        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            let name = self.expect(
                TokenType::Identifier,
                format!("Expected field name, got {:?}", self.peek()).to_string()
            );

            self.expect(
                TokenType::Colon,
                format!("Expected ':' after field name, got {:?}", self.peek()).to_string()
            );

            let value = self.expression();

            fields.push((name.clone(), value));

            if !self.match_token(vec![TokenType::Comma]) {
                break;
            }
        }

        self.expect(
            TokenType::RightBrace,
            format!("Expected '}}' after struct initialization, got {:?}", self.peek()).to_string()
        );

        fields
    }

    fn primary(&mut self) -> Expression {
        let start = self.tokens[self.current].clone();

        if self.match_token(vec![TokenType::Identifier]) {
            let x = self.previous();

            if Type::is_primitive_from_kind(Type::from_string(x.lexeme.as_str())) {
                self.error(
                    x.clone(),
                    format!("Cannot use primitive type as expression, got {:?}", x).to_string(),
                    None
                )
            }

            if self.match_token(vec![TokenType::Dot]) {
                return self.get(Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: x.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: x.line,
                        start_pos: start.pos,
                        end_pos: x.pos
                    },
                    
                });
            }

            if self.match_token(vec![TokenType::LeftBracket]) {
                return self.index(Expression {
                    kind: ExpressionKind::Identifier(Box::new(Identifier {
                        name: x.clone()
                    })),
                    pos: Position {
                        start_line: start.line,
                        end_line: x.line,
                        start_pos: start.pos,
                        end_pos: x.pos
                    },
                    
                });
            }

            if self.match_token(vec![TokenType::LeftBrace]) {
                return self.struct_init(x);
            }

            return Expression {
                kind: ExpressionKind::Identifier(Box::new(Identifier {
                    name: self.previous().clone()
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                },
                
            }
        }

        if self.match_token(vec![TokenType::Integer, TokenType::Float, TokenType::String, TokenType::True, TokenType::False, TokenType::Null]) {
            return Expression {
                kind: ExpressionKind::Literal(Box::new(Literal {
                    value: self.previous().clone()
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                },
                
            }
        }

        if self.match_token(vec![TokenType::LeftParen]) {
            let expression = self.expression();

            self.expect(
                TokenType::RightParen,
                format!("Expected ')' after expression, got {:?}", self.peek()).to_string()
            );

            return Expression {
                kind: ExpressionKind::Grouping(Box::new(Grouping {
                    expression: Box::new(expression)
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                },
                
            }
        }

        if self.match_token(vec![TokenType::LeftBracket]) {
            let mut elements = Vec::new();

            if !self.check(TokenType::RightBracket) {
                loop {
                    if elements.len() >= 255 {
                        self.error(
                            self.peek(),
                            format!("Cannot have more than 255 elements in an array, got {:?}", self.peek()).to_string(),
                            None
                        );
                    }

                    elements.push(self.expression());

                    if !self.match_token(vec![TokenType::Comma]) {
                        break;
                    }
                }
            }

            self.expect(
                TokenType::RightBracket,
                format!("Expected ']' after array elements, got {:?}", self.peek()).to_string()
            );

            return Expression {
                kind: ExpressionKind::Array(Box::new(Array {
                    elements
                })),
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                },
                
            }
        }

        if self.match_token(vec![TokenType::Eof]) {
            return Expression {
                kind: ExpressionKind::Error,
                pos: Position {
                    start_line: start.line,
                    end_line: self.previous().line,
                    start_pos: start.pos,
                    end_pos: self.previous().pos
                },
                
            }
        }

        self.error(
            self.peek(),
            format!("Expected expression, got {:?}", self.peek()).to_string(),
            None
        );

        Expression {
            kind: ExpressionKind::Error,
            pos: Position {
                start_line: start.line,
                end_line: self.previous().line,
                start_pos: start.pos,
                end_pos: self.previous().pos
            },
            
        }
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenType::Eof
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn peek_next(&self) -> Token {
        self.tokens[self.current + 1].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn check(&self, kind: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().kind == kind
    }

    fn match_token(&mut self, kinds: Vec<TokenType>) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn expect(&mut self, kind: TokenType, message: String) -> Token {
        if self.check(kind) {
            return self.advance();
        }
        self.error(self.peek(), message, None);
        self.peek()
    }

    fn error(&mut self, _token: Token, message: String, labels: Option<Vec<Label<usize>>>) {
        if self.panic_mode {
            return;
        }
        self.panic_mode = true;

        let mut error = Error::new(message, String::from("E003"));
        error.build(
            self.filename.clone(),
            self.content.clone(),
            labels.unwrap_or(vec![])
        );
        error.emit();
        self.had_error = true;
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().kind == TokenType::Semicolon {
                return;
            }

            match self.peek().kind {
                TokenType::Func | TokenType::Struct | TokenType::Enum | TokenType::Var | TokenType::If | TokenType::While | TokenType::For | TokenType::Return => {
                    return;
                },
                _ => {}
            }

            self.advance();
        }
    }
}