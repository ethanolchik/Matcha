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
        imports::ImportHandler
    },
    errors::errors::Error,
    ast::ast::Visitor
};

//> Definitions

pub struct Resolver {
    pub symtable: SymbolTable,
    pub filename: String,
    pub source: String,
    pub import_handler: ImportHandler,

    pub had_error: bool,

    in_function_body: bool,
    in_loop: bool
}

//> Implementations

// TODO: Typecheck
impl Resolver {
    pub fn new(symtable: SymbolTable, filename: String, source: String) -> Self {
        Self {
            symtable,
            filename: filename.clone(),
            source,
            import_handler: ImportHandler::new(filename),
            had_error: false,
            in_function_body: false,
            in_loop: false
        }
    }

    pub fn resolve(&mut self, program: &Module) {
        program.accept(self);
    }

    fn error(&mut self, message: &str) {
        self.had_error = true;

        let mut error = Error::new(message.to_string(), "E005".to_string());
        error.build(
            self.filename.clone(),
            self.source.clone(),
            vec![]);
            //vec![Label::primary(self.fileid, pos.start_pos..pos.end_pos).with_message(message.to_string())]);

        error.emit();
    }
}

impl Visitor for Resolver {
    fn visit_module(&mut self, module: &Module) -> TypeOption {
        for decl in self.symtable.decl_queue.clone() {
            match decl {
                DeclarationKind::Struct(struct_) => struct_.accept(self),
                DeclarationKind::Enum(enum_) => enum_.accept(self),
                DeclarationKind::Function(func) => func.accept(self),
                DeclarationKind::Variable(var) => var.accept(self),
                DeclarationKind::Method(method) => method.accept(self)
            };
        }

        for s in module.statements.clone() {
            s.accept(self);
        }
        TypeOption::None
    }

    fn visit_statement(&mut self, statement: &Statement) -> TypeOption {
        match statement.kind {
            StatementKind::Expression(ref expr) => expr.accept(self),
            StatementKind::Import(ref import) => import.accept(self),
            StatementKind::Variable(ref var) => var.accept(self),
            StatementKind::Function(ref func) => func.accept(self),
            StatementKind::Struct(ref struct_) => struct_.accept(self),
            StatementKind::Enum(ref enum_) => enum_.accept(self),
            StatementKind::Return(ref return_) => return_.accept(self),
            StatementKind::If(ref if_) => if_.accept(self),
            StatementKind::While(ref while_) => while_.accept(self),
            StatementKind::For(ref for_) => for_.accept(self),
            StatementKind::Block(ref block) => block.accept(self),
            StatementKind::Export(ref export) => export.accept(self),
            StatementKind::Break(ref break_) => break_.accept(self),
            StatementKind::Continue(ref continue_) => continue_.accept(self)
        }
    }

    fn visit_expression(&mut self, expression: &Expression) -> TypeOption {
        match expression.kind {
            ExpressionKind::Binary(ref binary) => binary.accept(self),
            ExpressionKind::Unary(ref unary) => unary.accept(self),
            ExpressionKind::Literal(ref literal) => literal.accept(self),
            ExpressionKind::Identifier(ref identifier) => identifier.accept(self),
            ExpressionKind::Call(ref call) => call.accept(self),
            ExpressionKind::Grouping(ref grouping) => grouping.accept(self),
            ExpressionKind::Assignment(ref assignment) => assignment.accept(self),
            ExpressionKind::Array(ref array) => array.accept(self),
            ExpressionKind::Index(ref index) => index.accept(self),
            ExpressionKind::StructInit(ref struct_instance) => struct_instance.accept(self),
            ExpressionKind::Get(ref get) => get.accept(self),
            ExpressionKind::Set(ref set) => set.accept(self),
            ExpressionKind::Cast(ref cast) => cast.accept(self),
            ExpressionKind::Error => TypeOption::None
        }
    }

    fn visit_import(&mut self, _import: &Import) -> TypeOption {
        TypeOption::None
    }

    fn visit_variable(&mut self, variable: &Variable) -> TypeOption {
        // We can just check whether or not the name already exists since we have already checked
        // That there are no duplicate names in the first pass.
        if self.symtable.current_mut().var_exists(variable) {
            return TypeOption::None;
        }

        if self.symtable.global() {
            if !variable.type_.modifiers.is_const {
                self.error("Cannot define a non-constant variable in the global scope");
            } else if variable.type_.modifiers.is_pub {
                self.error("'pub' modifier can only be used on object attributes/methods");
            }
        }

        let _type = variable.type_.accept(self);

        if variable.value.is_some() {
            variable.value.as_ref().unwrap().accept(self);
        }

        self.symtable.current_mut().add_variable(variable.clone());

        TypeOption::None
    }

    fn visit_function(&mut self, function: &Function) -> TypeOption {
        if self.symtable.current_mut().function_exists(function) {
            return TypeOption::None;
        }

        let name = match &function.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        if function.is_method {
            if !self.symtable.global() {
                self.error("Methods can only be defined in the global scope");
            }
            if function.type_.modifiers.is_extern {
                self.error("Cannot declare an extern method.");
            }

            let obj_type = match &function.obj_ref_name.as_ref().unwrap().kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };

            if let Some(sym) = self.symtable.current_mut().get_struct_by_name(obj_type.clone()) {
                let mut x = sym.get();
                for m in x.methods.clone() {
                    let m_name = match &m.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!()
                    };

                    if m_name == name {
                        self.error(format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str());
                    }
                }

                for f in x.fields.clone() {
                    let f_name = match &f.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!()
                    };

                    if f_name == name {
                        self.error(format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str());
                    }
                }

                x.methods.push(function.clone());

                self.symtable.current_mut().edit_struct(sym.id, x.clone());
            } else if let Some(sym) = self.symtable.current_mut().get_enum_by_name(obj_type.clone()) {
                let mut x = sym.get();
                for m in x.methods.clone() {
                    let m_name = match &m.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!()
                    };

                    if m_name == name {
                        self.error(format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str());
                    }
                }

                for v in x.variants.clone() {
                    let v_name = match &v.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!()
                    };

                    if v_name == name {
                        self.error(format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str());
                    }
                }

                x.methods.push(function.clone());
                self.symtable.current_mut().edit_enum(sym.id, x.clone());
            } else {
                panic!("Object {:?} not found.", obj_type);
            }
        } else {
            if function.type_.modifiers.is_pub {
                self.error("'pub' modifier can only be used on object attributes/methods");
            }
        }

        self.symtable.push(name.clone());

        for param in function.parameters.clone() {
            param.accept(self);
        }

        self.in_function_body = true;
        function.body.accept(self);
        self.in_function_body = false;
        self.symtable.pop();

        self.symtable.current_mut().add_function(function.clone());
        TypeOption::None
    }

    fn visit_struct(&mut self, struct_: &Struct) -> TypeOption {
        if self.symtable.current_mut().struct_exists(struct_) {
            return TypeOption::None;
        }

        if !self.symtable.global() {
            self.error("Structs can only be defined in the global scope");
        }

        if struct_.type_.modifiers.is_const {
            self.error("'const' modifier cannot be used on structs.");
        }
        if struct_.type_.modifiers.is_pub {
            self.error("'pub' modifier can only be used on object attributes/methods");
        }
        if struct_.type_.modifiers.is_extern {
            self.error("Cannot declare an extern struct.");
        }

        let name = match &struct_.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        self.symtable.push(name);

        for field in struct_.fields.clone() {
            field.accept(self);
        }

        // We do not need to go through the methods as they are processed seperately.
        self.symtable.pop();

        self.symtable.current_mut().add_struct(struct_.clone());
        self.symtable.current_mut().add_type(struct_.type_.clone());
        TypeOption::None
    }

    fn visit_enum(&mut self, enum_: &Enum) -> TypeOption {
        if self.symtable.current_mut().enum_exists(enum_) {
            return TypeOption::None;
        }

        if !self.symtable.global() {
            self.error("Enums can only be defined in the global scope");
        }

        if enum_.type_.modifiers.is_const {
            self.error("'const' modifier cannot be used on enums.");
        }
        if enum_.type_.modifiers.is_pub {
            self.error("'pub' modifier can only be used on object attributes/methods");
        }
        if enum_.type_.modifiers.is_extern {
            self.error("Cannot declare an extern enum.");
        }

        let name = match &enum_.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        self.symtable.push(name);

        for variant in enum_.variants.clone() {
            variant.accept(self);
        }

        // We do not need to go through the methods as they are processed seperately.
        self.symtable.pop();

        self.symtable.current_mut().add_enum(enum_.clone());
        self.symtable.current_mut().add_type(enum_.type_.clone());
        TypeOption::None
    }

    fn visit_return(&mut self, return_: &Return) -> TypeOption {
        if self.symtable.global() || !self.in_function_body {
            self.error("Cannot return from the global scope");
        }

        if let Some(value) = &return_.value {
            value.accept(self);
        }

        TypeOption::None
    }

    fn visit_if(&mut self, if_: &If) -> TypeOption {
        if_.condition.accept(self);
        if_.then_branch.accept(self);

        if let Some(else_branch) = &if_.else_branch {
            else_branch.accept(self);
        }

        TypeOption::None
    }

    fn visit_while(&mut self, while_: &While) -> TypeOption {
        while_.condition.accept(self);

        self.in_loop = true;
        while_.body.accept(self);
        self.in_loop = false;

        TypeOption::None
    }

    fn visit_for(&mut self, for_: &For) -> TypeOption {
        if let Some(init) = &for_.initializer {
            init.accept(self);
        }
        if let Some(cond) = &for_.condition {
            cond.accept(self);
        }
        if let Some(inc) = &for_.increment {
            inc.accept(self);
        }

        self.in_loop = true;
        for_.body.accept(self);
        self.in_loop = false;

        TypeOption::None
    }

    fn visit_block(&mut self, block: &Block) -> TypeOption {
        self.symtable.push("block".to_string());

        for s in block.statements.clone() {
            s.accept(self);
        }

        self.symtable.pop();
        TypeOption::None
    }

    fn visit_export(&mut self, export: &Export) -> TypeOption {
        for statement in export.statements.clone() {
            statement.accept(self);
        }

        self.symtable.exported = export.statements.clone();

        TypeOption::None
    }

    fn visit_break(&mut self, _break: &Break) -> TypeOption {
        if !self.in_loop {
            self.error("Cannot break outside of a loop");
        }

        TypeOption::None
    }

    fn visit_continue(&mut self, _continue: &Continue) -> TypeOption {
        if !self.in_loop {
            self.error("Cannot continue outside of a loop");
        }

        TypeOption::None
    }

    fn visit_binary(&mut self, binary: &Binary) -> TypeOption {
        binary.left.accept(self);
        binary.right.accept(self);

        TypeOption::None
    }

    fn visit_unary(&mut self, unary: &Unary) -> TypeOption {
        unary.right.accept(self);

        TypeOption::None
    }

    fn visit_literal(&mut self, _literal: &Literal) -> TypeOption {
        TypeOption::None
    }

    fn visit_identifier(&mut self, identifier: &Identifier) -> TypeOption {
        if self.symtable.current_mut().lookup(identifier.name.lexeme.clone()) {
            return TypeOption::None;
        }

        self.error(format!("Undefined symbol {}", identifier.name.lexeme).as_str());
        TypeOption::None
    }

    fn visit_call(&mut self, call: &Call) -> TypeOption {
        call.callee.accept(self);

        for arg in call.args.clone() {
            arg.accept(self);
        }

        TypeOption::None
    }

    fn visit_grouping(&mut self, grouping: &Grouping) -> TypeOption {
        grouping.expression.accept(self);

        TypeOption::None
    }

    fn visit_assignment(&mut self, assignment: &Assignment) -> TypeOption {
        assignment.left.accept(self);
        assignment.right.accept(self);

        TypeOption::None
    }

    fn visit_array(&mut self, array: &Array) -> TypeOption {
        for elem in array.elements.clone() {
            elem.accept(self);
        }

        TypeOption::None
    }

    fn visit_index(&mut self, index: &Index) -> TypeOption {
        index.target.accept(self);
        index.index.accept(self);

        // TODO: Check if the object has the attribute [RUNTIME]

        TypeOption::None
    }

    fn visit_struct_init(&mut self, struct_instance: &StructInit) -> TypeOption {
        if let Some(_) = self.symtable.current_mut().get_struct_by_name(struct_instance.name.lexeme.clone()) {
            for f in struct_instance.fields.clone() {
                f.1.accept(self);
            }

            return TypeOption::None;
        }

        self.error(format!("Undefined struct {}", struct_instance.name.lexeme).as_str());
        TypeOption::None
    }

    fn visit_get(&mut self, get: &Get) -> TypeOption {
        get.object.accept(self);

        // TODO: Check if the object has the attribute [RUNTIME]

        TypeOption::None
    }

    fn visit_set(&mut self, set: &Set) -> TypeOption {
        set.object.accept(self);
        set.name.accept(self);
        set.value.accept(self);

        // TODO: Check if the object has the attribute [RUNTIME]

        TypeOption::None
    }

    fn visit_cast(&mut self, cast: &Cast) -> TypeOption {
        cast.value.accept(self);
        cast.type_.accept(self);

        TypeOption::None
    }

    fn visit_type(&mut self, _type: &Type) -> TypeOption {
        TypeOption::None
    }
}