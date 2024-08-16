pub mod codegen;

use crate::semantic::types::{Type, TypeKind, UserType, UserTypeKind};

pub fn type_to_string(t: &Type, is_return_type: bool) -> String {
    match t.kind {
        TypeKind::Int32 |
        TypeKind::Float32 |
        TypeKind::Int64 |
        TypeKind::Float64 |
        TypeKind::Bool |
        TypeKind::Char |
        TypeKind::String => t.to_string(),
        TypeKind::Array(ref t) => {
            if is_return_type {
                return format!("{}*", type_to_string(t, is_return_type));
            } else {
                return format!("{}", type_to_string(t, is_return_type));
            }
        }
        TypeKind::Void => String::from("void"),
        TypeKind::UserType(ref name) => {
            if name.name.contains(".") {
                "MATCHA__".to_string() + &name.name.replace(".", "__MATCHA__")
            } else {
                "MATCHA__".to_string() + &name.name
            }
        }

        _ => panic!("Invalid type: {:?}", t),
    }
}