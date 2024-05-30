// Ethan Olchik
// src/semantic/mod.rs
// This module contains definitions and implementations for data structures used in the semantic analysis phase of the compiler.

pub mod types;
pub mod resolver;
pub mod first_pass;

//> Imports

use crate::{
    ast::ast::*,
    semantic::types::Type,
    utils::{
        module::MatchaModule,
        imports::ImportHandler,
        maths::jaro_winkler
    }
};

use self::types::TypeKind;

//> Definitions

/// DeclarationKind enum<br>
/// This enum is used to store the kind of declaration.<br>
#[derive(Debug)]
pub enum DeclarationKind {
    Function(Function),
    Method(Function),
    Variable(Variable),
    Struct(Struct),
    Enum(Enum),
}

/// SymbolTable(filename) struct
/// This struct is used to store the current environment in the program as well as forward declarations.
pub struct SymbolTable {
    pub envs: Vec<Environment>,

    pub import_handler: ImportHandler,

    pub decl_queue: Vec<DeclarationKind>,

    pub filename: String,

    pub exported: Vec<Box<Identifier>>
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
    pub modules: Vec<MatchaModule>,

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

pub struct Typechecker<T>
where T: Node{
    pub queue: Vec<T>,

    pub had_error: bool
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
            modules: Vec::new(),
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
        let mut x = false;
        for symbol in self.types.iter_mut() {
            if symbol.id == id {
                symbol.edit(type_.clone());
                x = true;
                break;
            }
        }

        if x { return }

        if self.parent.is_some() {
            self.parent.clone().unwrap().edit_type(id, type_);
        } else {
            panic!("This should never happen: Cannot find type to edit.")
        }
    }

    pub fn edit_function(&mut self, id: i32, function: Function) {
        let mut x = false;
        for symbol in self.functions.iter_mut() {
            if symbol.id == id {
                symbol.edit(function.clone());
                x = true;
                break;
            }
        }

        if x { return }

        if self.parent.is_some() {
            self.parent.clone().unwrap().edit_function(id, function);
        } else {
            panic!("This should never happen: Cannot find function to edit.")
        }
    }

    pub fn edit_variable(&mut self, id: i32, variable: Variable) {
        let mut x = false;
        for symbol in self.variables.iter_mut() {
            if symbol.id == id {
                symbol.edit(variable.clone());
                x = true;
                break;
            }
        }

        if x { return }

        if self.parent.is_some() {
            self.parent.clone().unwrap().edit_variable(id, variable);
        } else {
            panic!("This should never happen: Cannot find variable to edit.")
        }
    }

    pub fn edit_struct(&mut self, id: i32, struct_: Struct) {
        let mut x = false;
        for symbol in self.structs.iter_mut() {
            if symbol.id == id {
                symbol.edit(struct_.clone());
                x = true;
                break;
            }
        }

        if x { return }

        if self.parent.is_some() {
            self.parent.clone().unwrap().edit_struct(id, struct_);
        } else {
            panic!("This should never happen: Cannot find struct to edit.")
        }
    }

    pub fn edit_enum(&mut self, id: i32, enum_: Enum) {
        let mut x = false;
        for symbol in self.enums.iter_mut() {
            if symbol.id == id {
                symbol.edit(enum_.clone());
                x = true;
                break;
            }
        }

        if x { return }

        if self.parent.is_some() {
            self.parent.clone().unwrap().edit_enum(id, enum_);
        } else {
            panic!("This should never happen: Cannot find enum to edit.")
        }
    }

    pub fn get_type(&self, id: i32) -> Option<Type> {
        for symbol in self.types.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_type(id);
        }

        None
    }

    pub fn get_function(&self, id: i32) -> Option<Function> {
        for symbol in self.functions.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_function(id);
        }

        None
    }

    pub fn get_variable(&self, id: i32) -> Option<Variable> {
        for symbol in self.variables.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_variable(id);
        }

        None
    }

    pub fn get_struct(&self, id: i32) -> Option<Struct> {
        for symbol in self.structs.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_struct(id);
        }

        None
    }

    pub fn get_enum(&self, id: i32) -> Option<Enum> {
        for symbol in self.enums.iter() {
            if symbol.id == id {
                return Some(symbol.get());
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_enum(id);
        }

        None
    }

    pub fn get_struct_by_name(&self, name: String) -> Option<Symbol<Struct>> {
        for symbol in self.structs.iter() {
            let struct_ = symbol.get();
            let struct_name = match &struct_.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if struct_name == name {
                return Some(symbol.clone());
            }
        }
        
        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_struct_by_name(name);
        }

        None
    }

    pub fn get_enum_by_name(&self, name: String) -> Option<Symbol<Enum>> {
        for symbol in self.enums.iter() {
            let enum_ = symbol.get();
            let enum_name = match &enum_.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if enum_name == name {
                return Some(symbol.clone());
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().get_enum_by_name(name);
        }

        None
    }

    pub fn var_exists(&self, variable: &Variable) -> bool {
        for symbol in self.variables.iter() {
            let var = symbol.get();
            let n1 = match var.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            let n2 = match &variable.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if n1 == n2 {
                return true;
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().var_exists(variable);
        }

        false
    }

    pub fn function_exists(&self, function: &Function) -> bool {
        for symbol in self.functions.iter() {
            let f = symbol.get();
            let n1 = match f.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            let n2 = match &function.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if n1 == n2 {
                return true;
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().function_exists(function);
        }

        false
    }

    pub fn struct_exists(&self, struct_: &Struct) -> bool {
        for symbol in self.structs.iter() {
            let s = symbol.get();
            let n1 = match s.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            let n2 = match &struct_.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if n1 == n2 {
                return true;
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().struct_exists(struct_);
        }

        false
    }

    pub fn enum_exists(&self, enum_: &Enum) -> bool {
        for symbol in self.enums.iter() {
            let e = symbol.get();
            let n1 = match e.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            let n2 = match &enum_.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if n1 == n2 {
                return true;
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().enum_exists(enum_);
        }

        false
    }

    pub fn get_type_by_name(&self, name: String) -> Option<Symbol<Type>> {
        for symbol in self.types.iter() {
            let type_ = symbol.get();
            let type_name = type_.to_string();
            if type_name == name {
                return Some(symbol.clone());
            }
        }

        None
    }

    // TODO: Optimise
    pub fn lookup(&self, name: String) -> bool {
        for symbol in self.types.iter() {
            let type_ = symbol.get();
            let type_name = type_.to_string();
            if type_name == name {
                return true;
            }
        }

        for symbol in self.functions.iter() {
            let function = symbol.get();
            let function_name = match function.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if function_name == name {
                return true;
            }
        }

        for symbol in self.variables.iter() {
            let variable = symbol.get();
            let variable_name = match variable.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if variable_name == name {
                return true;
            }
        }

        for symbol in self.structs.iter() {
            let struct_ = symbol.get();
            let struct_name = match struct_.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if struct_name == name {
                return true;
            }
        }

        for symbol in self.enums.iter() {
            let enum_ = symbol.get();
            let enum_name = match enum_.name.kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };
            if enum_name == name {
                return true;
            }
        }

        if self.parent.is_some() {
            return self.parent.as_ref().unwrap().lookup(name);
        }

        false
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
            modules: self.modules.clone(),
            parent: self.parent.clone(),

            filename: self.filename.clone(),

            sid: SymbolIdGen {
                cur: self.sid.cur
            }
        }
    }

}

impl Clone for DeclarationKind {
    fn clone(&self) -> Self {
        match self {
            DeclarationKind::Function(function) => DeclarationKind::Function(function.clone()),
            DeclarationKind::Method(method) => DeclarationKind::Method(method.clone()),
            DeclarationKind::Variable(variable) => DeclarationKind::Variable(variable.clone()),
            DeclarationKind::Struct(struct_) => DeclarationKind::Struct(struct_.clone()),
            DeclarationKind::Enum(enum_) => DeclarationKind::Enum(enum_.clone())
        }
    }
}

impl SymbolTable {
    pub fn new(filename: String) -> Self {
        Self {
            envs: vec![Environment::new(String::from("global"), None, None, filename.clone())],
            import_handler: ImportHandler::new(filename.clone()),
            decl_queue: vec![],
            filename,
            exported: vec![]
        }
    }

    pub fn push(&mut self, name: String) {
        let prev = self.current();
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
        self.envs.pop().unwrap();
    }

    // takes a string in and compares it with all defined symbols and returns the most similar one
    pub fn suggest(&self, s: String) -> Option<String> {
        let mut max = 0.0;
        let mut name = String::new();
        
        for x in self.decl_queue.iter() {
            match x {
                DeclarationKind::Function(function) => {
                    match &function.name.kind {
                        ExpressionKind::Identifier(id) => {
                            let sim = jaro_winkler(s.clone(), id.name.lexeme.clone());
                            if sim > max {
                                max = sim;
                                name = id.name.lexeme.clone();
                            }
                        }
                        _ => unreachable!()
                    }
                }
                DeclarationKind::Method(method) => {
                    match &method.name.kind {
                        ExpressionKind::Identifier(id) => {
                            let sim = jaro_winkler(s.clone(), id.name.lexeme.clone());
                            if sim > max {
                                max = sim;
                                name = id.name.lexeme.clone();
                            }
                        }
                        _ => unreachable!()
                    }
                }
                DeclarationKind::Variable(variable) => {
                    match &variable.name.kind {
                        ExpressionKind::Identifier(id) => {
                            let sim = jaro_winkler(s.clone(), id.name.lexeme.clone());
                            if sim > max {
                                max = sim;
                                name = id.name.lexeme.clone();
                            }
                        }
                        _ => unreachable!()
                    }
                }
                DeclarationKind::Struct(struct_) => {
                    match &struct_.name.kind {
                        ExpressionKind::Identifier(id) => {
                            let sim = jaro_winkler(s.clone(), id.name.lexeme.clone());
                            if sim > max {
                                max = sim;
                                name = id.name.lexeme.clone();
                            }
                        }
                        _ => unreachable!()
                    }
                }
                DeclarationKind::Enum(enum_) => {
                    match &enum_.name.kind {
                        ExpressionKind::Identifier(id) => {
                            let sim = jaro_winkler(s.clone(), id.name.lexeme.clone());
                            if sim > max {
                                max = sim;
                                name = id.name.lexeme.clone();
                            }
                        }
                        _ => unreachable!()
                    }
                }
            }
        }

        if max > 0.75 {
            Some(format!("'{}' ({}% match)", name, (max*100.0) as i32))
        } else {
            None
        }
    }

    pub fn current_mut(&mut self) -> &mut Environment {
        self.envs.last_mut().unwrap()
    }

    pub fn sort_queue(&mut self) {
        let mut queue: Vec<DeclarationKind> = vec![];
        let mut names: Vec<String> = vec![];
        // TODO: Optimise this

        for decl in &self.decl_queue {
            match decl {
                DeclarationKind::Struct(struct_) => {
                    queue.push(decl.clone());

                    match &struct_.name.kind {
                        ExpressionKind::Identifier(id) => {
                            if !names.contains(&id.name.lexeme) {
                                names.push(id.name.lexeme.clone());
                            } else {
                                panic!("Struct {} has the same name as another symbol", id.name.lexeme);
                            }
                        }
                        _ => unreachable!()
                    }
                }
                _ => {}
            }
        }

        for decl in &self.decl_queue {
            match decl {
                DeclarationKind::Enum(enum_) => {
                    queue.push(decl.clone());
                    match &enum_.name.kind {
                        ExpressionKind::Identifier(id) => {
                            if !names.contains(&id.name.lexeme) {
                                names.push(id.name.lexeme.clone());
                            } else {
                                panic!("Enum {} has the same name as another symbol", id.name.lexeme);
                            }
                        }
                        _ => unreachable!()
                    }
                }
                _ => {}
            }
        }
        for decl in &self.decl_queue {
            match decl {
                DeclarationKind::Function(function) => {
                    queue.push(decl.clone());
                    match &function.name.kind {
                        ExpressionKind::Identifier(id) => {
                            if !names.contains(&id.name.lexeme) {
                                names.push(id.name.lexeme.clone());
                            } else {
                                panic!("Function {} has the same name as another symbol", id.name.lexeme);
                            }
                        }
                        _ => unreachable!()
                    }
                }
                _ => {}
            }
        }
        for decl in &self.decl_queue {
            match decl {
                DeclarationKind::Method(method) => {
                    queue.push(decl.clone());
                    match &method.name.kind {
                        ExpressionKind::Identifier(id) => {
                            if !names.contains(&id.name.lexeme) {
                                names.push(id.name.lexeme.clone());
                            } else {
                                panic!("Method {} has the same name as another symbol", id.name.lexeme);
                            }
                        }
                        _ => unreachable!()
                    }
                }
                _ => {}
            }
        }
        for decl in &self.decl_queue {
            match decl {
                DeclarationKind::Variable(variable) => {
                    queue.push(decl.clone());

                    match &variable.name.kind {
                        ExpressionKind::Identifier(id) => {
                            if !names.contains(&id.name.lexeme) {
                                names.push(id.name.lexeme.clone());
                            } else {
                                panic!("Variable {} has the same name as another symbol", id.name.lexeme);
                            }
                        }
                        _ => unreachable!()
                    }
                }
                _ => {}
            }
        }

        self.decl_queue = queue;
    }

    fn current(&self) -> Option<Box<Environment>> {
        Some(Box::new(self.envs.last().unwrap().clone()))
    }

    fn define_module(&mut self, module: MatchaModule) {
        let env = self.envs.last_mut().unwrap();
        env.modules.push(module.clone());
    }

    fn global(&mut self) -> bool {
        self.envs.len() == 1
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

// TODO: Implement typechecker

// Here is an explaination of this function:
// The unify function takes two types as arguments and returns a single type.
// The purpose of this function is to unify two types into a single type.
// This is useful when we have two types that are similar but not exactly the same.
// For example, if we have two types that are both integers, we can unify them into a single integer type.
// Lets say we have an Int32 and Int64 type, we need to figure out which type to choose. Here we will choose Int64 as we do not want to lose data.
fn unify(left: &Type, right: &Type) -> Type {
    todo!()
}