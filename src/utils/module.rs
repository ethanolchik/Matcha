// Ethan Olchik
// src/utils/module.rs
// The MatchaModule struct represents a module in Matcha.

//> Imports

use crate::{
    ast::ast::ExpressionKind,
    semantic::{Symbol, SymbolKind},
    utils::maths::*,
};

use std::fmt::{Debug, Formatter};

//> Definitions

#[derive(Clone)]
pub struct MatchaModule {
    pub name: String,
    pub dependencies: Vec<MatchaModule>,

    pub path: String,
    pub exported_symbols: Vec<Symbol>,

    pub all_symbols: Vec<Symbol>,
}

pub struct DependencyTable {
    pub modules: Vec<MatchaModule>,
}

//> Implementations

impl DependencyTable {
    pub fn new() -> Self {
        Self { modules: vec![] }
    }

    pub fn circular_dependencies(&self) -> Option<Vec<MatchaModule>> {
        let mut circular_dependencies = Vec::new();

        for pkg in self.modules.iter() {
            if self.has_circular_dependency(pkg, &mut Vec::new()) {
                circular_dependencies.push(pkg.clone());
            }
        }

        if circular_dependencies.is_empty() {
            None
        } else {
            Some(circular_dependencies)
        }
    }

    pub fn has_circular_dependency(
        &self,
        pkg: &MatchaModule,
        visited: &mut Vec<MatchaModule>,
    ) -> bool {
        if visited.contains(pkg) {
            return true;
        }

        visited.push(pkg.clone());

        for dep in pkg.dependencies.iter() {
            if self.has_circular_dependency(dep, visited) {
                return true;
            }
        }

        visited.pop();

        false
    }

    pub fn push_module(&mut self, module: MatchaModule) {
        self.modules.push(module);

        if let Some(circular_dependencies) = self.circular_dependencies() {
            // TODO: Extend error library to handle this error.
            panic!(
                "Circular dependencies detected: {:?}",
                circular_dependencies
            );
        }
    }
}

impl PartialEq for MatchaModule {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl Debug for MatchaModule {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "MatchaModule {{ name: {}, path: {} }}",
            self.name, self.path
        )
    }
}

impl MatchaModule {
    pub fn suggest(&self, query: String) -> Option<String> {
        let mut best_match = (0.0, None);

        for symbol in self.exported_symbols.iter() {
            let name: String = match &symbol.kind {
                SymbolKind::Function(ref function) => match function.name.kind {
                    ExpressionKind::Identifier(ref ident) => ident.name.lexeme.clone(),
                    _ => unreachable!(),
                },
                SymbolKind::Variable(ref variable) => match variable.name.kind {
                    ExpressionKind::Identifier(ref ident) => ident.name.lexeme.clone(),
                    _ => unreachable!(),
                },
                SymbolKind::Struct(ref struct_) => match struct_.name.kind {
                    ExpressionKind::Identifier(ref ident) => ident.name.lexeme.clone(),
                    _ => unreachable!(),
                },
                SymbolKind::Enum(ref enum_) => match enum_.name.kind {
                    ExpressionKind::Identifier(ref ident) => ident.name.lexeme.clone(),
                    _ => unreachable!(),
                },
                SymbolKind::Constant(ref constant) => match constant.name.kind {
                    ExpressionKind::Identifier(ref ident) => ident.name.lexeme.clone(),
                    _ => unreachable!(),
                },
                _ => unreachable!(),
            };

            let distance = jaro_winkler(name.clone(), query.clone());

            if distance > best_match.0 {
                best_match = (distance, Some(name.clone()));
            }
        }

        if best_match.0 > 0.75 {
            Some(format!(
                "'{}' ({}% match)",
                best_match.1.unwrap(),
                (best_match.0 * 100.0) as i32
            ))
        } else {
            None
        }
    }
}
