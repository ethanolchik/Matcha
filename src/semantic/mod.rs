// Ethan Olchik
// src/semantic/mod.rs
// This module contains definitions and implementations for data structures used in the semantic analysis phase of the compiler.

pub mod types;
pub mod resolver;

//> Imports

use crate::{
    ast::ast::*,
    semantic::types::Type,
    utils::{
        module::MatchaModule,
        imports::ImportHandler
    }
};

//> Definitions


/// SymbolTable(filename) struct
/// This struct is used to store the current environment in the program as well as forward declarations.
pub struct SymbolTable {
    pub envs: Vec<Environment>,

    pub import_handler: ImportHandler,

    pub forward_decl_types: Vec<Type>,
    pub forward_decl_functions: Vec<Function>,
    pub forward_decl_constants: Vec<Variable>,

    pub filename: String
}



/// Environment(name, parent, filename) struct<br>
/// This struct is used to store the current environment in the program.<br>
/// All symbols are stored in the environment.<br>
/// impl Clone
pub struct Environment {
    pub name: String,
    pub module: Option<MatchaModule>,

    pub types: Vec<Symbol<Type>>,
    pub functions: Vec<Symbol<Function>>,
    pub variables: Vec<Symbol<Variable>>,
    pub structs: Vec<Symbol<Struct>>,
    pub enums: Vec<Symbol<Enum>>,

    // Forward Declarations allow the user to reference a type, function, or constant before it is defined.
    pub forward_decl_types: Vec<Type>,
    pub forward_decl_functions: Vec<Function>,
    pub forward_decl_constants: Vec<Variable>,

    pub parent: Option<Box<Environment>>,

    pub filename: String,

    pub sid: SymbolIdGen
}


/// Symbol<T: Node + Clone>(value, s, kind) struct<br>
/// This struct is used to store a symbol in the environment. <br>
/// impl Clone
pub struct Symbol<T> 
where T: Node {
    pub id: i32,
    pub value: T,
    pub kind: SymbolKind
}

/// SymbolIdGen struct<br>
/// This struct is used to generate unique symbol ids.
pub struct SymbolIdGen {
    pub cur: i32
}


/// SymbolKind enum <br>
/// This enum is used to store the kind of symbol.<br>
/// impl Clone PartialEq
#[derive(Clone, PartialEq)]
pub enum SymbolKind {
    Function,
    Method,
    Variable,
    Constant,
    Struct,
    Enum,
    Type
}

//TODO: Add docs
//TODO: Implemenations

//> Implementations

impl Environment {
    pub fn new(name: String, module: Option<MatchaModule>, parent: Option<Box<Environment>>, filename: String) -> Self {
        Self {
            name,
            module,
            types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),

            forward_decl_types: Vec::new(),
            forward_decl_functions: Vec::new(),
            forward_decl_constants: Vec::new(),

            parent,

            filename,

            sid: SymbolIdGen::new()
        }
    }

    pub fn add_type(&mut self, type_: Type) {
        self.types.push(Symbol::new(type_, &mut self.sid, SymbolKind::Type));
    }

    pub fn add_function(&mut self, function: Function) {
        let kind: SymbolKind;

        if function.is_method {
            kind = SymbolKind::Method;
        } else {
            kind = SymbolKind::Function;
        }
        self.functions.push(Symbol::new(function, &mut self.sid, kind));
    }

    pub fn add_variable(&mut self, variable: Variable) {
        let kind: SymbolKind;
        if variable.type_.modifiers.is_const {
            kind = SymbolKind::Constant;
        } else {
            kind = SymbolKind::Variable;
        }
        self.variables.push(Symbol::new(variable, &mut self.sid, kind));
    }

    pub fn add_struct(&mut self, struct_: Struct) {
        self.structs.push(Symbol::new(struct_, &mut self.sid, SymbolKind::Struct));
    }

    pub fn add_enum(&mut self, enum_: Enum) {
        self.enums.push(Symbol::new(enum_, &mut self.sid, SymbolKind::Enum));
    }

    pub fn edit_type(&mut self, id: i32, type_: Type) {
        for symbol in self.types.iter_mut() {
            if symbol.id == id {
                symbol.edit(type_);
                break;
            }
        }
    }

    pub fn edit_function(&mut self, id: i32, function: Function) {
        for symbol in self.functions.iter_mut() {
            if symbol.id == id {
                symbol.edit(function);
                break;
            }
        }
    }

    pub fn edit_variable(&mut self, id: i32, variable: Variable) {
        for symbol in self.variables.iter_mut() {
            if symbol.id == id {
                symbol.edit(variable);
                break;
            }
        }
    }

    pub fn edit_struct(&mut self, id: i32, struct_: Struct) {
        for symbol in self.structs.iter_mut() {
            if symbol.id == id {
                symbol.edit(struct_);
                break;
            }
        }
    }

    pub fn edit_enum(&mut self, id: i32, enum_: Enum) {
        for symbol in self.enums.iter_mut() {
            if symbol.id == id {
                symbol.edit(enum_);
                break;
            }
        }
    }

    pub fn get_type(&self, id: i32) -> Option<Type> {
        for symbol in self.types.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        None
    }

    pub fn get_function(&self, id: i32) -> Option<Function> {
        for symbol in self.functions.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        None
    }

    pub fn get_variable(&self, id: i32) -> Option<Variable> {
        for symbol in self.variables.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        None
    }

    pub fn get_struct(&self, id: i32) -> Option<Struct> {
        for symbol in self.structs.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        None
    }

    pub fn get_enum(&self, id: i32) -> Option<Enum> {
        for symbol in self.enums.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        None
    }
}

impl Clone for Environment {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            module: self.module.clone(),

            types: self.types.clone(),
            functions: self.functions.clone(),
            variables: self.variables.clone(),
            structs: self.structs.clone(),
            enums: self.enums.clone(),

            forward_decl_types: self.forward_decl_types.clone(),
            forward_decl_functions: self.forward_decl_functions.clone(),
            forward_decl_constants: self.forward_decl_constants.clone(),

            parent: self.parent.clone(),

            filename: self.filename.clone(),

            sid: SymbolIdGen {
                cur: self.sid.cur
            }
        }
    }

}

impl SymbolTable {
    pub fn new(filename: String) -> Self {
        Self {
            envs: vec![Environment::new(String::from("global"), None, None, filename.clone())],
            import_handler: ImportHandler::new(filename.clone()),
            forward_decl_types: vec![],
            forward_decl_functions: vec![],
            forward_decl_constants: vec![],
            filename
        }
    }

    pub fn push(&mut self, name: String) {
        let prev = self.prev();
        self.envs.push(
            Environment::new(
                name,
                None,
                prev,
                self.filename.clone()
            )
        );
    }

    pub fn pop(&mut self) {
        // Should never be None as the global environment is always present
        let mut x = self.envs.pop().unwrap();
        self.merge_fd(&mut x);
    }

    fn merge_fd(&mut self, env: &mut Environment) {
        let fd = self.forward_decl_types.clone();
        for type_ in &env.forward_decl_types {
            // TODO: check if type is already in forward_decl_types
            for t in &fd {
                if t != type_ {
                    self.forward_decl_types.push(type_.clone());
                }
            }
        }

        let fd = self.forward_decl_functions.clone();
        for function in &env.forward_decl_functions {
            let fn_name = match &function.name.kind {
                ExpressionKind::Identifier(name) => &name.name.lexeme,
                _ => panic!("This should never happen")
            };
            // TODO: check if type is already in forward_decl_types
            for f in &fd {
                let name = match &f.name.kind {
                    ExpressionKind::Identifier(name) => &name.name.lexeme,
                    _ => panic!("This should never happen")
                };
                if name != fn_name {
                    self.forward_decl_functions.push(function.clone());
                }
            }
        }
    
        let fd = self.forward_decl_constants.clone();
        for consts in &env.forward_decl_constants {
            let const_name = match &consts.name.kind {
                ExpressionKind::Identifier(name) => &name.name.lexeme,
                _ => panic!("This should never happen")
            };
            // TODO: check if type is already in forward_decl_types
            for c in &fd {
                let name = match &c.name.kind {
                    ExpressionKind::Identifier(name) => &name.name.lexeme,
                    _ => panic!("This should never happen")
                };
                if name != const_name {
                    self.forward_decl_constants.push(consts.clone());
                }
            }
        }
    }

    fn prev(&self) -> Option<Box<Environment>> {
        self.envs.last().unwrap().parent.clone()
    }
}

impl<T: Node + Clone> Symbol<T> {
    pub fn new(value: T, s: &mut SymbolIdGen, kind: SymbolKind) -> Self {
        Self {
            id: s.gen(),
            value,
            kind
        }
    }

    pub fn eq(&self, other: &Symbol<T>) -> bool {
        self.id == other.id
    }

    pub fn get(&self) -> T {
        self.value.clone()
    }

    pub fn edit(&mut self, value: T) {
        self.value = value;
    }
}

impl<T: Node + Clone> Clone for Symbol<T> {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            value: self.value.clone(),
            kind: self.kind.clone()
        }
    }
}

impl SymbolIdGen {
    pub fn new() -> Self {
        Self {
            cur: 0
        }
    }

    pub fn gen(&mut self) -> i32 {
        self.cur += 1;
        self.cur
    }
}