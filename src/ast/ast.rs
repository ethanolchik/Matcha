// Ethan Olchik
// src/ast/ast.rs
// This file contains the AST definitions for the compiler.

//> Macro rules

/// impl_box! macro<br>
/// This macro is used to implement the Node trait for Box<T> where T is a Node.
macro_rules! impl_box {
    ($type:ty) => {
        impl Node for Box<$type> {
            fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
                self.as_ref().accept(visitor)
            }

            fn get_children(&self) -> Vec<&dyn Node> {
                self.as_ref().get_children()
            }

            fn as_any(&self) -> &dyn Any {
                self
            }
        }
    };
}

//> Imports

use crate::frontend::lexer::token::Token;
use crate::semantic::types::{Type, TypeOption};
use crate::utils::Position;

use dyn_clone;

use std::any::Any;

//> Definitions

/// Node trait
/// This trait is given to all AST nodes and it allows them to be visited by a visitor.
pub trait Node: Any + dyn_clone::DynClone {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption;
    fn get_children(&self) -> Vec<&dyn Node>;

    fn as_any(&self) -> &dyn Any;
}


/// Visitor trait
/// This trait is given to all AST visitors and it allows them to visit AST nodes.
pub trait Visitor {
    fn visit_module(&mut self, module: &Module) -> TypeOption;
    fn visit_statement(&mut self, statement: &Statement) -> TypeOption;
    fn visit_expression(&mut self, expression: &Expression) -> TypeOption;

    fn visit_import(&mut self, import: &Import) -> TypeOption;
    fn visit_variable(&mut self, variable: &Variable) -> TypeOption;
    fn visit_function(&mut self, function: &Function) -> TypeOption;
    fn visit_struct(&mut self, struct_: &Struct) -> TypeOption;
    fn visit_enum(&mut self, enum_: &Enum) -> TypeOption;
    fn visit_return(&mut self, return_: &Return) -> TypeOption;
    fn visit_if(&mut self, if_: &If) -> TypeOption;
    fn visit_while(&mut self, while_: &While) -> TypeOption;
    fn visit_for(&mut self, for_: &For) -> TypeOption;
    fn visit_block(&mut self, block: &Block) -> TypeOption;
    fn visit_export(&mut self, export: &Export) -> TypeOption;
    fn visit_break(&mut self, break_: &Break) -> TypeOption;
    fn visit_continue(&mut self, continue_: &Continue) -> TypeOption;

    fn visit_binary(&mut self, binary: &Binary) -> TypeOption;
    fn visit_unary(&mut self, unary: &Unary) -> TypeOption;
    fn visit_literal(&mut self, literal: &Literal) -> TypeOption;
    fn visit_identifier(&mut self, identifier: &Identifier) -> TypeOption;
    fn visit_call(&mut self, call: &Call) -> TypeOption;
    fn visit_grouping(&mut self, grouping: &Grouping) -> TypeOption;
    fn visit_assignment(&mut self, assignment: &Assignment) -> TypeOption;
    fn visit_array(&mut self, array: &Array) -> TypeOption;
    fn visit_index(&mut self, index: &Index) -> TypeOption;
    fn visit_struct_init(&mut self, struct_instance: &StructInit) -> TypeOption;
    fn visit_get(&mut self, get: &Get) -> TypeOption;
    fn visit_set(&mut self, set: &Set) -> TypeOption;
    fn visit_cast(&mut self, cast: &Cast) -> TypeOption;
    fn visit_type(&mut self, type_: &Type) -> TypeOption;
}

/// Module struct<br>
/// This struct represents the root of the AST.<br>
/// impl Debug, Clone, Node
#[derive(Debug, Clone)]
pub struct Module {
    pub name: Identifier,
    pub filename: String,
    pub statements: Vec<Box<Statement>>,

    pub imports: Vec<Import>
}

/// Statement struct<br>
/// This trait is given to all AST statements.<br>
/// impl Clone, Debug, Node
#[derive(Clone, Debug)]
pub struct Statement {
    pub kind: StatementKind,
    
    pub pos: Position,
}

/// StatementKind enum<br>
/// This enum represents the different kinds of statements in the AST.<br>
/// impl Clone, Debug
#[derive(Clone, Debug)]
pub enum StatementKind {
    Expression(Box<Expression>),
    Import(Box<Import>),
    Variable(Box<Variable>),
    Function(Box<Function>),
    Struct(Box<Struct>),
    Enum(Box<Enum>),
    Return(Box<Return>),
    Break(Box<Break>),
    Continue(Box<Continue>),
    If(Box<If>),
    While(Box<While>),
    For(Box<For>),
    Block(Box<Block>),
    Export(Box<Export>),
}

/// Expression struct<br>
/// This struct represents an expression in the AST.<br>
/// impl Clone, Debug, Node
#[derive(Clone, Debug)]
pub struct Expression {
    pub kind: ExpressionKind,
    
    pub pos: Position,
}

/// ExpressionKind enum<br>
/// This enum represents the different kinds of expressions in the AST.<br>
/// impl Clone, Debug
#[derive(Clone, Debug)]
pub enum ExpressionKind {
    Binary(Box<Binary>),
    Unary(Box<Unary>),
    Literal(Box<Literal>),
    Identifier(Box<Identifier>),
    Call(Box<Call>),
    Grouping(Box<Grouping>),
    Assignment(Box<Assignment>),
    Array(Box<Array>),
    Index(Box<Index>),
    StructInit(Box<StructInit>),
    Get(Box<Get>),
    Set(Box<Set>),
    Cast(Box<Cast>),
    Error,
}

//> Statement Definitions:

#[derive(Clone, Debug)]
pub struct Import {
    pub path: Expression,

    pub alias: Option<String>, // not used for now
}

#[derive(Clone, Debug)]
pub struct Variable {
    pub name: Expression,
    pub value: Option<Expression>, // allows for optional initialization
    pub type_: Type,

    pub is_field: bool,
    pub owner: Option<String>
}

#[derive(Clone, Debug)]
pub struct Function {
    pub name: Expression,
    pub parameters: Vec<Variable>,
    pub type_: Type,
    pub body: Box<Statement>,

    pub is_method: bool,
    pub struct_name: Option<Expression>,
    pub struct_ref_name: Option<Expression>
}

#[derive(Clone, Debug)]
pub struct Struct {
    pub name: Expression,
    pub fields: Vec<Variable>,
    pub methods: Vec<Function>,
    pub type_: Type
}

#[derive(Clone, Debug)]
pub struct Enum {
    pub name: Expression,
    pub variants: Vec<Variable>,

    pub type_: Type,
    pub methods: Vec<Function>
}

#[derive(Clone, Debug)]
pub struct Return {
    pub value: Option<Expression>,
    pub pos: Position
}

#[derive(Clone, Debug)]
pub struct If {
    pub condition: Expression,
    pub then_branch: Box<Statement>,
    pub else_branch: Option<Box<Statement>>
}

#[derive(Clone, Debug)]
pub struct While {
    pub condition: Expression,
    pub body: Box<Statement>
}

#[derive(Clone, Debug)]
pub struct For {
    pub initializer: Option<Box<Statement>>,
    pub condition: Option<Expression>,
    pub increment: Option<Expression>,
    pub body: Box<Statement>
}

#[derive(Clone, Debug)]
pub struct Block {
    pub statements: Vec<Box<Statement>>
}

#[derive(Clone, Debug)]
pub struct Export {
    pub statements: Vec<Box<Statement>>
}

#[derive(Clone, Debug)]
pub struct Break {
    pub pos: Position
}

#[derive(Clone, Debug)]
pub struct Continue {
    pub pos: Position
}

//> Expression Definitions:

#[derive(Clone, Debug)]
pub struct Binary {
    pub left: Box<Expression>,
    pub operator: Token,
    pub right: Box<Expression>,
}

#[derive(Clone, Debug)]
pub struct Unary {
    pub operator: Token,
    pub right: Box<Expression>,

    pub is_prefix: bool
}

#[derive(Clone, Debug)]
pub struct Literal {
    pub value: Token
}

#[derive(Clone, Debug)]
pub struct Identifier {
    pub name: Token
}

#[derive(Clone, Debug)]
pub struct Call {
    pub callee: Box<Expression>,
    pub args: Vec<Expression>
}

#[derive(Clone, Debug)]
pub struct Grouping {
    pub expression: Box<Expression>
}

#[derive(Clone, Debug)]
pub struct Assignment {
    pub operator: Token,
    pub left: Box<Expression>,
    pub right: Box<Expression>
}

#[derive(Clone, Debug)]
pub struct Array {
    pub elements: Vec<Expression>
}

#[derive(Clone, Debug)]
pub struct Index {
    pub target: Box<Expression>,
    pub index: Box<Expression>
}

/// StructInit struct
/// This struct represents an instance of a struct in the AST.
/// [Clone]

#[derive(Clone, Debug)]
pub struct StructInit {
    pub name: Token,
    pub fields: Vec<(Token, Expression)>
}

#[derive(Clone, Debug)]
pub struct Get {
    pub object: Box<Expression>,
    pub name: Box<Expression>
}

#[derive(Clone, Debug)]
pub struct Set {
    pub object: Box<Expression>,
    pub name: Box<Expression>,
    pub value: Box<Expression>
}

#[derive(Clone, Debug)]
pub struct Cast {
    pub operator: Token,
    pub value: Box<Expression>,
    pub type_: Type
}

//> Implementations

impl Node for Module {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_module(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        self.statements.iter().map(|s| s.as_ref() as &dyn Node).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl Node for Statement {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_statement(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        match &self.kind {
            StatementKind::Expression(expression) => vec![expression.as_ref() as &dyn Node],
            StatementKind::Import(import) => vec![import as &dyn Node],
            StatementKind::Variable(variable) => vec![variable as &dyn Node],
            StatementKind::Function(function) => vec![function as &dyn Node],
            StatementKind::Struct(struct_) => vec![struct_ as &dyn Node],
            StatementKind::Enum(enum_) => vec![enum_ as &dyn Node],
            StatementKind::Return(return_) => match &return_.value {
                Some(expression) => vec![expression as &dyn Node],
                None => vec![]
            },
            StatementKind::Export(export) => vec![export as &dyn Node],
            StatementKind::Break(break_) => vec![break_ as &dyn Node],
            StatementKind::Continue(continue_) => vec![continue_ as &dyn Node],
            StatementKind::If(if_) => vec![if_ as &dyn Node],
            StatementKind::While(while_) => vec![while_ as &dyn Node],
            StatementKind::For(for_) => vec![for_ as &dyn Node],
            StatementKind::Block(block) => vec![block as &dyn Node],
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Statement {
    pub fn get_name(&self) -> String {
        match &self.kind {
            StatementKind::Expression(expression) => match &expression.kind {
                ExpressionKind::Identifier(identifier) => identifier.name.lexeme.clone(),
                _ => String::new()
            },
            StatementKind::Variable(variable) => match &variable.name.kind {
                ExpressionKind::Identifier(identifier) => identifier.name.lexeme.clone(),
                _ => String::new()
            },
            StatementKind::Function(function) => match &function.name.kind {
                ExpressionKind::Identifier(identifier) => identifier.name.lexeme.clone(),
                _ => String::new()
            },
            StatementKind::Struct(struct_) => match &struct_.name.kind {
                ExpressionKind::Identifier(identifier) => identifier.name.lexeme.clone(),
                _ => String::new()
            },
            StatementKind::Enum(enum_) => match &enum_.name.kind {
                ExpressionKind::Identifier(identifier) => identifier.name.lexeme.clone(),
                _ => String::new()
            },
            _ => String::new()
        }
    }
}

impl Node for Expression {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_expression(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        match &self.kind {
            ExpressionKind::Binary(binary) => vec![binary as &dyn Node],
            ExpressionKind::Unary(unary) => vec![unary as &dyn Node],
            ExpressionKind::Literal(literal) => vec![literal as &dyn Node],
            ExpressionKind::Identifier(identifier) => vec![identifier as &dyn Node],
            ExpressionKind::Call(call) => vec![call as &dyn Node],
            ExpressionKind::Grouping(grouping) => vec![grouping as &dyn Node],
            ExpressionKind::Assignment(assignment) => vec![assignment as &dyn Node],
            ExpressionKind::Array(array) => vec![array as &dyn Node],
            ExpressionKind::Index(index) => vec![index as &dyn Node],
            ExpressionKind::StructInit(struct_) => vec![struct_ as &dyn Node],
            ExpressionKind::Get(get) => vec![get as &dyn Node],
            ExpressionKind::Set(set) => vec![set as &dyn Node],
            ExpressionKind::Cast(cast) => vec![cast as &dyn Node],
            ExpressionKind::Error => vec![],
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Import {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_import(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.path]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Variable {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_variable(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        match &self.value {
            Some(expression) => vec![&self.name, expression],
            None => vec![&self.name]
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Function {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_function(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.name, &self.body]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Struct {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_struct(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Enum {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_enum(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.name]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Return {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_return(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        match &self.value {
            Some(expression) => vec![expression],
            None => vec![]
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for If {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_if(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        match &self.else_branch {
            Some(else_branch) => vec![&self.condition, &self.then_branch, else_branch],
            None => vec![&self.condition, &self.then_branch]
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for While {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_while(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.condition, &self.body]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for For {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_for(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        match &self.initializer {
            Some(initializer) => vec![initializer, &self.body],
            None => vec![&self.body]
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Block {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_block(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        self.statements.iter().map(|s| s.as_ref() as &dyn Node).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Export {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_export(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        self.statements.iter().map(|s| s.as_ref() as &dyn Node).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Break {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_break(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Continue {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_continue(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Binary {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_binary(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.left, &self.right]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Unary {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_unary(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.right]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Literal {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_literal(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Identifier {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_identifier(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Call {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_call(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.callee]
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Grouping {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_grouping(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.expression]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Assignment {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_assignment(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.left, &self.right]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Array {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_array(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        self.elements.iter().map(|e| e as &dyn Node).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Index {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_index(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.target, &self.index]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for StructInit {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_struct_init(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        self.fields.iter().map(|(_name, expression)| expression as &dyn Node).collect()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Get {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_get(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.object]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Set {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_set(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.object, &self.value]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Cast {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_cast(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![&self.value]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Node for Type {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        visitor.visit_type(self)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        vec![]
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Clone for Box<dyn Node> {
    fn clone(&self) -> Self {
        dyn_clone::clone_box(&**self)
    }
}

impl Node for Box<dyn Node> {
    fn accept(&self, visitor: &mut dyn Visitor) -> TypeOption {
        self.as_ref().accept(visitor)
    }

    fn get_children(&self) -> Vec<&dyn Node> {
        self.as_ref().get_children()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

//> Box<Node> Implementations

impl_box!(Module);
impl_box!(Statement);
impl_box!(Import);
impl_box!(Variable);
impl_box!(Function);
impl_box!(Struct);
impl_box!(Enum);
impl_box!(Return);
impl_box!(If);
impl_box!(While);
impl_box!(For);
impl_box!(Block);
impl_box!(Export);
impl_box!(Break);
impl_box!(Continue);
impl_box!(Binary);
impl_box!(Unary);
impl_box!(Expression);
impl_box!(Literal);
impl_box!(Identifier);
impl_box!(Call);
impl_box!(Grouping);
impl_box!(Assignment);
impl_box!(Array);
impl_box!(Index);
impl_box!(StructInit);
impl_box!(Get);
impl_box!(Set);
impl_box!(Cast);