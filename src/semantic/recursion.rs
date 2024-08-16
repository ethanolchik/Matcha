// Ethan Olchik
// src/semantic/recursion.rs

use crate::{
    ast::ast::Visitor,
    errors::errors::{Diagnostic, DiagnosticKind},
    semantic::{types::*, *},
    utils::imports::ImportHandler,
};

use std::collections::HashSet;

pub struct RecursionChecker {
    pub symtable: SymbolTable,
    pub import_handler: ImportHandler,

    pub filename: String,

    pub had_error: bool,

    recursion_stack: HashSet<String>,
}

impl RecursionChecker {
    pub fn new(filename: String, symtable: SymbolTable, import_handler: ImportHandler) -> Self {
        Self {
            symtable,
            import_handler,
            filename,
            had_error: false,
            recursion_stack: HashSet::new(),
        }
    }

    pub fn check(&mut self, program: &Module) {
        program.accept(self);
    }

    fn error(&mut self, message: &str, line: usize, col: usize, labels: Option<Vec<String>>) {
        self.had_error = true;

        let error = Diagnostic::new(
            DiagnosticKind::Fatal,
            message.to_string(),
            line,
            col,
            self.filename.clone(),
        );

        error.emit();

        if let Some(l) = labels {
            for i in l {
                let label =
                    Diagnostic::new(DiagnosticKind::Note, i, line, col, self.filename.clone());

                label.emit();
            }
        }
    }

    fn warning(&mut self, message: &str, line: usize, col: usize, labels: Option<Vec<String>>) {
        let warning = Diagnostic::new(
            DiagnosticKind::Warning,
            message.to_string(),
            line,
            col,
            self.filename.clone(),
        );

        warning.emit();

        if let Some(l) = labels {
            for i in l {
                let label =
                    Diagnostic::new(DiagnosticKind::Note, i, line, col, self.filename.clone());

                label.emit();
            }
        }
    }
}

impl Visitor for RecursionChecker {
    fn visit_module(&mut self, module: &Module) -> TypeOption {
        for import in module.imports.clone() {
            import.accept(self);
        }

        for statement in module.statements.clone() {
            statement.accept(self);
        }

        TypeOption::None
    }

    fn visit_statement(&mut self, statement: &Statement) -> TypeOption {
        match statement.kind {
            StatementKind::Function(ref function) => function.accept(self),
            StatementKind::Struct(ref struct_) => struct_.accept(self),
            _ => TypeOption::None,
        }
    }

    fn visit_expression(&mut self, _expression: &Expression) -> TypeOption {
        TypeOption::None
    }

    fn visit_function(&mut self, _function: &Function) -> TypeOption {
        TypeOption::None
    }

    fn visit_struct(&mut self, struct_: &Struct) -> TypeOption {
        let name = match struct_.name.kind {
            ExpressionKind::Identifier(ref id) => id.name.lexeme.clone(),
            _ => unreachable!(),
        };

        if self.recursion_stack.contains(&name) {
            self.error(
                &format!("Struct '{}' is recursively defined", name),
                struct_.name.pos.start_line,
                struct_.name.pos.start_pos,
                Some(vec![format!("{:?}", self.recursion_stack)]),
            );
        }

        self.recursion_stack.insert(name.clone());

        for field in struct_.fields.clone() {
            field.type_.accept(self);
        }

        self.recursion_stack.remove(&name);

        TypeOption::None
    }

    fn visit_enum(&mut self, _enum: &Enum) -> TypeOption {
        TypeOption::None
    }

    fn visit_import(&mut self, _import: &Import) -> TypeOption {
        TypeOption::None
    }

    fn visit_variable(&mut self, _variable: &Variable) -> TypeOption {
        TypeOption::None
    }

    fn visit_return(&mut self, _return_: &Return) -> TypeOption {
        TypeOption::None
    }

    fn visit_if(&mut self, _if: &If) -> TypeOption {
        TypeOption::None
    }

    fn visit_while(&mut self, _while: &While) -> TypeOption {
        TypeOption::None
    }

    fn visit_for(&mut self, _for: &For) -> TypeOption {
        TypeOption::None
    }

    fn visit_block(&mut self, _block: &Block) -> TypeOption {
        TypeOption::None
    }

    fn visit_export(&mut self, _export: &Export) -> TypeOption {
        TypeOption::None
    }

    fn visit_break(&mut self, _break: &Break) -> TypeOption {
        TypeOption::None
    }

    fn visit_continue(&mut self, _continue: &Continue) -> TypeOption {
        TypeOption::None
    }

    fn visit_binary(&mut self, _binary: &Binary) -> TypeOption {
        TypeOption::None
    }

    fn visit_unary(&mut self, _unary: &Unary) -> TypeOption {
        TypeOption::None
    }

    fn visit_literal(&mut self, _literal: &Literal) -> TypeOption {
        TypeOption::None
    }

    fn visit_identifier(&mut self, _identifier: &Identifier) -> TypeOption {
        TypeOption::None
    }

    fn visit_call(&mut self, _call: &Call) -> TypeOption {
        TypeOption::None
    }

    fn visit_grouping(&mut self, _grouping: &Grouping) -> TypeOption {
        TypeOption::None
    }

    fn visit_assignment(&mut self, _assignment: &Assignment) -> TypeOption {
        TypeOption::None
    }

    fn visit_array(&mut self, _array: &Array) -> TypeOption {
        TypeOption::None
    }

    fn visit_index(&mut self, _index: &Index) -> TypeOption {
        TypeOption::None
    }

    fn visit_struct_init(&mut self, _struct_instance: &StructInit) -> TypeOption {
        TypeOption::None
    }

    fn visit_get(&mut self, _get: &Get) -> TypeOption {
        TypeOption::None
    }

    fn visit_set(&mut self, _set: &Set) -> TypeOption {
        TypeOption::None
    }

    fn visit_cast(&mut self, _cast: &Cast) -> TypeOption {
        TypeOption::None
    }

    fn visit_type(&mut self, type_: &Type) -> TypeOption {
        let name = type_.to_string().split(" ").last().unwrap().to_string();
        if self.recursion_stack.contains(&name) {
            self.warning(
                &format!("Recursion between {:?}", self.recursion_stack),
                type_.pos.start_line,
                type_.pos.start_pos,
                None,
            );
            self.error(
                // TODO: Consider changing this to a warning or implement a way for bypassing this.
                &format!("Type '{}' is recursively defined", name),
                type_.pos.start_line,
                type_.pos.start_pos,
                None,
            );
        }

        if let Some(x) = self
            .symtable
            .current()
            .unwrap()
            .get_struct_or_enum_by_name(name.clone())
        {
            self.recursion_stack.insert(name.clone());
            match x.get() {
                SymbolKind::Struct(struct_) => {
                    for field in struct_.fields.clone() {
                        field.type_.accept(self);
                    }
                }
                SymbolKind::Enum(enum_) => {
                    for variant in enum_.variants.clone() {
                        variant.accept(self);
                    }
                }
                _ => unreachable!(),
            }
            self.recursion_stack.remove(&name);
        }
        TypeOption::None
    }
}
