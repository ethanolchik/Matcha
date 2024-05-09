// Ethan Olchik
// src/semantic/types.rs
// This file contains the Type definitions for Matcha, as well as the Modifiers enum.

//> Imports

use crate::{frontend::lexer::token::TokenType, utils::Position};

//> Definitions


/// TypeKindOrNone enum<br>
/// All visitor functions return a TypeKindOrNone, which can either be a TypeKind or None.
pub enum TypeOption {
    TypeKind(TypeKind),
    Type(Type),
    None
}


/// Types enum<br>
/// This enum represents the different types in Matcha.<br>
/// impl Debug, Clone, PartialEq
#[derive(Debug, Clone)]
pub enum TypeKind {
    Int32,
    Int64,
    Float32,
    Float64,
    Bool,
    String,
    Void,
    Struct,
    Enum,
    Array(Box<Type>),
    UserType(UserType),

    Object,
    Error
}

/// Type(kind, modifiers) struct<br>
/// This struct represents a type in Matcha.<br>
/// impl Debug, Clone, PartialEq
#[derive(Debug, Clone)]
pub struct Type {
    pub kind: TypeKind,
    pub modifiers: Modifiers,

    pub pos: Position,
}

/// UserType(name, kind) struct<br>
/// This struct represents a user-defined type in Matcha.<br>
/// impl Debug, Clone, PartialEq
#[derive(Debug, Clone)]
pub struct UserType {
    pub name: String,
    pub kind: UserTypeKind,
}

/// UserTypeKind enum
/// User-defined types can either be structs or enums.
/// [Debug]
#[derive(Debug, Clone)]
pub enum UserTypeKind {
    Struct,
    Enum,

    Unknown // assigned during parsing, resolved during semantic analysis
}

/// Modifiers(is_pub?, is_extern?, is_static?, is_const?) struct<br>
/// This struct represents the modifiers that can be applied to variables, functions fields, etc.<br>
/// impl Debug, Clone
#[derive(Debug, Clone)]
pub struct Modifiers {
    pub is_export: bool, // TODO: Change this to an 'export' block.
    pub is_pub: bool,
    pub is_extern: bool,
    pub is_static: bool,
    pub is_const: bool
}

//> Implementations

impl TypeOption {
    pub fn unwrap_kind(self) -> TypeKind {
        match self {
            TypeOption::TypeKind(t) => t,
            TypeOption::Type(t) => t.kind,
            TypeOption::None => panic!("TypeOption::None cannot be unwrapped")
        }
    }

    pub fn unwrap_type(self) -> Type {
        match self {
            TypeOption::Type(t) => t,
            _ => {
                panic!("Cannot unwrap TypeOption::TypeKind or TypeOption::None to type")
            }
        }
    }

    pub fn clone(&self) -> Self {
        match self {
            TypeOption::TypeKind(t) => TypeOption::TypeKind(t.clone()),
            TypeOption::Type(t) => TypeOption::Type(t.clone()),
            TypeOption::None => TypeOption::None
        }
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}

impl PartialEq for TypeKind {
    fn eq(&self, other: &Self) -> bool {
        match self {
            TypeKind::Int32 => match other {
                TypeKind::Int32 => true,
                _ => false
            },
            TypeKind::Int64 => match other {
                TypeKind::Int64 => true,
                _ => false
            },
            TypeKind::Float32 => match other {
                TypeKind::Float32 => true,
                _ => false
            },
            TypeKind::Float64 => match other {
                TypeKind::Float64 => true,
                _ => false
            },
            TypeKind::Bool => match other {
                TypeKind::Bool => true,
                _ => false
            },
            TypeKind::String => match other {
                TypeKind::String => true,
                _ => false
            },
            TypeKind::Void => match other {
                TypeKind::Void => true,
                _ => false
            },
            TypeKind::Struct => match other {
                TypeKind::Struct => true,
                _ => false
            },
            TypeKind::Enum => match other {
                TypeKind::Enum => true,
                _ => false
            },
            TypeKind::Array(t) => match other {
                TypeKind::Array(o) => t == o,
                _ => false
            },
            TypeKind::UserType(t) => match other {
                TypeKind::UserType(o) => t == o,
                _ => false
            },
            TypeKind::Object => match other {
                TypeKind::Object => true,
                _ => false
            },
            TypeKind::Error => match other {
                TypeKind::Error => true,
                _ => false
            }
        }
    }

}

impl PartialEq for UserType {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.kind == other.kind
    }
}

impl PartialEq for UserTypeKind {
    fn eq(&self, other: &Self) -> bool {
        match self {
            UserTypeKind::Struct => match other {
                UserTypeKind::Struct => true,
                _ => false
            },
            UserTypeKind::Enum => match other {
                UserTypeKind::Enum => true,
                _ => false
            },
            UserTypeKind::Unknown => true
        }
    }
}

impl Type {
    pub fn new(kind: TypeKind, modifiers: Modifiers, pos: Position) -> Self {
        Self {
            kind,
            modifiers,
            pos
        }
    }

    pub fn is_primitive(&self) -> bool {
        match self.kind {
            TypeKind::Int32 | TypeKind::Int64 | TypeKind::Float32 | TypeKind::Float64 | TypeKind::Bool | TypeKind::String | TypeKind::Void => true,
            _ => false
        }
    }

    pub fn is_primitive_from_kind(kind: TypeKind) -> bool {
        match kind {
            TypeKind::Int32 | TypeKind::Int64 | TypeKind::Float32 | TypeKind::Float64 | TypeKind::Bool | TypeKind::String | TypeKind::Void => true,
            _ => false
        }
    }

    pub fn is_primitive_from_string(name: String) -> bool {
        match name.as_str() {
            "Int32" | "Int64" | "Float32" | "Float64" | "Bool" | "String" | "Void" => true,
            _ => false
        }
    }

    pub fn is_user_defined(&self) -> bool {
        match self.kind {
            TypeKind::UserType(_) => true,
            _ => false
        }
    }

    pub fn is_struct(&self) -> bool {
        match self.kind {
            TypeKind::UserType(UserType { kind: UserTypeKind::Struct, .. }) => true,
            _ => false
        }
    }

    pub fn is_enum(&self) -> bool {
        match self.kind {
            TypeKind::UserType(UserType { kind: UserTypeKind::Enum, .. }) => true,
            _ => false
        }
    }

    pub fn is_array(&self) -> bool {
        match self.kind {
            TypeKind::Array(_) => true,
            _ => false
        }
    }

    pub fn get_array_type(&self) -> Option<&Type> {
        match &self.kind {
            TypeKind::Array(t) => Some(t),
            _ => None
        }
    }

    pub fn to_string(&self) -> String {
        match &self.kind {
            TypeKind::Int32 => String::from("Int32"),
            TypeKind::Int64 => String::from("Int64"),
            TypeKind::Float32 => String::from("Float32"),
            TypeKind::Float64 => String::from("Float64"),
            TypeKind::Bool => String::from("Bool"),
            TypeKind::String => String::from("String"),
            TypeKind::Void => String::from("Void"),
            TypeKind::Struct => String::from("Struct"),
            TypeKind::Enum => String::from("Enum"),
            TypeKind::Array(t) => format!("[{}]", t.to_string()),
            TypeKind::UserType(t) => String::from(t.to_string()),
            TypeKind::Object => String::from("Object"),
            TypeKind::Error => String::from("!Error")
        }
    }

    pub fn from_string(s: &str) -> TypeKind {
        match s {
            "Int32" => TypeKind::Int32,
            "Int64" => TypeKind::Int64,
            "Float32" => TypeKind::Float32,
            "Float64" => TypeKind::Float64,
            "Bool" => TypeKind::Bool,
            "String" => TypeKind::String,
            "Void" => TypeKind::Void,
            _ => return TypeKind::UserType(UserType::new(String::from(s), UserTypeKind::Unknown))
        }
    }

    // recursive function to create an array and allow arrays of arrays
    pub fn new_array_type(t: Type, depth: usize) -> Type {
        if depth == 0 {
            return Type::new(
                TypeKind::Array(Box::new(t.clone())),
                Modifiers::new(false, false, false, false, false),
                Position {
                    start_line: t.pos.start_line,
                    start_pos: t.pos.start_pos,
                    end_line: t.pos.end_line,
                    end_pos: t.pos.end_pos
                }
            );
        }

        let mut array_type = t.clone();
        for _ in 0..depth {
            array_type = Type::new(
                TypeKind::Array(Box::new(array_type.clone())),
                Modifiers::new(false, false, false, false, false),
                Position {
                    start_line: array_type.pos.start_line,
                    start_pos: array_type.pos.start_pos,
                    end_line: array_type.pos.end_line,
                    end_pos: array_type.pos.end_pos
                }
            );
        }

        array_type
    }
}

impl TypeKind {
    pub fn to_string(&self) -> String  {
        match self {
            TypeKind::Int32 => String::from("Int32"),
            TypeKind::Int64 => String::from("Int64"),
            TypeKind::Float32 => String::from("Float32"),
            TypeKind::Float64 => String::from("Float64"),
            TypeKind::Bool => String::from("Bool"),
            TypeKind::String => String::from("String"),
            TypeKind::Void => String::from("Void"),
            TypeKind::Struct => String::from("Struct"),
            TypeKind::Enum => String::from("Enum"),
            TypeKind::Array(t) => format!("[{}]", t.to_string()),
            TypeKind::UserType(t) => String::from(t.to_string()),
            TypeKind::Object => String::from("Object"),
            TypeKind::Error => String::from("!Error")
        }
    }

    pub fn is_numeric(&self) -> bool {
        match self {
            TypeKind::Int32 | TypeKind::Int64 | TypeKind::Float32 | TypeKind::Float64 => true,
            _ => false
        }
    }

    pub fn precedence(&self, right: TypeKind) -> TypeKind {
        match (self, right.clone()) {
            (TypeKind::Float64, _) => self.clone(),
            (_, TypeKind::Float64) => right,
            (TypeKind::Int64, TypeKind::Int32) => self.clone(),
            (TypeKind::Int32, TypeKind::Int64) => right,
            (TypeKind::Float32, TypeKind::Int32) => self.clone(),
            (TypeKind::Int32, TypeKind::Float32) => right,
            (TypeKind::Float32, TypeKind::Int64) => TypeKind::Float64,
            (TypeKind::Int64, TypeKind::Float32) => TypeKind::Float64,
            _ => self.clone()
        }
    }

    pub fn castable(self, right: TypeKind) -> bool {
        if self.is_numeric() && right.is_numeric() {
            return true;
        }
        if self.clone() == right {
            return true;
        }
        if right.is_numeric() && self.clone() == TypeKind::Bool {
            return true;
        }
        return false;
    }
}

impl UserType {
    pub fn new(name: String, kind: UserTypeKind) -> Self {
        Self {
            name,
            kind
        }
    }

    pub fn to_string(&self) -> String {
        match self.kind {
            UserTypeKind::Struct => format!("Struct {}", self.name),
            UserTypeKind::Enum => format!("Enum {}", self.name),
            UserTypeKind::Unknown => format!("Unknown {}", self.name)
        }
    }
}


impl Modifiers {
    pub fn new(is_export: bool, is_pub: bool, is_extern: bool, is_static: bool, is_const: bool) -> Self {
        Self {
            is_export,
            is_pub,
            is_extern,
            is_static,
            is_const
        }
    }

    pub fn get_modifiers(&self) -> Vec<String> {
        let mut modifiers = Vec::<String>::new();

        if self.is_export {
            modifiers.push(String::from("export"))
        }
        if self.is_pub {
            modifiers.push(String::from("pub"))
        }

        if self.is_extern {
            modifiers.push(String::from("extern"))
        }

        if self.is_static {
            modifiers.push(String::from("static"))
        }

        if self.is_const {
            modifiers.push(String::from("const"))
        }

        modifiers
    }

    pub fn from_tokentype(&mut self, tt: TokenType) {
        match tt {
            TokenType::Export => self.is_export = true,
            TokenType::Pub => self.is_pub = true,
            TokenType::Extern => self.is_extern = true,
            TokenType::Static => self.is_static = true,
            TokenType::Const => self.is_const = true,
            _ => {}
        }
    }
}