// Ethan Olchik
// src/semantic/resolver.rs
// The resolver is used to resolve names into symbols.

//> Imports

use crate::{
    semantic::{
        types::*,
        *
    },
    utils::{
        module::*,
        imports::ImportHandler
    },
    ast::ast::Visitor
};

//> Definitions

pub struct Resolver {
    pub symtable: SymbolTable,
    pub filename: String,
    pub import_handler: ImportHandler,

    pub had_error: bool
}

//> Implementations

impl Resolver {
    pub fn new(filename: String) -> Self {
        Self {
            symtable: SymbolTable::new(filename.clone()),
            filename: filename.clone(),
            import_handler: ImportHandler::new(filename),
            had_error: false
        }
    }

    pub fn resolve(&mut self, program: &Module) {
        program.accept(self);
    }
}

impl Visitor for Resolver {
    fn visit_module(&mut self, module: &Module) -> types::TypeOption {
        for import in module.imports.clone() {
            self.import_handler.add_import(import);
        }

        TypeOption::None
    }

    fn visit_statement(&mut self, statement: &Statement) -> types::TypeOption {
        todo!()
    }

    fn visit_expression(&mut self, expression: &Expression) -> types::TypeOption {
        todo!()
    }

    fn visit_import(&mut self, import: &Import) -> types::TypeOption {
        todo!()
    }

    fn visit_variable(&mut self, variable: &Variable) -> types::TypeOption {
        todo!()
    }

    fn visit_function(&mut self, function: &Function) -> types::TypeOption {
        todo!()
    }

    fn visit_struct(&mut self, struct_: &Struct) -> types::TypeOption {
        todo!()
    }

    fn visit_enum(&mut self, enum_: &Enum) -> types::TypeOption {
        todo!()
    }

    fn visit_return(&mut self, return_: &Return) -> types::TypeOption {
        todo!()
    }

    fn visit_if(&mut self, if_: &If) -> types::TypeOption {
        todo!()
    }

    fn visit_while(&mut self, while_: &While) -> types::TypeOption {
        todo!()
    }

    fn visit_for(&mut self, for_: &For) -> types::TypeOption {
        todo!()
    }

    fn visit_block(&mut self, block: &Block) -> types::TypeOption {
        todo!()
    }

    fn visit_export(&mut self, export: &Export) -> TypeOption {
        todo!()
    }

    fn visit_break(&mut self, break_: &Break) -> types::TypeOption {
        todo!()
    }

    fn visit_continue(&mut self, continue_: &Continue) -> types::TypeOption {
        todo!()
    }

    fn visit_binary(&mut self, binary: &Binary) -> types::TypeOption {
        todo!()
    }

    fn visit_unary(&mut self, unary: &Unary) -> types::TypeOption {
        todo!()
    }

    fn visit_literal(&mut self, literal: &Literal) -> types::TypeOption {
        todo!()
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> types::TypeOption {
        todo!()
    }

    fn visit_call(&mut self, call: &Call) -> types::TypeOption {
        todo!()
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> types::TypeOption {
        todo!()
    }

    fn visit_assignment(&mut self, assignment: &Assignment) -> types::TypeOption {
        todo!()
    }

    fn visit_array(&mut self, array: &Array) -> types::TypeOption {
        todo!()
    }

    fn visit_index(&mut self, index: &Index) -> types::TypeOption {
        todo!()
    }

    fn visit_struct_init(&mut self, struct_instance: &StructInit) -> types::TypeOption {
        todo!()
    }

    fn visit_get(&mut self, get: &Get) -> types::TypeOption {
        todo!()
    }

    fn visit_set(&mut self, set: &Set) -> types::TypeOption {
        todo!()
    }

    fn visit_cast(&mut self, cast: &Cast) -> types::TypeOption {
        todo!()
    }

    fn visit_type(&mut self, type_: &Type) -> types::TypeOption {
        todo!()
    }
}