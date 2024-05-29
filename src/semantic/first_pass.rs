use crate::{
    semantic::{
        types::*,
        *
    },
    utils::imports::ImportHandler,
    ast::ast::Visitor
};

pub struct FirstPassResolver {
    pub symtable: SymbolTable,
    pub import_handler: ImportHandler,

    pub filename: String
}

impl FirstPassResolver {
    pub fn new(filename: String) -> Self {
        Self {
            symtable: SymbolTable::new(filename.clone()),
            import_handler: ImportHandler::new(filename.clone()),
            filename
        }
    }

    pub fn resolve(&mut self, program: &Module) {
        program.accept(self);

        self.symtable.sort_queue();
    }
}

impl Visitor for FirstPassResolver {
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
            StatementKind::Enum(ref enum_) => enum_.accept(self),
            StatementKind::Variable(ref variable) => variable.accept(self),
            StatementKind::Import(ref import) => import.accept(self),
            _ => TypeOption::None
        }
    }

    fn visit_expression(&mut self, _expression: &Expression) -> TypeOption {
        TypeOption::None
    }

    fn visit_import(&mut self, import: &Import) -> TypeOption {
        let m = self.import_handler.process_import(import.clone());

        self.symtable.define_module(m);

        TypeOption::None
    }

    fn visit_variable(&mut self, variable: &Variable) -> TypeOption {
        self.symtable.decl_queue.push(DeclarationKind::Variable(variable.clone()));

        TypeOption::None
    }

    fn visit_function(&mut self, function: &Function) -> TypeOption {
        if function.is_method {
            self.symtable.decl_queue.push(DeclarationKind::Method(function.clone()));
        } else {
            self.symtable.decl_queue.push(DeclarationKind::Function(function.clone()));
        }

        TypeOption::None
    }

    fn visit_struct(&mut self, struct_: &Struct) -> TypeOption {
        self.symtable.decl_queue.push(DeclarationKind::Struct(struct_.clone()));
        TypeOption::None
    }

    fn visit_enum(&mut self, enum_: &Enum) -> TypeOption {
        self.symtable.decl_queue.push(DeclarationKind::Enum(enum_.clone()));
        TypeOption::None
    }

    fn visit_return(&mut self, _return: &Return) -> TypeOption {
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

    fn visit_type(&mut self, _type: &Type) -> TypeOption {
        TypeOption::None
    }
}