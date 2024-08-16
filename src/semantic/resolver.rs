// Ethan Olchik
// src/semantic/resolver.rs
// The resolver is used to resolve names into symbols.

//> Imports

use crate::{
    ast::ast::Visitor, errors::errors::{
        Diagnostic,
        DiagnosticKind
    }, frontend::lexer::token::{Token, TokenType}, semantic::{
        types::*,
        *
    }, utils::imports::ImportHandler
};

//> Definitions

pub struct Resolver {
    pub symtable: SymbolTable,
    pub filename: String,
    pub source: String,
    pub import_handler: ImportHandler,

    pub had_error: bool,

    in_function_body: bool,
    in_loop: bool,

    past_decl_queue_loop: bool
}
//> Implementations

impl Clone for Resolver {
    fn clone(&self) -> Self {
        Self {
            symtable: self.symtable.clone(),
            filename: self.filename.clone(),
            source: self.source.clone(),
            import_handler: self.import_handler.clone(),
            had_error: self.had_error,
            in_function_body: self.in_function_body,
            in_loop: self.in_loop,
            past_decl_queue_loop: self.past_decl_queue_loop
        }
    }
}

// TODO: Typecheck
impl Resolver {
    pub fn new(symtable: SymbolTable, filename: String, source: String, import_handler: ImportHandler) -> Self {
        Self {
            symtable,
            filename: filename.clone(),
            source,
            import_handler,
            had_error: false,
            in_function_body: false,
            in_loop: false,
            past_decl_queue_loop: false
        }
    }

    pub fn resolve(&mut self, program: &Module) {
        program.accept(self);
    }

    fn is_in_std(&self) -> bool {
        self.filename.contains("/home/ethan/Matcha/std/")
    }

    fn error(&mut self, message: &str, line: usize, col: usize, labels: Option<Vec<String>>) {
        self.had_error = true;

        let error = Diagnostic::new(
            DiagnosticKind::Error,
            message.to_string(),
            line,
            col,
            self.filename.clone()
        );
        
        error.emit();

        if let Some(l) = labels {
            for i in l {
                let label = Diagnostic::new(
                    DiagnosticKind::Note,
                    i,
                    line,
                    col,
                    self.filename.clone()
                );

                label.emit();
            }
        }
    }

    fn fatal(&mut self, message: &str, line: usize, col: usize, labels: Option<Vec<String>>) {
        self.had_error = true;

        let error = Diagnostic::new(
            DiagnosticKind::Fatal,
            message.to_string(),
            line,
            col,
            self.filename.clone()
        );
        
        error.emit();

        if let Some(l) = labels {
            for i in l {
                let label = Diagnostic::new(
                    DiagnosticKind::Note,
                    i,
                    line,
                    col,
                    self.filename.clone()
                );

                label.emit();
            }
        }
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

        self.past_decl_queue_loop = true;

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
        if variable.type_.modifiers.is_const {
            if self.past_decl_queue_loop && self.symtable.in_decl_queue(SymbolKind::Constant(variable.clone())) {
                return TypeOption::None;
            }
        } else {
            if self.past_decl_queue_loop && self.symtable.in_decl_queue(SymbolKind::Variable(variable.clone())) {
                return TypeOption::None;
            }
        }

        if self.symtable.global() {
            if !variable.type_.modifiers.is_const {
                self.error(
                    "Cannot define a non-constant variable in the global scope",
                    variable.name.pos.start_line,
                    variable.name.pos.start_pos,
                    None
                );
            } else if variable.type_.modifiers.is_pub {
                self.error(
                    "'pub' modifier can only be used on object attributes/methods",
                    variable.name.pos.start_line,
                    variable.name.pos.start_pos,
                    None
                );
            }
        }

        if variable.type_.modifiers.is_static {
            if variable.type_.modifiers.is_const {
                self.error(
                    "Cannot define a static constant",
                    variable.name.pos.start_line,
                    variable.name.pos.start_pos,
                    None
                );
            }
            if self.symtable.global() || self.symtable.current().unwrap().name == "block" {
                self.error(
                    "Static variables can only be defined in a struct",
                    variable.name.pos.start_line,
                    variable.name.pos.start_pos,
                    None
                );
            }
        }

        let _type = variable.type_.accept(self);

        if variable.value.is_some() {
            variable.value.as_ref().unwrap().accept(self);
        }

        let name = match &variable.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        if self.symtable.current_mut().var_exists_in_current_scope(variable) {
            self.symtable.current_mut().edit_variable(name, variable.clone());
        } else {
            self.symtable.current_mut().add_variable(variable.clone());
        } // TODO: Do this for the rest of the symbol kinds

        TypeOption::None
    }

    fn visit_function(&mut self, function: &Function) -> TypeOption {
        if function.is_method {
            if self.past_decl_queue_loop && self.symtable.in_decl_queue(SymbolKind::Method(function.clone())) {
                return TypeOption::None;
            }
        } else {
            if self.past_decl_queue_loop && self.symtable.in_decl_queue(SymbolKind::Function(function.clone())) {
                return TypeOption::None;
            }
        }

        let name = match &function.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        if function.is_method {
            if !self.symtable.global() {
                self.error(
                    "Methods can only be defined in the global scope",
                    function.name.pos.start_line,
                    function.name.pos.start_pos,
                    None
                );
            }
            if function.type_.modifiers.is_extern {
                self.error(
                    "Cannot declare an extern method.",
                    function.name.pos.start_line,
                    function.name.pos.start_pos,
                    None
                );
            }

            let obj_type = match &function.obj_name.as_ref().unwrap().kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };

            if function.type_.modifiers.is_static {
                if let Some(sym) = self.symtable.current_mut().get_struct_by_name(obj_type.clone()) {
                    let mut x = match sym.get() {
                        SymbolKind::Struct(s) => s,
                        _ => unreachable!()
                    };

                    for m in x.methods.clone() {
                        let m_name = match &m.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };

                        if m_name == name {
                            self.error(
                                format!("Method with name '{}' already exists on {}", name, name).as_str(),
                                function.name.pos.start_line,
                                function.name.pos.start_pos,
                                None
                            );
                        }
                    }

                    for f in x.fields.clone() {
                        let f_name = match &f.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };

                        if f_name == name {
                            self.error(
                                format!("Method with name '{}' already exists on {}", name, name).as_str(),
                                function.name.pos.start_line,
                                function.name.pos.start_pos,
                                None
                            );
                        }
                    }

                    x.methods.push(function.clone());

                    let s_name = match &x.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!()
                    };

                    self.symtable.current_mut().edit_struct(s_name, x.clone());
                } else if let Some(_) = self.symtable.current_mut().get_enum_by_name(name.clone()) {
                    self.error(
                        "Static Methods cannot be defined on enums",
                        function.name.pos.start_line,
                        function.name.pos.start_pos,
                        None
                    );
                }
            } else {
                if function.type_.modifiers.is_builtin {
                    if !self.is_in_std() {
                        self.error(
                            "Builtin methods can only be defined in the standard library",
                            function.name.pos.start_line,
                            function.name.pos.start_pos,
                            None
                        );
                    }

                    if Type::is_primitive_from_string(obj_type.clone()) {
                        if let Some(sym) = self.symtable.current_mut().get_type_by_name(obj_type.clone()) {
                            let mut x = match sym.get() {
                                SymbolKind::Type(t) => t,
                                _ => unreachable!()
                            };
    
                            x.methods.push(function.clone());
    
                            let t_name = x.to_string();
                            self.symtable.current_mut().edit_type(t_name, x.clone());
                        } else {
                            self.fatal(
                                format!("Object {:?} not found.", obj_type).as_str(),
                                function.name.pos.start_line,
                                function.name.pos.start_pos,
                                None
                            );
                        }
                    } else {
                        self.error(
                            "Builtin methods can only be defined on primitive types",
                            function.name.pos.start_line,
                            function.name.pos.start_pos,
                            None
                        );
                    }
                } else {
                    if let Some(sym) = self.symtable.current_mut().get_struct_by_name(obj_type.clone()) {
                        let mut x = match sym.get() {
                            SymbolKind::Struct(s) => s,
                            _ => unreachable!()
                        };
        
                        for m in x.methods.clone() {
                            let m_name = match &m.name.kind {
                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                _ => unreachable!()
                            };
        
                            if m_name == name {
                                self.error(
                                    format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str(),
                                    function.name.pos.start_line,
                                    function.name.pos.start_pos,
                                    None
                                );
                            }
                        }
        
                        for f in x.fields.clone() {
                            let f_name = match &f.name.kind {
                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                _ => unreachable!()
                            };
        
                            if f_name == name {
                                self.error(
                                    format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str(),
                                    function.name.pos.start_line,
                                    function.name.pos.start_pos,
                                    None
                                );
                            }
                        }
        
                        x.methods.push(function.clone());
        
                        let s_name = match &x.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };
                        self.symtable.current_mut().edit_struct(s_name, x.clone());
                    } else if let Some(sym) = self.symtable.current_mut().get_enum_by_name(obj_type.clone()) {
                        let mut x = match sym.get() {
                            SymbolKind::Enum(e) => e,
                            _ => unreachable!()
                        };
        
                        for m in x.methods.clone() {
                            let m_name = match &m.name.kind {
                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                _ => unreachable!()
                            };
        
                            if m_name == name {
                                self.error(
                                    format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str(),
                                    function.name.pos.start_line,
                                    function.name.pos.start_pos,
                                    None
                                );
                            }
                        }
        
                        for v in x.variants.clone() {
                            let v_name = match &v.name.kind {
                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                _ => unreachable!()
                            };
        
                            if v_name == name {
                                self.error(
                                    format!("Attribute with name '{}' already exists on {}", name, obj_type).as_str(),
                                    function.name.pos.start_line,
                                    function.name.pos.start_pos,
                                    None
                                );
                            }
                        }
        
                        x.methods.push(function.clone());
    
                        let e_name = match &x.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };
                        self.symtable.current_mut().edit_enum(e_name, x.clone());
                    } else {
                        self.fatal(
                            format!("Object {:?} not found.", obj_type).as_str(),
                            function.name.pos.start_line,
                            function.name.pos.start_pos,
                            None
                        );
                    }
                }
            }
        } else {
            if function.type_.modifiers.is_pub {
                self.error(
                    "'pub' modifier can only be used on object attributes/methods",
                    function.name.pos.start_line,
                    function.name.pos.start_pos,
                    None
                );
            }
        }

        self.symtable.push(name.clone());

        if function.is_method & !function.type_.modifiers.is_static {
            let obj_name = match function.obj_name.clone().unwrap().kind {
                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                _ => unreachable!()
            };

            let type_ = match self.symtable.current().unwrap().get_type_by_name(obj_name.clone()).unwrap().get() {
                SymbolKind::Type(t) => t,
                _ => unreachable!()
            };

            self.symtable.current_mut().add_variable(Variable {
                name: function.obj_ref_name.clone().unwrap(),
                value: None,
                type_,
                is_field: false,
                owner: None
            });
        }
        for param in function.parameters.clone() {
            param.accept(self);
        }

        self.in_function_body = true;
        function.body.accept(self);
        self.in_function_body = false;
        self.symtable.pop();

        if !function.is_method {
            if self.symtable.current_mut().function_exists_in_current_scope(function) {
                self.symtable.current_mut().edit_function(name, function.clone());
            } else {
                self.symtable.current_mut().add_function(function.clone());
            }
        }

        TypeOption::None
    }

    fn visit_struct(&mut self, struct_: &Struct) -> TypeOption {
        if self.symtable.current_mut().struct_exists(struct_) {
            return TypeOption::None;
        }

        if !self.symtable.global() {
            self.error(
                "Structs can only be defined in the global scope",
                struct_.name.pos.start_line,
                struct_.name.pos.start_pos,
                None
            );
        }

        if struct_.type_.modifiers.is_const {
            self.error(
                "'const' modifier cannot be used on structs.",
                struct_.name.pos.start_line,
                struct_.name.pos.start_pos,
                None
            );
        }
        if struct_.type_.modifiers.is_pub {
            self.error(
                "'pub' modifier can only be used on object attributes/methods",
                struct_.name.pos.start_line,
                struct_.name.pos.start_pos,
                None
            );
        }
        if struct_.type_.modifiers.is_extern {
            self.error(
                "Cannot declare an extern struct.",
                struct_.name.pos.start_line,
                struct_.name.pos.start_pos,
                None
            );
        }

        let name = match &struct_.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        self.symtable.current_mut().add_struct(struct_.clone());
        self.symtable.current_mut().add_type(struct_.type_.clone());

        self.symtable.push(name);

        for field in struct_.fields.clone() {
            field.accept(self);
        }

        // We do not need to go through the methods as they are processed seperately.
        self.symtable.pop();
        TypeOption::None
    }

    fn visit_enum(&mut self, enum_: &Enum) -> TypeOption {
        if self.symtable.current_mut().enum_exists(enum_) {
            return TypeOption::None;
        }

        if !self.symtable.global() {
            self.error(
                "Enums can only be defined in the global scope",
                enum_.name.pos.start_line,
                enum_.name.pos.start_pos,
                None
            );
        }

        if enum_.type_.modifiers.is_const {
            self.error(
                "'const' modifier cannot be used on enums.",
                enum_.name.pos.start_line,
                enum_.name.pos.start_pos,
                None
            );
        }
        if enum_.type_.modifiers.is_pub {
            self.error(
                "'pub' modifier can only be used on object attributes/methods",
                enum_.name.pos.start_line,
                enum_.name.pos.start_pos,
                None
            );
        }
        if enum_.type_.modifiers.is_extern {
            self.error(
                "Cannot declare an extern enum.",
                enum_.name.pos.start_line,
                enum_.name.pos.start_pos,
                None
            );
        }

        let name = match &enum_.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        self.symtable.current_mut().add_enum(enum_.clone());
        self.symtable.current_mut().add_type(enum_.type_.clone());

        self.symtable.push(name);

        for variant in enum_.variants.clone() {
            variant.accept(self);
        }

        // We do not need to go through the methods as they are processed seperately.
        self.symtable.pop();
        TypeOption::None
    }

    fn visit_return(&mut self, return_: &Return) -> TypeOption {
        if self.symtable.global() || !self.in_function_body {
            self.error(
                "Cannot return from the global scope",
                return_.pos.start_line,
                return_.pos.start_pos,
                None
            );
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
            self.error(
                "'break' statement cannot be used outside of a loop.",
                _break.pos.start_line,
                _break.pos.start_pos,
                None
            );
        }

        TypeOption::None
    }

    fn visit_continue(&mut self, _continue: &Continue) -> TypeOption {
        if !self.in_loop {
            self.error(
                "'continue' statement cannot be used outside of a loop.",
                _continue.pos.start_line,
                _continue.pos.start_pos,
                None
            );
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

        let x = self.symtable.suggest(identifier.name.lexeme.clone());

        if x.is_some() {
            self.error(
                format!("Undefined name '{}'", identifier.name.lexeme).as_str(),
                identifier.name.line,
                identifier.name.pos,
                Some(vec![format!("Did you mean {}?", x.unwrap())])
            );
        } else {
            self.error(
                format!("Undefined name '{}'", identifier.name.lexeme).as_str(),
                identifier.name.line,
                identifier.name.pos,
                None
            );
        }

        TypeOption::None
    }

    fn visit_call(&mut self, call: &Call) -> TypeOption {
        let name = match &call.callee.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            _ => unreachable!()
        };

        if let Some(c) = self.symtable.current().unwrap().find(name.clone()) {
            match c.kind {
                SymbolKind::Function(f) => {
                    if f.parameters.len() != call.args.len() {
                        self.error(
                            format!("Expected {} arguments, but got {}.", f.parameters.len(), call.args.len()).as_str(),
                            call.callee.pos.start_line,
                            call.callee.pos.start_pos,
                            None
                        );
                    }
                },
                SymbolKind::Method(m) => {
                    if m.parameters.len() != call.args.len() {
                        self.error(
                            format!("Expected {} arguments, but got {}.", m.parameters.len(), call.args.len()).as_str(),
                            call.callee.pos.start_line,
                            call.callee.pos.start_pos,
                            None
                        );
                    }
                },
                _ => {
                    self.error(
                        format!("'{}' is not a function.", name).as_str(),
                        call.callee.pos.start_line,
                        call.callee.pos.start_pos,
                        None
                    );
                }
            }
        }

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

        // TODO: Check if the object has the attribute

        TypeOption::None
    }

    fn visit_struct_init(&mut self, struct_instance: &StructInit) -> TypeOption {
        if let Some(s) = self.symtable.current_mut().get_struct_by_name(struct_instance.name.lexeme.clone()) {
            let s = match s.get() {
                SymbolKind::Struct(s) => s,
                _ => unreachable!()
            };
            
            if struct_instance.fields.len() == 0 {
                return TypeOption::None;
            }
            if struct_instance.fields.len() != s.fields.len() {
                self.error(
                    format!("Expected {} fields, but got {}.", s.fields.len(), struct_instance.fields.len()).as_str(),
                    struct_instance.name.line,
                    struct_instance.name.pos,
                    None
                );
            }
            for f in struct_instance.fields.clone() {
                f.1.accept(self);
            }

            return TypeOption::None;
        }

        let x = self.symtable.suggest(struct_instance.name.lexeme.clone());

        if x.is_some() {
            self.error(
                format!("Undefined struct '{}'", struct_instance.name.lexeme).as_str(),
                struct_instance.name.line,
                struct_instance.name.pos,
                Some(vec![format!("Did you mean {}?", x.unwrap())])
            );
        } else {
            self.error(
                format!("Undefined struct '{}'", struct_instance.name.lexeme).as_str(),
                struct_instance.name.line,
                struct_instance.name.pos,
                None
            );
        }

        TypeOption::None
    }

    fn visit_get(&mut self, get: &Get) -> TypeOption {
        let object_name = match &get.object.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            ExpressionKind::StructInit(s) => {
                s.name.lexeme.clone()
            }
            ExpressionKind::Call(c) => {
                c.accept(self);
                match &c.callee.kind {
                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                    _ => unreachable!()
                }
            }
            ExpressionKind::Get(g) => {
                let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                get_parts[0].0.clone()
            }
            _ => {
                return TypeOption::None // TODO: check for other expressions.
            }
        };

        let mut is_call: bool = false;

        let get_name = match &get.name.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            ExpressionKind::StructInit(s) => {
                s.name.lexeme.clone()
            }
            ExpressionKind::Call(c) => {
                is_call = true;

                match &c.callee.kind {
                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                    _ => unreachable!()
                }
            }
            ExpressionKind::Get(g) => {
                let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                is_call = get_parts[0].1.clone();
                get_parts[0].0.clone()
            }
            _ => {
                return TypeOption::None // TODO: check for other expressions
            }
        };


        let x = self.symtable.current().unwrap().get_module(object_name.clone());

        if let Some(m) = x {
            let mut found = false;

            for sym in m.exported_symbols.clone() {
                match sym.kind {
                    SymbolKind::Constant(c) => {
                        let name = match &c.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };
                        if name == get_name {
                            found = true;
                        }
                    },
                    SymbolKind::Function(f) => {
                        let name = match &f.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };
                        if name == get_name {
                            found = true;
                        }
                    },
                    SymbolKind::Struct(s) => {
                        let name = match &s.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };
                        if name == get_name {
                            found = true;
                        }
                    },
                    SymbolKind::Enum(e) => {
                        let name = match &e.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!()
                        };
                        if name == get_name {
                            found = true;
                        }
                    },
                    _ => {}
                }
            }

            if !found {
                let x = m.suggest(get_name.clone());

                if x.is_some() {
                    self.fatal(
                        format!("Name '{}' does not exist in module '{}'.", get_name, object_name).as_str(),
                        get.name.pos.start_line,
                        get.name.pos.start_pos,
                        Some(vec![format!("Did you mean {}?", x.unwrap())])
                    );
                } else {
                    self.fatal(
                        format!("Name '{}' does not exist in module '{}'.", get_name, object_name).as_str(),
                        get.name.pos.start_line,
                        get.name.pos.start_pos,
                        None
                    );
                }
            }

            return TypeOption::None;
        }

        let mut last: Option<SymbolKind> = None;

        match &get.object.kind {
            ExpressionKind::Identifier(id) => {
                if !(self.symtable.current_mut().lookup(id.name.lexeme.clone()) || self.symtable.current().unwrap().get_module(id.name.lexeme.clone()).is_some()) {
                    let x = self.symtable.suggest(id.name.lexeme.clone());

                    if x.is_some() {
                        self.error(
                            format!("Undefined name '{}'", id.name.lexeme).as_str(),
                            id.name.line,
                            id.name.pos,
                            Some(vec![format!("Did you mean {}?", x.unwrap())])
                        );
                    } else {
                        self.error(
                            format!("Undefined name '{}'", id.name.lexeme).as_str(),
                            id.name.line,
                            id.name.pos,
                            None
                        );
                    }
                } else {
                    let sym = self.symtable.current().unwrap().find(id.name.lexeme.clone()).unwrap();
                    match sym.kind {
                        SymbolKind::Constant(ref v) | SymbolKind::Variable(ref v) => {
                            let t = v.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = v.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    let mut found = false;
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            if is_call {
                                                for m in &s.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };

                                                    if name == get_name {
                                                        found = true;
                                                        last = Some(SymbolKind::Method(m.clone()));
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for f in &s.fields {
                                                    let name = match &f.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        found = true;
                                                        last = Some(SymbolKind::Variable(f.clone()));
                                                        break;
                                                    }
                                                }
                                            }

                                            // So here, we have already read "get.name". The problem is that later in the code we are checking if get.name exists in get.name
                                            // since thats what we're setting last to be. We need to check if the variable found is the same as get.name.
                                            if !found {
                                                self.fatal(
                                                    format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                    get.name.pos.start_line,
                                                    get.name.pos.start_pos,
                                                    None
                                                );
                                            } else {
                                                if let Some(ref l) = last {
                                                    match l {
                                                        SymbolKind::Variable(v) => {
                                                            let name = match &v.name.kind {
                                                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                                _ => unreachable!()
                                                            };
                                                            if name == get_name {
                                                                return TypeOption::None;
                                                            }
                                                        }
                                                        SymbolKind::Method(m) => {
                                                            let name = match &m.name.kind {
                                                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                                _ => unreachable!()
                                                            };
                                                            if name == get_name {
                                                                return TypeOption::None;
                                                            }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                        SymbolKind::Enum(e) => {
                                            if is_call {
                                                for m in &e.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        found = true;
                                                        last = Some(SymbolKind::Method(m.clone()));
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for v in &e.variants {
                                                    let name = match &v.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        found = true;
                                                        last = Some(SymbolKind::Constant(v.clone()));
                                                        break;
                                                    }
                                                }
                                            }

                                            if !found {
                                                self.fatal(
                                                    format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                    get.name.pos.start_line,
                                                    get.name.pos.start_pos,
                                                    None
                                                );
                                            }
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                            }
                        }
                        SymbolKind::Method(f) => {
                            let t = f.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            if is_call {
                                                for m in &s.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        return TypeOption::None;
                                                    }
                                                }
                                            } else {
                                                for f in &s.fields {
                                                    let name = match &f.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        return TypeOption::None;
                                                    }
                                                }
                                            }

                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            if is_call {
                                                for m in &e.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        return TypeOption::None;
                                                    }
                                                }
                                            } else {
                                                for v in &e.variants {
                                                    let name = match &v.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        return TypeOption::None;
                                                    }
                                                }
                                            }

                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => unreachable!()
                                    }
                                }
                            }
                        }
                        SymbolKind::Struct(s) => {
                            if is_call {
                                for m in &s.methods {
                                    let name = match &m.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    }; // TODO: Fix call checking. Currently, only certain calls will work.
                                        // TODO: Fix access modifiers. We are not checking if the method/attribute is public.
                                        // 
                                    if name == get_name && m.type_.modifiers.is_static { 
                                        return TypeOption::None;
                                    }
                                }
                            } else {
                                for f in &s.fields {
                                    let name = match &f.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name && f.type_.modifiers.is_static {
                                        return TypeOption::None;
                                    }
                                }
                            }

                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                Some(vec![
                                    "Did you forget to initialise the object?".to_string()
                                ])
                            );
                        }
                        SymbolKind::Enum(e) => {
                            if is_call {
                                for m in &e.methods {
                                    let name = match &m.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name {
                                        return TypeOption::None;
                                    }
                                }
                            } else {
                                for v in &e.variants {
                                    let name = match &v.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name {
                                        return TypeOption::None;
                                    }
                                }
                            }

                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Type(t) => {
                            // TODO: Static methods
                            if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                return TypeOption::None;
                            } else {
                                self.fatal(
                                    format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                    get.name.pos.start_line,
                                    get.name.pos.start_pos,
                                    None
                                );
                            }
                        }

                        _ => unreachable!()
                    }
                }
            }
            ExpressionKind::Get(g) => {
                let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                let mut obj = self.symtable.current().unwrap().find(get_parts[0].0.clone()).unwrap().kind;

                for i in 0..get_parts.len() {
                    let get_name: String;
                    let is_call: bool;
                    if i == get_parts.len() - 1 {
                        break;
                    } else {
                        get_name = get_parts[i + 1].0.clone();
                        is_call = get_parts[i + 1].1;
                    }

                    match obj.clone() {
                        SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
                            let t = c.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    obj = t.get_name(get_name.clone());
                                    break;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    let mut found = false;
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            if is_call {
                                                for m in &s.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Method(m.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for f in &s.fields {
                                                    let name = match &f.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Variable(f.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            }

                                            if found { continue }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            if is_call {
                                                for m in &e.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Method(m.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for v in &e.variants {
                                                    let name = match &v.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Constant(v.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            }

                                            if found { continue }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => unreachable!()
                                    }
                                }
                            }
                        }
                        SymbolKind::Method(f) => {
                            let t = f.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    obj = t.get_name(get_name.clone());
                                    break;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    let mut found = false;
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            if is_call {
                                                for m in &s.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Method(m.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for f in &s.fields {
                                                    let name = match &f.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Variable(f.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            }

                                            if found { continue }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            if is_call {
                                                for m in &e.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Method(m.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for v in &e.variants {
                                                    let name = match &v.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Constant(v.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            }

                                            if found { continue }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => unreachable!()
                                    }
                                }
                            }
                        }
                        SymbolKind::Struct(s) => {
                            let mut found = false;

                            if is_call {
                                for m in &s.methods {
                                    let name = match &m.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name && m.type_.modifiers.is_static {
                                        obj = SymbolKind::Method(m.clone());
                                        found = true;
                                        break;
                                    }
                                }
                            } else {
                                for f in &s.fields {
                                    let name = match &f.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name && f.type_.modifiers.is_static {
                                        obj = SymbolKind::Variable(f.clone());
                                        found = true;
                                        break;
                                    }
                                }
                            }

                            if found { continue }
                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Enum(e) => {
                            let mut found = false;
                            if is_call {
                                for m in &e.methods {
                                    let name = match &m.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name {
                                        obj = SymbolKind::Method(m.clone());
                                        found = true;
                                        break;
                                    }
                                }
                            } else {
                                for v in &e.variants {
                                    let name = match &v.name.kind {
                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                        _ => unreachable!()
                                    };
                                    if name == get_name {
                                        obj = SymbolKind::Constant(v.clone());
                                        found = true;
                                        break;
                                    }
                                }
                            }

                            if found { continue }
                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Type(t) => {
                            // TODO: implement static methods
                            if t.is_primitive() {
                                if t.contains_name(get_name.clone()) {
                                    obj = t.get_name(get_name.clone());
                                } else {
                                    self.error(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = t.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());
                                if x.is_some() {
                                    let t_ = x.unwrap();
                                    let mut found = false;

                                    match t_.kind {
                                        SymbolKind::Struct(s) => {
                                            if is_call {
                                                for m in &s.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Method(m.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for f in &s.fields {
                                                    let name = match &f.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Variable(f.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            }

                                            if found { continue }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            if is_call {
                                                for m in &e.methods {
                                                    let name = match &m.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Method(m.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            } else {
                                                for v in &e.variants {
                                                    let name = match &v.name.kind {
                                                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                        _ => unreachable!()
                                                    };
                                                    if name == get_name {
                                                        obj = SymbolKind::Constant(v.clone());
                                                        found = true;
                                                        break;
                                                    }
                                                }
                                            }

                                            if found { continue }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => unreachable!()
                                    }
                                }
                            }
                        }
                        _ => { unreachable!() }
                    }
                }

                last = Some(obj);
            }
            ExpressionKind::Call(c) => {
                c.accept(self);
            }
            ExpressionKind::StructInit(s) => {
                s.accept(self);
            }
            _ => {}
        }

        // TODO: Implement StructInit
        // TODO: Static Attributes
        // TODO: Index, Set.
        // Last = p, next it should go to a call and then another get.
        if last.is_some() {
            match get.name.kind.clone() {
                ExpressionKind::Identifier(id) => {
                    let get_name = id.name.lexeme.clone();
                    match &last.unwrap() {
                        SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
                            let t = c.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    let found = false;
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            for m in &s.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                                            for f in &s.fields {
                                                let name = match &f.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }

                                            if found { return TypeOption::None }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            for m in &e.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                                            for v in &e.variants {
                                                let name = match &v.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                            }
                        }
                        SymbolKind::Method(f) => {
                            let t = f.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    let found = false;
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            for m in &s.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                                            for f in &s.fields {
                                                let name = match &f.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }

                                            if found { return TypeOption::None }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            for m in &e.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                                            for v in &e.variants {
                                                let name = match &v.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }

                                            if found { return TypeOption::None }
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => unreachable!()
                                    }
                                }
                            }
                        }
                        SymbolKind::Struct(s) => {
                            for m in &s.methods {
                                let name = match &m.name.kind {
                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                    _ => unreachable!()
                                };
                                if name == get_name && m.type_.modifiers.is_static {
                                    return TypeOption::None;
                                }
                            }
                            for f in &s.fields {
                                let name = match &f.name.kind {
                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                    _ => unreachable!()
                                };
                                if name == get_name && f.type_.modifiers.is_static {
                                    return TypeOption::None;
                                }
                            }

                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Enum(e) => {
                            let found = false;
                            for v in &e.variants {
                                let name = match &v.name.kind {
                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                    _ => unreachable!()
                                };
                                if name == get_name {
                                    return TypeOption::None;
                                }
                            }
                            for m in &e.methods {
                                let name = match &m.name.kind {
                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                    _ => unreachable!()
                                };
                                if name == get_name {
                                    return TypeOption::None;
                                }
                            }

                            if found { return TypeOption::None }
                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Type(t) => {
                            if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                return TypeOption::None;
                            } else {
                                self.fatal(
                                    format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                    get.name.pos.start_line,
                                    get.name.pos.start_pos,
                                    None
                                );
                            }
                        }
                        _ => unreachable!()
                    }
                }
                ExpressionKind::Call(c) => {
                    c.accept(self);
                    let get_name = match c.callee.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!()
                    };

                    match &last.unwrap() {
                        SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
                            let t = c.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            for m in &s.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }

                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            for m in &e.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                            
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                            }
                        }
                        SymbolKind::Method(f) => {
                            let t = f.type_.clone();
                            if t.is_primitive() {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            } else {
                                let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                if x.is_some() {
                                    let t = x.unwrap();
                                    match t.kind {
                                        SymbolKind::Struct(s) => {
                                            for m in &s.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }

                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        SymbolKind::Enum(e) => {
                                            for m in &e.methods {
                                                let name = match &m.name.kind {
                                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                    _ => unreachable!()
                                                };
                                                if name == get_name {
                                                    return TypeOption::None;
                                                }
                                            }
                            
                                            self.fatal(
                                                format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                get.name.pos.start_line,
                                                get.name.pos.start_pos,
                                                None
                                            );
                                        }
                                        _ => {
                                            unreachable!()
                                        }
                                    }
                                }
                            }
                        }
                        SymbolKind::Struct(s) => {
                            for m in &s.methods {
                                let name = match &m.name.kind {
                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                    _ => unreachable!()
                                };
                                if name == get_name && m.type_.modifiers.is_static {
                                    return TypeOption::None;
                                }
                            }

                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Enum(e) => {
                            for m in &e.methods {
                                let name = match &m.name.kind {
                                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                    _ => unreachable!()
                                };
                                if name == get_name {
                                    return TypeOption::None;
                                }
                            }

                            self.fatal(
                                format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                get.name.pos.start_line,
                                get.name.pos.start_pos,
                                None
                            );
                        }
                        SymbolKind::Type(t) => {
                            if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                return TypeOption::None;
                            } else {
                                self.fatal(
                                    format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                    get.name.pos.start_line,
                                    get.name.pos.start_pos,
                                    None
                                );
                            }
                        }
                        _ => unreachable!()
                    }
                }
                ExpressionKind::Get(g) => {
                    let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                    let mut obj = last.unwrap();

                    for i in 0..get_parts.len() {
                        let get_name: String;
                        let is_call: bool;

                        // if i == get_parts.len() - 1 || i == 0 {
                        //     println!("i: {}", i);
                        //     println!("get_parts: {:?}", get_parts);
                        //     println!("get_parts[i]: {:?}", get_parts[i]);
                        //     println!("{:?}", obj);
                        //     get_name = get_parts[i].0.clone();
                        //     is_call = get_parts[i].1;
                        // } else if i > 0 {
                        //     println!("i: {}", i);
                        //     println!("get_parts: {:?}", get_parts);
                        //     println!("get_parts[i+1]: {:?}", get_parts[i+1]);
                        //     println!("{:?}", obj);
                        //     get_name = get_parts[i + 1].0.clone();
                        //     is_call = get_parts[i + 1].1;
                        // } else {
                        //     get_name = get_parts[i].0.clone();
                        //     is_call = get_parts[i].1;
                        // }

                        get_name = get_parts[i].0.clone();
                        is_call = get_parts[i].1;

                        match obj.clone() {
                            SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
                                let t = c.type_.clone();
                                if t.is_primitive() {
                                    if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                        // obj = t.get_name(get_name.clone());
                                        break;
                                    } else {
                                        self.fatal(
                                            format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                            get.name.pos.start_line,
                                            get.name.pos.start_pos,
                                            None
                                        );
                                    }
                                } else {
                                    let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                    let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                    if x.is_some() {
                                        let t = x.unwrap();
                                        let mut found = false;
                                        match t.kind {
                                            SymbolKind::Struct(s) => {
                                                if is_call {
                                                    for m in &s.methods {
                                                        let name = match &m.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Method(m.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    for f in &s.fields {
                                                        let name = match &f.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Variable(f.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                }

                                                if found { continue }
                                                self.fatal(
                                                    format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                    get.name.pos.start_line,
                                                    get.name.pos.start_pos,
                                                    None
                                                );
                                            }
                                            SymbolKind::Enum(e) => {
                                                if is_call {
                                                    for m in &e.methods {
                                                        let name = match &m.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Method(m.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    for v in &e.variants {
                                                        let name = match &v.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Constant(v.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                }

                                                if found { continue }
                                                self.fatal(
                                                    format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                    get.name.pos.start_line,
                                                    get.name.pos.start_pos,
                                                    None
                                                );
                                            }
                                            _ => unreachable!()
                                        }
                                    }
                                }
                            }
                            SymbolKind::Method(f) => {
                                let t = f.type_.clone();
                                if t.is_primitive() {
                                    if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                        // obj = t.get_name(get_name.clone());
                                        break;
                                    } else {
                                        self.fatal(
                                            format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                            get.name.pos.start_line,
                                            get.name.pos.start_pos,
                                            None
                                        );
                                    }
                                } else {
                                    let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
                                    let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

                                    if x.is_some() {
                                        let t = x.unwrap();
                                        let mut found = false;
                                        match t.kind {
                                            SymbolKind::Struct(s) => {
                                                if is_call {
                                                    for m in &s.methods {
                                                        let name = match &m.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Method(m.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    for f in &s.fields {
                                                        let name = match &f.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Variable(f.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                }

                                                if found { continue }
                                                self.fatal(
                                                    format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                    get.name.pos.start_line,
                                                    get.name.pos.start_pos,
                                                    None
                                                );
                                            }
                                            SymbolKind::Enum(e) => {
                                                if is_call {
                                                    for m in &e.methods {
                                                        let name = match &m.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Method(m.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                } else {
                                                    for v in &e.variants {
                                                        let name = match &v.name.kind {
                                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                                            _ => unreachable!()
                                                        };
                                                        if name == get_name {
                                                            obj = SymbolKind::Constant(v.clone());
                                                            found = true;
                                                            break;
                                                        }
                                                    }
                                                }

                                                if found { continue }
                                                self.fatal(
                                                    format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
                                                    get.name.pos.start_line,
                                                    get.name.pos.start_pos,
                                                    None
                                                );
                                            }
                                            _ => unreachable!()
                                        }
                                    }
                                }
                            }
                            SymbolKind::Struct(s) => {
                                let mut found = false;

                                if is_call {
                                    for m in &s.methods {
                                        let name = match &m.name.kind {
                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                            _ => unreachable!()
                                        };
                                        if name == get_name && m.type_.modifiers.is_static {
                                            obj = SymbolKind::Method(m.clone());
                                            found = true;
                                            break;
                                        }
                                    }
                                } else {
                                    for f in &s.fields {
                                        let name = match &f.name.kind {
                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                            _ => unreachable!()
                                        };
                                        if name == get_name && f.type_.modifiers.is_static {
                                            obj = SymbolKind::Variable(f.clone());
                                            found = true;
                                            break;
                                        }
                                    }
                                }

                                if found { continue }
                                self.fatal(
                                    format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                    get.name.pos.start_line,
                                    get.name.pos.start_pos,
                                    None
                                );
                            }
                            SymbolKind::Enum(e) => {
                                let mut found = false;
                                if is_call {
                                    for m in &e.methods {
                                        let name = match &m.name.kind {
                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                            _ => unreachable!()
                                        };
                                        if name == get_name {
                                            obj = SymbolKind::Method(m.clone());
                                            found = true;
                                            break;
                                        }
                                    }
                                } else {
                                    for v in &e.variants {
                                        let name = match &v.name.kind {
                                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                            _ => unreachable!()
                                        };
                                        if name == get_name {
                                            obj = SymbolKind::Constant(v.clone());
                                            found = true;
                                            break;
                                        }
                                    }

                                    if found { continue }
                                    self.fatal(
                                        format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }

                                if found { continue }
                                self.fatal(
                                    format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
                                    get.name.pos.start_line,
                                    get.name.pos.start_pos,
                                    None
                                );
                            }
                            SymbolKind::Type(t) => {
                                if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
                                    return TypeOption::None;
                                } else {
                                    self.fatal(
                                        format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
                                        get.name.pos.start_line,
                                        get.name.pos.start_pos,
                                        None
                                    );
                                }
                            }
                            _ => unreachable!()
                        }
                    }
                }
                _ => unreachable!()
            }
        }

        TypeOption::None
    }

    fn visit_set(&mut self, set: &Set) -> TypeOption {
        set.object.accept(self);
        set.name.accept(self);
        set.value.accept(self);

        TypeOption::None
    }

    fn visit_cast(&mut self, cast: &Cast) -> TypeOption {
        cast.value.accept(self);
        cast.type_.accept(self);

        TypeOption::None
    }

    fn visit_type(&mut self, type_: &Type) -> TypeOption {
        let name = type_.to_string().split(" ").last().unwrap().to_string();
        if type_.is_primitive() {
            return TypeOption::Type(type_.clone());
        }

        if name.contains(".") {
            let parent = name.split(".").collect::<Vec<&str>>()[0].to_string();
            let name = name.split(".").collect::<Vec<&str>>()[1].to_string();
            let x = self.symtable.current().unwrap().get_module(parent.clone());

            if let Some(m) = x {
                for e in m.exported_symbols {
                    match e.kind {
                        SymbolKind::Type(t) => {
                            if t.to_string() == name {
                                return TypeOption::Type(t.clone());
                            }
                        }
                        SymbolKind::Struct(s) => {
                            let s_name = match s.name.kind {
                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                _ => unreachable!()
                            };

                            if s_name == name {
                                return TypeOption::Type(s.type_.clone());
                            }
                        }
                        SymbolKind::Enum(e) => {
                            let e_name = match e.name.kind {
                                ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                                _ => unreachable!()
                            };

                            if e_name == name {
                                return TypeOption::Type(e.type_.clone());
                            }
                        }
                        _ => {}
                    }
                }

                let x = self.symtable.suggest(name.clone());
                if x.is_some() {
                    self.error(
                        format!("Type '{}' does not exist.", name).as_str(),
                        type_.pos.start_line,
                        type_.pos.start_pos,
                        Some(vec![format!("Did you mean '{}'?", x.unwrap())])
                    );
                } else {
                    self.error(
                        format!("Type '{}' does not exist.", name).as_str(),
                        type_.pos.start_line,
                        type_.pos.start_pos,
                        None
                    );
                }
            } else {
                let x = self.symtable.suggest(name.clone());
                if x.is_some() {
                    self.error(
                        format!("Type '{}' does not exist.", name).as_str(),
                        type_.pos.start_line,
                        type_.pos.start_pos,
                        Some(vec![format!("Did you mean '{}'?", x.unwrap())])
                    );
                } else {
                    self.error(
                        format!("Type '{}' does not exist.", name).as_str(),
                        type_.pos.start_line,
                        type_.pos.start_pos,
                        None
                    );
                }
            }
        }

        if self.symtable.current().unwrap().get_type_by_name(name.clone()).is_none() {
            let x = self.symtable.suggest(name.clone());
            if x.is_some() {
                self.error(
                    format!("Type '{}' does not exist.", name).as_str(),
                    type_.pos.start_line,
                    type_.pos.start_pos,
                    Some(vec![format!("Did you mean '{}'?", x.unwrap())])
                );
            } else {
                self.error(
                    format!("Type '{}' does not exist.", name).as_str(),
                    type_.pos.start_line,
                    type_.pos.start_pos,
                    None
                );
            }
        }

        return TypeOption::Type(type_.clone());
    }
}