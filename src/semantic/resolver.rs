// Ethan Olchik
// src/semantic/resolver.rs
// The resolver is used to resolve names into symbols.

//> Imports

use crate::{
    semantic::{
        types::*,
        *
    },
    utils::imports::ImportHandler,
    errors::errors::{
        Diagnostic,
        DiagnosticKind
    },
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
    pub fn new(symtable: SymbolTable, filename: String, source: String) -> Self {
        Self {
            symtable,
            filename: filename.clone(),
            source,
            import_handler: ImportHandler::new(filename),
            had_error: false,
            in_function_body: false,
            in_loop: false,
            past_decl_queue_loop: false
        }
    }

    pub fn resolve(&mut self, program: &Module) {
        program.accept(self);
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

    fn resolve_object_name(&mut self, kind: &ExpressionKind) -> String {
        match kind {
            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
            ExpressionKind::StructInit(s) => s.name.lexeme.clone(),
            ExpressionKind::Call(c) => {
                c.accept(self);
                match &c.callee.kind {
                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                    _ => unreachable!(),
                }
            }
            ExpressionKind::Get(g) => {
                let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                get_parts[0].0.clone()
            }
            _ => unreachable!(),
        }
    }

    fn resolve_get_name_info(&self, kind: &ExpressionKind) -> GetNameInfo {
        match kind {
            ExpressionKind::Identifier(id) => GetNameInfo {
                name: id.name.lexeme.clone(),
                is_call: false,
                pos: Position {
                    start_line: id.name.line,
                    start_pos: id.name.pos,
                    end_line: id.name.line,
                    end_pos: id.name.pos + id.name.lexeme.len(),
                },
            },
            ExpressionKind::StructInit(s) => GetNameInfo {
                name: s.name.lexeme.clone(),
                is_call: false,
                pos: Position {
                    start_line: s.name.line,
                    start_pos: s.name.pos,
                    end_line: s.name.line,
                    end_pos: s.name.pos + s.name.lexeme.len(),
                },
            },
            ExpressionKind::Call(c) => {
                let name = match &c.callee.kind {
                    ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                    _ => unreachable!(),
                };
                GetNameInfo {
                    name,
                    is_call: true,
                    pos: Position {
                        start_line: c.callee.pos.start_line,
                        start_pos: c.callee.pos.start_pos,
                        end_line: c.callee.pos.end_line,
                        end_pos: c.callee.pos.end_pos,
                    },
                }
            }
            ExpressionKind::Get(g) => {
                let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                GetNameInfo {
                    name: get_parts[0].0.clone(),
                    is_call: get_parts[0].1,
                    pos: Position {
                        start_line: g.object.pos.start_line,
                        start_pos: g.object.pos.start_pos,
                        end_line: g.object.pos.end_line,
                        end_pos: g.object.pos.end_pos,
                    },
                }
            }
            _ => unreachable!(),
        }
    }
    

    fn is_name_exported(&self, module: &MatchaModule, name: &String) -> bool {
        module.exported_symbols.iter().any(|sym| match &sym.kind {
            SymbolKind::Constant(c) => matches!(&c.name.kind, ExpressionKind::Identifier(id) if id.name.lexeme == *name),
            SymbolKind::Function(f) => matches!(&f.name.kind, ExpressionKind::Identifier(id) if id.name.lexeme == *name),
            SymbolKind::Struct(s) => matches!(&s.name.kind, ExpressionKind::Identifier(id) if id.name.lexeme == *name),
            SymbolKind::Enum(e) => matches!(&e.name.kind, ExpressionKind::Identifier(id) if id.name.lexeme == *name),
            _ => false,
        })
    }    

    fn suggest_and_fatal(&mut self, get_name_info: &GetNameInfo, object_name: &String) {
        let suggestion = self.symtable.current().unwrap().get_module(object_name.clone()).unwrap().suggest(get_name_info.name.clone());
        let message = format!("Name '{}' does not exist in module '{}'.", get_name_info.name, object_name);
        self.fatal(
            &message,
            get_name_info.pos.start_line,
            get_name_info.pos.start_pos,
            suggestion.map(|s| vec![format!("Did you mean {}?", s)]),
        );
    }

    fn is_defined_or_module(&mut self, id: &Identifier) -> bool {
        self.symtable.current_mut().lookup(id.name.lexeme.clone()) ||
        self.symtable.current().unwrap().get_module(id.name.lexeme.clone()).is_some()
    }

    fn suggest_and_error(&mut self, name: &String, line: usize, pos: usize) {
        let suggestion = self.symtable.suggest(name.clone());
        let message = format!("Undefined name '{}'", name);
        self.error(
            &message,
            line,
            pos,
            suggestion.map(|s| vec![format!("Did you mean {}?", s)]),
        );
    }

    fn process_symbol(&mut self, symbol_name: &String, get_name_info: &GetNameInfo) {
        if let Some(sym) = self.symtable.current().unwrap().find(symbol_name.clone()) {
            match sym.kind {
                SymbolKind::Constant(ref v) | SymbolKind::Variable(ref v) => {
                    self.process_variable_or_constant(v, get_name_info);
                }
                SymbolKind::Method(ref f) => {
                    self.process_method(f, get_name_info);
                }
                SymbolKind::Struct(ref s) => {
                    self.process_struct_or_enum_fields(&SymbolKind::Struct(s.clone()), get_name_info);
                }
                SymbolKind::Enum(ref e) => {
                    self.process_struct_or_enum_fields(&SymbolKind::Enum(e.clone()), get_name_info);
                }
                SymbolKind::Type(ref t) => {
                    self.process_type(t, get_name_info);
                }
                _ => unreachable!(),
            }
        }
    }

    fn process_get_chain(&mut self, g: &Get, get: &Get) {
        let get_parts = self.symtable.get_to_list(g.clone(), &mut self.clone());
        let mut obj = self.symtable.current().unwrap().find(get_parts[0].0.clone()).unwrap().kind;
    
        for i in 0..get_parts.len() - 1 {
            let next_get = &get_parts[i + 1];
            obj = match obj {
                SymbolKind::Constant(ref c) | SymbolKind::Variable(ref c) => {
                    self.process_chained_variable_or_constant(c, next_get)
                }
                SymbolKind::Method(ref f) => {
                    self.process_chained_method(f, next_get)
                }
                SymbolKind::Struct(ref s) => {
                    self.process_chained_struct_or_enum(&obj, next_get)
                }
                SymbolKind::Enum(ref e) => {
                    self.process_chained_struct_or_enum(&obj, next_get)
                }
                _ => unreachable!(),
            }
        }
    }
    

    fn process_variable_or_constant(&mut self, v: &Variable, get_name_info: &GetNameInfo) {
        let t = v.type_.clone();
        if t.is_primitive() {
            self.check_primitive_type_contains_name(&t, get_name_info);
        } else {
            let ty = t.to_string().split_whitespace().nth(1).unwrap().to_string();
            self.check_struct_or_enum_contains_name(&ty, get_name_info);
        }
    }

    fn process_method(&mut self, f: &Function, get_name_info: &GetNameInfo) {
        let t = f.type_.clone();
        if t.is_primitive() {
            self.check_primitive_type_contains_name(&t, get_name_info);
        } else {
            let ty = t.to_string().split_whitespace().nth(1).unwrap().to_string();
            self.check_struct_or_enum_contains_name(&ty, get_name_info);
        }
    }

    fn process_type(&mut self, t: &Type, get_name_info: &GetNameInfo) {
        if self.symtable.primitive_type_contains_name(t.clone(), get_name_info.name.clone()) {
            return;
        } else {
            self.fatal(
                &format!("Name '{}' does not exist in type '{}'.", get_name_info.name, t.to_string()),
                get_name_info.pos.start_line,
                get_name_info.pos.start_pos,
                None,
            );
        }
    }

    fn process_struct_or_enum_fields(&mut self, s: &SymbolKind, get_name_info: &GetNameInfo) {
        match s {
            SymbolKind::Struct(st) => {
                if get_name_info.is_call {
                    for method in &st.methods {
                        let name = match &method.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == get_name_info.name {
                            return;
                        }
                    }
                } else {
                    for field in &st.fields {
                        let name = match &field.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == get_name_info.name {
                            return;
                        }
                    }
                }

                self.fatal(
                    &format!("Name '{}' does not exist in struct '{}'.", get_name_info.name, get_name_info.name),
                    get_name_info.pos.start_line,
                    get_name_info.pos.start_pos,
                    None,
                );
            }
            SymbolKind::Enum(e) => {
                if get_name_info.is_call {
                    for method in &e.methods {
                        let name = match &method.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == get_name_info.name {
                            return;
                        }
                    }
                } else {
                    for variant in &e.variants {
                        let name = match &variant.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == get_name_info.name {
                            return;
                        }
                    }
                }

                self.fatal(
                    &format!("Name '{}' does not exist in enum '{}'.", get_name_info.name, get_name_info.name),
                    get_name_info.pos.start_line,
                    get_name_info.pos.start_pos,
                    None,
                );
            }
            _ => unreachable!(),
        }
    }

    fn check_primitive_type_contains_name(&mut self, t: &Type, get_name_info: &GetNameInfo) {
        if self.symtable.primitive_type_contains_name(t.clone(), get_name_info.name.clone()) {
            return;
        } else {
            self.fatal(
                &format!("Name '{}' does not exist in type '{}'.", get_name_info.name, t.to_string()),
                get_name_info.pos.start_line,
                get_name_info.pos.start_pos,
                None,
            );
        }
    }

    fn check_struct_or_enum_contains_name(&mut self, ty: &String, get_name_info: &GetNameInfo) {
        if let Some(sym) = self.symtable.current().unwrap().find(ty.clone()) {
            match sym.kind {
                SymbolKind::Struct(ref s) => self.process_struct_or_enum_fields(&SymbolKind::Struct(s.clone()), get_name_info),
                SymbolKind::Enum(ref e) => self.process_struct_or_enum_fields(&SymbolKind::Enum(e.clone()), get_name_info),
                _ => unreachable!(),
            }
        }
    }

    fn process_chained_variable_or_constant(&mut self, c: &Variable, next_get: &(String, bool)) -> SymbolKind {
        let t = c.type_.clone();
        if t.is_primitive() {
            self.check_primitive_type_contains_name(&t, &GetNameInfo {
                name: next_get.0.clone(),
                is_call: next_get.1,
                pos: Position { start_line: c.name.pos.start_line, start_pos: c.name.pos.start_pos, end_line: c.name.pos.end_line, end_pos: c.name.pos.end_pos },
            });
            SymbolKind::Type(t)
        } else {
            let ty = t.to_string().split_whitespace().nth(1).unwrap().to_string();
            self.check_struct_or_enum_contains_name(&ty, &GetNameInfo {
                name: next_get.0.clone(),
                is_call: next_get.1,
                pos: Position { start_line: c.name.pos.start_line, start_pos: c.name.pos.start_pos, end_line: c.name.pos.end_line, end_pos: c.name.pos.end_pos },
            });
            self.symtable.current().unwrap().find(ty.clone()).unwrap().kind
        }
    }

    fn process_chained_method(&mut self, f: &Function, next_get: &(String, bool)) -> SymbolKind {
        let t = f.type_.clone();
        if t.is_primitive() {
            self.check_primitive_type_contains_name(&t, &GetNameInfo {
                name: next_get.0.clone(),
                is_call: next_get.1,
                pos: Position { start_line: f.name.pos.start_line, start_pos: f.name.pos.start_pos, end_line: f.name.pos.end_line, end_pos: f.name.pos.end_pos },
            });
            SymbolKind::Type(t)
        } else {
            let ty = t.to_string().split_whitespace().nth(1).unwrap().to_string();
            self.check_struct_or_enum_contains_name(&ty, &GetNameInfo {
                name: next_get.0.clone(),
                is_call: next_get.1,
                pos: Position { start_line: f.name.pos.start_line, start_pos: f.name.pos.start_pos, end_line: f.name.pos.end_line, end_pos: f.name.pos.end_pos },
            });
            self.symtable.current().unwrap().find(ty.clone()).unwrap().kind
        }
    }    

    fn process_chained_struct_or_enum(&mut self, kind: &SymbolKind, next_get: &(String, bool)) -> SymbolKind {
        match kind {
            SymbolKind::Struct(st) => {
                if next_get.1 {
                    for method in &st.methods {
                        let name = match &method.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == next_get.0 {
                            return SymbolKind::Method(method.clone());
                        }
                    }
                } else {
                    for field in &st.fields {
                        let name = match &field.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == next_get.0 {
                            if field.type_.modifiers.is_const {
                                return SymbolKind::Constant(field.clone());
                            } else {
                                return SymbolKind::Variable(field.clone());
                            }
                        }
                    }
                }

                self.fatal(
                    &format!("Name '{}' does not exist in '{}'.", next_get.0, match &st.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!(),
                    }),
                    0, // Use the correct line number and position if available
                    0,
                    None,
                );
            }
            SymbolKind::Enum(st) => {
                if next_get.1 {
                    for method in &st.methods {
                        let name = match &method.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == next_get.0 {
                            return SymbolKind::Method(method.clone());
                        }
                    }
                } else {
                    for variant in &st.variants {
                        let name = match &variant.name.kind {
                            ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                            _ => unreachable!(),
                        };
                        if name == next_get.0 {
                            return SymbolKind::Constant(variant.clone());
                        }
                    }
                }

                self.fatal(
                    &format!("Name '{}' does not exist in '{}'.", next_get.0, match &st.name.kind {
                        ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
                        _ => unreachable!(),
                    }),
                    0, // Use the correct line number and position if available
                    0,
                    None,
                );
            }
            _ => unreachable!(),
        }
        
        panic!()
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

    // fn visit_get(&mut self, get: &Get) -> TypeOption {
    //     let object_name = match &get.object.kind {
    //         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //         ExpressionKind::StructInit(s) => {
    //             s.name.lexeme.clone()
    //         }
    //         ExpressionKind::Call(c) => {
    //             c.accept(self);
    //             match &c.callee.kind {
    //                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                 _ => unreachable!()
    //             }
    //         }
    //         ExpressionKind::Get(g) => {
    //             let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
    //             get_parts[0].0.clone()
    //         }
    //         _ => {
    //             return TypeOption::None // TODO: check for other expressions.
    //         }
    //     };

    //     let mut is_call: bool = false;

    //     let get_name = match &get.name.kind {
    //         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //         ExpressionKind::StructInit(s) => {
    //             s.name.lexeme.clone()
    //         }
    //         ExpressionKind::Call(c) => {
    //             println!("{:?}", get);
    //             is_call = true;

    //             match &c.callee.kind {
    //                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                 _ => unreachable!()
    //             }
    //         }
    //         ExpressionKind::Get(g) => {
    //             let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
    //             is_call = get_parts[0].1.clone();
    //             get_parts[0].0.clone()
    //         }
    //         _ => {
    //             return TypeOption::None // TODO: check for other expressions
    //         }
    //     };


    //     let x = self.symtable.current().unwrap().get_module(object_name.clone());

    //     if let Some(m) = x {
    //         let mut found = false;

    //         for sym in m.exported_symbols.clone() {
    //             match sym.kind {
    //                 SymbolKind::Constant(c) => {
    //                     let name = match &c.name.kind {
    //                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                         _ => unreachable!()
    //                     };
    //                     if name == get_name {
    //                         found = true;
    //                     }
    //                 },
    //                 SymbolKind::Function(f) => {
    //                     let name = match &f.name.kind {
    //                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                         _ => unreachable!()
    //                     };
    //                     if name == get_name {
    //                         found = true;
    //                     }
    //                 },
    //                 SymbolKind::Struct(s) => {
    //                     let name = match &s.name.kind {
    //                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                         _ => unreachable!()
    //                     };
    //                     if name == get_name {
    //                         found = true;
    //                     }
    //                 },
    //                 SymbolKind::Enum(e) => {
    //                     let name = match &e.name.kind {
    //                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                         _ => unreachable!()
    //                     };
    //                     if name == get_name {
    //                         found = true;
    //                     }
    //                 },
    //                 _ => {}
    //             }
    //         }

    //         if !found {
    //             let x = m.suggest(get_name.clone());

    //             if x.is_some() {
    //                 self.fatal(
    //                     format!("Name '{}' does not exist in module '{}'.", get_name, object_name).as_str(),
    //                     get.name.pos.start_line,
    //                     get.name.pos.start_pos,
    //                     Some(vec![format!("Did you mean {}?", x.unwrap())])
    //                 );
    //             } else {
    //                 self.fatal(
    //                     format!("Name '{}' does not exist in module '{}'.", get_name, object_name).as_str(),
    //                     get.name.pos.start_line,
    //                     get.name.pos.start_pos,
    //                     None
    //                 );
    //             }
    //         }

    //         return TypeOption::None;
    //     }

    //     let mut last: Option<SymbolKind> = None;

    //     match &get.object.kind {
    //         ExpressionKind::Identifier(id) => {
    //             if !(self.symtable.current_mut().lookup(id.name.lexeme.clone()) || self.symtable.current().unwrap().get_module(id.name.lexeme.clone()).is_some()) {
    //                 let x = self.symtable.suggest(id.name.lexeme.clone());

    //                 if x.is_some() {
    //                     self.error(
    //                         format!("Undefined name '{}'", id.name.lexeme).as_str(),
    //                         id.name.line,
    //                         id.name.pos,
    //                         Some(vec![format!("Did you mean {}?", x.unwrap())])
    //                     );
    //                 } else {
    //                     self.error(
    //                         format!("Undefined name '{}'", id.name.lexeme).as_str(),
    //                         id.name.line,
    //                         id.name.pos,
    //                         None
    //                     );
    //                 }
    //             } else {
    //                 let sym = self.symtable.current().unwrap().find(id.name.lexeme.clone()).unwrap();
    //                 match sym.kind {
    //                     SymbolKind::Constant(ref v) | SymbolKind::Variable(ref v) => {
    //                         let t = v.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = v.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 let mut found = false;
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         if is_call {
    //                                             for m in &s.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };

    //                                                 if name == get_name {
    //                                                     found = true;
    //                                                     last = Some(SymbolKind::Method(m.clone()));
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for f in &s.fields {
    //                                                 let name = match &f.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     found = true;
    //                                                     last = Some(SymbolKind::Variable(f.clone()));
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         // So here, we have already read "get.name". The problem is that later in the code we are checking if get.name exists in get.name
    //                                         // since thats what we're setting last to be. We need to check if the variable found is the same as get.name.
    //                                         if !found {
    //                                             self.fatal(
    //                                                 format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                                 get.name.pos.start_line,
    //                                                 get.name.pos.start_pos,
    //                                                 None
    //                                             );
    //                                         } else {
    //                                             if let Some(ref l) = last {
    //                                                 match l {
    //                                                     SymbolKind::Variable(v) => {
    //                                                         let name = match &v.name.kind {
    //                                                             ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                             _ => unreachable!()
    //                                                         };
    //                                                         if name == get_name {
    //                                                             return TypeOption::None;
    //                                                         }
    //                                                     }
    //                                                     SymbolKind::Method(m) => {
    //                                                         let name = match &m.name.kind {
    //                                                             ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                             _ => unreachable!()
    //                                                         };
    //                                                         if name == get_name {
    //                                                             return TypeOption::None;
    //                                                         }
    //                                                     }
    //                                                     _ => {}
    //                                                 }
    //                                             }
    //                                         }
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         if is_call {
    //                                             for m in &e.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     found = true;
    //                                                     last = Some(SymbolKind::Method(m.clone()));
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for v in &e.variants {
    //                                                 let name = match &v.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     found = true;
    //                                                     last = Some(SymbolKind::Constant(v.clone()));
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if !found {
    //                                             self.fatal(
    //                                                 format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                                 get.name.pos.start_line,
    //                                                 get.name.pos.start_pos,
    //                                                 None
    //                                             );
    //                                         }
    //                                     }
    //                                     _ => {
    //                                         unreachable!()
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Method(f) => {
    //                         let t = f.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         if is_call {
    //                                             for m in &s.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     return TypeOption::None;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for f in &s.fields {
    //                                                 let name = match &f.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     return TypeOption::None;
    //                                                 }
    //                                             }
    //                                         }

    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         if is_call {
    //                                             for m in &e.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     return TypeOption::None;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for v in &e.variants {
    //                                                 let name = match &v.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     return TypeOption::None;
    //                                                 }
    //                                             }
    //                                         }

    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => unreachable!()
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Struct(s) => {
    //                         if is_call {
    //                             for m in &s.methods {
    //                                 let name = match &m.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 }; // TODO: Fix call checking. Currently, only certain calls will work.
    //                                     // TODO: Fix access modifiers. We are not checking if the method/attribute is public.
    //                                     // 
    //                                 if name == get_name && m.type_.modifiers.is_static { 
    //                                     return TypeOption::None;
    //                                 }
    //                             }
    //                         } else {
    //                             for f in &s.fields {
    //                                 let name = match &f.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name && f.type_.modifiers.is_static {
    //                                     return TypeOption::None;
    //                                 }
    //                             }
    //                         }

    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             Some(vec![
    //                                 "Did you forget to initialise the object?".to_string()
    //                             ])
    //                         );
    //                     }
    //                     SymbolKind::Enum(e) => {
    //                         if is_call {
    //                             for m in &e.methods {
    //                                 let name = match &m.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name {
    //                                     return TypeOption::None;
    //                                 }
    //                             }
    //                         } else {
    //                             for v in &e.variants {
    //                                 let name = match &v.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name {
    //                                     return TypeOption::None;
    //                                 }
    //                             }
    //                         }

    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Type(t) => {
    //                         // TODO: Static methods
    //                         if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                             return TypeOption::None;
    //                         } else {
    //                             self.fatal(
    //                                 format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                 get.name.pos.start_line,
    //                                 get.name.pos.start_pos,
    //                                 None
    //                             );
    //                         }
    //                     }

    //                     _ => unreachable!()
    //                 }
    //             }
    //         }
    //         ExpressionKind::Get(g) => {
    //             let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
    //             let mut obj = self.symtable.current().unwrap().find(get_parts[0].0.clone()).unwrap().kind;

    //             for i in 0..get_parts.len() {
    //                 let get_name: String;
    //                 let is_call: bool;
    //                 if i == get_parts.len() - 1 {
    //                     break;
    //                 } else {
    //                     get_name = get_parts[i + 1].0.clone();
    //                     is_call = get_parts[i + 1].1;
    //                 }

    //                 match obj.clone() {
    //                     SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
    //                         let t = c.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 obj = t.get_name(get_name.clone());
    //                                 break;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 let mut found = false;
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         if is_call {
    //                                             for m in &s.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Method(m.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for f in &s.fields {
    //                                                 let name = match &f.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Variable(f.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if found { continue }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         if is_call {
    //                                             for m in &e.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Method(m.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for v in &e.variants {
    //                                                 let name = match &v.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Constant(v.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if found { continue }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => unreachable!()
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Method(f) => {
    //                         let t = f.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 obj = t.get_name(get_name.clone());
    //                                 break;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 let mut found = false;
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         if is_call {
    //                                             for m in &s.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Method(m.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for f in &s.fields {
    //                                                 let name = match &f.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Variable(f.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if found { continue }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         if is_call {
    //                                             for m in &e.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Method(m.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for v in &e.variants {
    //                                                 let name = match &v.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Constant(v.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if found { continue }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => unreachable!()
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Struct(s) => {
    //                         let mut found = false;

    //                         if is_call {
    //                             for m in &s.methods {
    //                                 let name = match &m.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name && m.type_.modifiers.is_static {
    //                                     obj = SymbolKind::Method(m.clone());
    //                                     found = true;
    //                                     break;
    //                                 }
    //                             }
    //                         } else {
    //                             for f in &s.fields {
    //                                 let name = match &f.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name && f.type_.modifiers.is_static {
    //                                     obj = SymbolKind::Variable(f.clone());
    //                                     found = true;
    //                                     break;
    //                                 }
    //                             }
    //                         }

    //                         if found { continue }
    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Enum(e) => {
    //                         let mut found = false;
    //                         if is_call {
    //                             for m in &e.methods {
    //                                 let name = match &m.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name {
    //                                     obj = SymbolKind::Method(m.clone());
    //                                     found = true;
    //                                     break;
    //                                 }
    //                             }
    //                         } else {
    //                             for v in &e.variants {
    //                                 let name = match &v.name.kind {
    //                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                     _ => unreachable!()
    //                                 };
    //                                 if name == get_name {
    //                                     obj = SymbolKind::Constant(v.clone());
    //                                     found = true;
    //                                     break;
    //                                 }
    //                             }
    //                         }

    //                         if found { continue }
    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Type(t) => {
    //                         // TODO: implement static methods
    //                         if t.is_primitive() {
    //                             if t.contains_name(get_name.clone()) {
    //                                 obj = t.get_name(get_name.clone());
    //                             } else {
    //                                 self.error(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = t.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());
    //                             if x.is_some() {
    //                                 let t_ = x.unwrap();
    //                                 let mut found = false;

    //                                 match t_.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         if is_call {
    //                                             for m in &s.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Method(m.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for f in &s.fields {
    //                                                 let name = match &f.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Variable(f.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if found { continue }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         if is_call {
    //                                             for m in &e.methods {
    //                                                 let name = match &m.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Method(m.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         } else {
    //                                             for v in &e.variants {
    //                                                 let name = match &v.name.kind {
    //                                                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                     _ => unreachable!()
    //                                                 };
    //                                                 if name == get_name {
    //                                                     obj = SymbolKind::Constant(v.clone());
    //                                                     found = true;
    //                                                     break;
    //                                                 }
    //                                             }
    //                                         }

    //                                         if found { continue }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => unreachable!()
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     _ => { unreachable!() }
    //                 }
    //             }

    //             last = Some(obj);
    //         }
    //         ExpressionKind::Call(c) => {
    //             c.accept(self);
    //         }
    //         ExpressionKind::StructInit(s) => {
    //             s.accept(self);
    //         }
    //         _ => {}
    //     }

    //     // TODO: Implement StructInit
    //     // TODO: Static Attributes
    //     // TODO: Index, Set.
    //     // Last = p, next it should go to a call and then another get.
    //     if last.is_some() {
    //         match get.name.kind.clone() {
    //             ExpressionKind::Identifier(id) => {
    //                 let get_name = id.name.lexeme.clone();
    //                 match &last.unwrap() {
    //                     SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
    //                         let t = c.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 let found = false;
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         for m in &s.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
    //                                         for f in &s.fields {
    //                                             let name = match &f.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }

    //                                         if found { return TypeOption::None }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         for m in &e.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
    //                                         for v in &e.variants {
    //                                             let name = match &v.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
    //                                     }
    //                                     _ => {
    //                                         unreachable!()
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Method(f) => {
    //                         let t = f.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 let found = false;
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         for m in &s.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
    //                                         for f in &s.fields {
    //                                             let name = match &f.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }

    //                                         if found { return TypeOption::None }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         for m in &e.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
    //                                         for v in &e.variants {
    //                                             let name = match &v.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }

    //                                         if found { return TypeOption::None }
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty.to_string()).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => unreachable!()
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Struct(s) => {
    //                         for m in &s.methods {
    //                             let name = match &m.name.kind {
    //                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                 _ => unreachable!()
    //                             };
    //                             if name == get_name && m.type_.modifiers.is_static {
    //                                 return TypeOption::None;
    //                             }
    //                         }
    //                         for f in &s.fields {
    //                             let name = match &f.name.kind {
    //                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                 _ => unreachable!()
    //                             };
    //                             if name == get_name && f.type_.modifiers.is_static {
    //                                 return TypeOption::None;
    //                             }
    //                         }

    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Enum(e) => {
    //                         let found = false;
    //                         for v in &e.variants {
    //                             let name = match &v.name.kind {
    //                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                 _ => unreachable!()
    //                             };
    //                             if name == get_name {
    //                                 return TypeOption::None;
    //                             }
    //                         }
    //                         for m in &e.methods {
    //                             let name = match &m.name.kind {
    //                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                 _ => unreachable!()
    //                             };
    //                             if name == get_name {
    //                                 return TypeOption::None;
    //                             }
    //                         }

    //                         if found { return TypeOption::None }
    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Type(t) => {
    //                         if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                             return TypeOption::None;
    //                         } else {
    //                             self.fatal(
    //                                 format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                 get.name.pos.start_line,
    //                                 get.name.pos.start_pos,
    //                                 None
    //                             );
    //                         }
    //                     }
    //                     _ => unreachable!()
    //                 }
    //             }
    //             ExpressionKind::Call(c) => {
    //                 c.accept(self);
    //                 let get_name = match c.callee.kind {
    //                     ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                     _ => unreachable!()
    //                 };

    //                 match &last.unwrap() {
    //                     SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
    //                         let t = c.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         for m in &s.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }

    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         for m in &e.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
                            
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => {
    //                                         unreachable!()
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Method(f) => {
    //                         let t = f.type_.clone();
    //                         if t.is_primitive() {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         } else {
    //                             let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                             let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                             if x.is_some() {
    //                                 let t = x.unwrap();
    //                                 match t.kind {
    //                                     SymbolKind::Struct(s) => {
    //                                         for m in &s.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }

    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     SymbolKind::Enum(e) => {
    //                                         for m in &e.methods {
    //                                             let name = match &m.name.kind {
    //                                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                 _ => unreachable!()
    //                                             };
    //                                             if name == get_name {
    //                                                 return TypeOption::None;
    //                                             }
    //                                         }
                            
    //                                         self.fatal(
    //                                             format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                             get.name.pos.start_line,
    //                                             get.name.pos.start_pos,
    //                                             None
    //                                         );
    //                                     }
    //                                     _ => {
    //                                         unreachable!()
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                     }
    //                     SymbolKind::Struct(s) => {
    //                         for m in &s.methods {
    //                             let name = match &m.name.kind {
    //                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                 _ => unreachable!()
    //                             };
    //                             if name == get_name && m.type_.modifiers.is_static {
    //                                 return TypeOption::None;
    //                             }
    //                         }

    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Enum(e) => {
    //                         for m in &e.methods {
    //                             let name = match &m.name.kind {
    //                                 ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                 _ => unreachable!()
    //                             };
    //                             if name == get_name {
    //                                 return TypeOption::None;
    //                             }
    //                         }

    //                         self.fatal(
    //                             format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                             get.name.pos.start_line,
    //                             get.name.pos.start_pos,
    //                             None
    //                         );
    //                     }
    //                     SymbolKind::Type(t) => {
    //                         if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                             return TypeOption::None;
    //                         } else {
    //                             self.fatal(
    //                                 format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                 get.name.pos.start_line,
    //                                 get.name.pos.start_pos,
    //                                 None
    //                             );
    //                         }
    //                     }
    //                     _ => unreachable!()
    //                 }
    //             }
    //             ExpressionKind::Get(g) => {
    //                 let get_parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
    //                 let mut obj = last.unwrap();

    //                 for i in 0..get_parts.len() {
    //                     let get_name: String;
    //                     let is_call: bool;

    //                     // if i == get_parts.len() - 1 || i == 0 {
    //                     //     println!("i: {}", i);
    //                     //     println!("get_parts: {:?}", get_parts);
    //                     //     println!("get_parts[i]: {:?}", get_parts[i]);
    //                     //     println!("{:?}", obj);
    //                     //     get_name = get_parts[i].0.clone();
    //                     //     is_call = get_parts[i].1;
    //                     // } else if i > 0 {
    //                     //     println!("i: {}", i);
    //                     //     println!("get_parts: {:?}", get_parts);
    //                     //     println!("get_parts[i+1]: {:?}", get_parts[i+1]);
    //                     //     println!("{:?}", obj);
    //                     //     get_name = get_parts[i + 1].0.clone();
    //                     //     is_call = get_parts[i + 1].1;
    //                     // } else {
    //                     //     get_name = get_parts[i].0.clone();
    //                     //     is_call = get_parts[i].1;
    //                     // }

    //                     get_name = get_parts[i].0.clone();
    //                     is_call = get_parts[i].1;

    //                     match obj.clone() {
    //                         SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
    //                             let t = c.type_.clone();
    //                             if t.is_primitive() {
    //                                 if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                     // obj = t.get_name(get_name.clone());
    //                                     break;
    //                                 } else {
    //                                     self.fatal(
    //                                         format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                         get.name.pos.start_line,
    //                                         get.name.pos.start_pos,
    //                                         None
    //                                     );
    //                                 }
    //                             } else {
    //                                 let ty = c.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                                 let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                                 if x.is_some() {
    //                                     let t = x.unwrap();
    //                                     let mut found = false;
    //                                     match t.kind {
    //                                         SymbolKind::Struct(s) => {
    //                                             if is_call {
    //                                                 for m in &s.methods {
    //                                                     let name = match &m.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Method(m.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             } else {
    //                                                 for f in &s.fields {
    //                                                     let name = match &f.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Variable(f.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             }

    //                                             if found { continue }
    //                                             self.fatal(
    //                                                 format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                                 get.name.pos.start_line,
    //                                                 get.name.pos.start_pos,
    //                                                 None
    //                                             );
    //                                         }
    //                                         SymbolKind::Enum(e) => {
    //                                             if is_call {
    //                                                 for m in &e.methods {
    //                                                     let name = match &m.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Method(m.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             } else {
    //                                                 for v in &e.variants {
    //                                                     let name = match &v.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Constant(v.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             }

    //                                             if found { continue }
    //                                             self.fatal(
    //                                                 format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                                 get.name.pos.start_line,
    //                                                 get.name.pos.start_pos,
    //                                                 None
    //                                             );
    //                                         }
    //                                         _ => unreachable!()
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                         SymbolKind::Method(f) => {
    //                             let t = f.type_.clone();
    //                             if t.is_primitive() {
    //                                 if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                     // obj = t.get_name(get_name.clone());
    //                                     break;
    //                                 } else {
    //                                     self.fatal(
    //                                         format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                         get.name.pos.start_line,
    //                                         get.name.pos.start_pos,
    //                                         None
    //                                     );
    //                                 }
    //                             } else {
    //                                 let ty = f.type_.to_string().split(" ").collect::<Vec<&str>>()[1].to_string();
    //                                 let x = self.symtable.current().unwrap().get_struct_or_enum_by_name(ty.clone());

    //                                 if x.is_some() {
    //                                     let t = x.unwrap();
    //                                     let mut found = false;
    //                                     match t.kind {
    //                                         SymbolKind::Struct(s) => {
    //                                             if is_call {
    //                                                 for m in &s.methods {
    //                                                     let name = match &m.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Method(m.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             } else {
    //                                                 for f in &s.fields {
    //                                                     let name = match &f.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Variable(f.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             }

    //                                             if found { continue }
    //                                             self.fatal(
    //                                                 format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                                 get.name.pos.start_line,
    //                                                 get.name.pos.start_pos,
    //                                                 None
    //                                             );
    //                                         }
    //                                         SymbolKind::Enum(e) => {
    //                                             if is_call {
    //                                                 for m in &e.methods {
    //                                                     let name = match &m.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Method(m.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             } else {
    //                                                 for v in &e.variants {
    //                                                     let name = match &v.name.kind {
    //                                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                                         _ => unreachable!()
    //                                                     };
    //                                                     if name == get_name {
    //                                                         obj = SymbolKind::Constant(v.clone());
    //                                                         found = true;
    //                                                         break;
    //                                                     }
    //                                                 }
    //                                             }

    //                                             if found { continue }
    //                                             self.fatal(
    //                                                 format!("Name '{}' does not exist in field of type {}.", get_name, ty).as_str(),
    //                                                 get.name.pos.start_line,
    //                                                 get.name.pos.start_pos,
    //                                                 None
    //                                             );
    //                                         }
    //                                         _ => unreachable!()
    //                                     }
    //                                 }
    //                             }
    //                         }
    //                         SymbolKind::Struct(s) => {
    //                             let mut found = false;

    //                             if is_call {
    //                                 for m in &s.methods {
    //                                     let name = match &m.name.kind {
    //                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                         _ => unreachable!()
    //                                     };
    //                                     if name == get_name && m.type_.modifiers.is_static {
    //                                         obj = SymbolKind::Method(m.clone());
    //                                         found = true;
    //                                         break;
    //                                     }
    //                                 }
    //                             } else {
    //                                 for f in &s.fields {
    //                                     let name = match &f.name.kind {
    //                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                         _ => unreachable!()
    //                                     };
    //                                     if name == get_name && f.type_.modifiers.is_static {
    //                                         obj = SymbolKind::Variable(f.clone());
    //                                         found = true;
    //                                         break;
    //                                     }
    //                                 }
    //                             }

    //                             if found { continue }
    //                             self.fatal(
    //                                 format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                                 get.name.pos.start_line,
    //                                 get.name.pos.start_pos,
    //                                 None
    //                             );
    //                         }
    //                         SymbolKind::Enum(e) => {
    //                             let mut found = false;
    //                             if is_call {
    //                                 for m in &e.methods {
    //                                     let name = match &m.name.kind {
    //                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                         _ => unreachable!()
    //                                     };
    //                                     if name == get_name {
    //                                         obj = SymbolKind::Method(m.clone());
    //                                         found = true;
    //                                         break;
    //                                     }
    //                                 }
    //                             } else {
    //                                 for v in &e.variants {
    //                                     let name = match &v.name.kind {
    //                                         ExpressionKind::Identifier(id) => id.name.lexeme.clone(),
    //                                         _ => unreachable!()
    //                                     };
    //                                     if name == get_name {
    //                                         obj = SymbolKind::Constant(v.clone());
    //                                         found = true;
    //                                         break;
    //                                     }
    //                                 }

    //                                 if found { continue }
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }

    //                             if found { continue }
    //                             self.fatal(
    //                                 format!("Name '{}' does not exist in '{}'.", get_name, object_name).as_str(),
    //                                 get.name.pos.start_line,
    //                                 get.name.pos.start_pos,
    //                                 None
    //                             );
    //                         }
    //                         SymbolKind::Type(t) => {
    //                             if self.symtable.primitive_type_contains_name(t.clone(), get_name.clone()) {
    //                                 return TypeOption::None;
    //                             } else {
    //                                 self.fatal(
    //                                     format!("Name '{}' does not exist in type '{}'.", get_name, t.to_string()).as_str(),
    //                                     get.name.pos.start_line,
    //                                     get.name.pos.start_pos,
    //                                     None
    //                                 );
    //                             }
    //                         }
    //                         _ => unreachable!()
    //                     }
    //                 }
    //             }
    //             _ => unreachable!()
    //         }
    //     }

    //     TypeOption::None
    // }

    fn visit_get(&mut self, get: &Get) -> TypeOption {
        let mut current_object = &get.object;
        let mut get_name_info = self.resolve_get_name_info(&get.name.kind);
    
        loop {
            match &current_object.kind {
                ExpressionKind::Identifier(id) => {
                    if !self.is_defined_or_module(id) {
                        self.suggest_and_error(&id.name.lexeme, id.name.line, id.name.pos);
                        return TypeOption::None;
                    } else {
                        self.process_symbol(&id.name.lexeme, &get_name_info);
                    }
                }
                ExpressionKind::Get(g) => {
                    let parts = self.symtable.get_to_list(*g.clone(), &mut self.clone());
                    let mut obj = self.symtable.current().unwrap().find(parts[0].0.clone()).unwrap().kind;
    
                    for i in 0..parts.len() - 1 {
                        let next_get = &parts[i + 1];
                        obj = match obj {
                            SymbolKind::Constant(ref c) | SymbolKind::Variable(ref c) => {
                                self.process_chained_variable_or_constant(c, next_get)
                            }
                            SymbolKind::Method(ref f) => {
                                self.process_chained_method(f, next_get)
                            }
                            SymbolKind::Struct(ref _s) => {
                                self.process_chained_struct_or_enum(&obj, next_get)
                            }
                            SymbolKind::Enum(ref _e) => {
                                self.process_chained_struct_or_enum(&obj, next_get)
                            }
                            _ => unreachable!(),
                        };
                    }
    
                    current_object = &g.object;
                    get_name_info = self.resolve_get_name_info(&g.name.kind);
                }
                _ => unreachable!(),
            }
    
            if let ExpressionKind::Get(_) = &current_object.kind {
                continue;
            } else {
                break;
            }
        }
    
        // Process the final part
        match &current_object.kind {
            ExpressionKind::Identifier(id) => {
                self.process_symbol(&id.name.lexeme, &get_name_info);
            }
            _ => unreachable!(),
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