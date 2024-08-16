use crate::{
    ast::ast::*,
    errors::errors::{Diagnostic, DiagnosticKind},
    semantic::{types::*, *},
    utils::imports::ImportHandler,
    frontend::lexer::token::{Token, TokenType},
    debug,
    codegen::type_to_string
};

use std::collections::HashSet;

// generate C code
pub struct Codegen {
    pub symtable: SymbolTable,
    pub import_handler: ImportHandler,

    pub filename: String,

    pub had_error: bool,

    pub code: String,

    pub current_function: String,

    pub current_call_object: Vec<Expression>,

    ptr_access: Vec<String>,

    cur_mod_name: Option<String>,
}

impl Codegen {
    pub fn new(filename: String, symtable: SymbolTable, import_handler: ImportHandler) -> Self {
        Self {
            symtable,
            import_handler,
            filename,
            had_error: false,
            code: String::new(),
            current_function: String::new(),
            current_call_object: Vec::new(),
            ptr_access: Vec::new(),
            cur_mod_name: None,
        }
    }

    pub fn generate(&mut self, program: &Module) {
        if self.symtable.current_mut().lookup("main".to_string()) {
            self.code += "#define MAINDEFINED\n";
        }
        self.code += "#include \"matcha.h\"\n";
        self.code += "#include <stdio.h>\n";

        for i in self.import_handler.resolved.clone() {
            self.cur_mod_name = Some(i.name.clone());
            for s in i.exported_symbols {
                match s.get() {
                    SymbolKind::Function(f) => {
                        f.accept(self);
                    }
                    SymbolKind::Variable(v) | SymbolKind::Constant(v) => {
                        v.accept(self);
                    }
                    SymbolKind::Struct(s) => {
                        s.accept(self);
                    }
                    SymbolKind::Enum(e) => {
                        e.accept(self);
                    }
                    _ => {}
                }

                self.code += ";\n";
            }
        }

        self.cur_mod_name = None;

        program.accept(self);

        // write to file
        let output_filename = self.filename.clone() + ".c";
        match std::fs::write(&output_filename, self.code.clone()) {
            Ok(_) => {},
            Err(err) => panic!("Failed to write to file {}: {}", output_filename, err),
        }
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

    fn name(&self, expr: Expression) -> String {
        match expr.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme,
            ExpressionKind::Call(call) => self.name(*call.callee),
            ExpressionKind::Get(get) => self.name(*get.name.clone()),
            _ => unreachable!(),
        }
    }
}

impl Visitor for Codegen {
    fn visit_module(&mut self, module: &Module) -> TypeOption {
        for statement in module.statements.clone() {
            statement.accept(self);
        }

        self.code += "\n";
        self.code += "int main() {\n";
        self.code += "    return matcha_init();\n";
        self.code += "}\n";

        TypeOption::None
    }

    fn visit_import(&mut self, import: &Import) -> TypeOption {
        TypeOption::None
    }

    fn visit_function(&mut self, function: &Function) -> TypeOption {
        let name = self.name(function.name.clone());

        if function.type_.modifiers.is_extern {
            return TypeOption::None;
        }
        self.current_function = name.clone();

        self.code += &(type_to_string(&function.type_, true));
        self.code += " MATCHA__";

        if self.cur_mod_name.is_some() {
            self.code += &self.cur_mod_name.clone().unwrap();
            self.code += "__MATCHA__";
        }

        self.code += &name.clone();
        self.code += "(";

        if function.is_method && !function.type_.modifiers.is_static {
            self.code += &("MATCHA__".to_string() + &self.name(function.obj_name.clone().unwrap()));
            self.code += &(" *MATCHA__".to_string() + &self.name(function.obj_ref_name.clone().unwrap()));

            self.ptr_access.push(self.name(function.obj_ref_name.clone().unwrap()));
        }
        let mut c = 0;
        for param in function.parameters.clone() {
            c += 1;

            self.code += &(type_to_string(&param.type_, true));
            self.code += &(" MATCHA__".to_string() + &self.name(param.name));

            if c < function.parameters.len() {
                self.code += ", ";
            }
        }

        self.code += ") ";

        function.body.accept(self);

        if function.is_method {
            self.ptr_access.pop();
        }
        self.code += "\n";

        TypeOption::None
    }

    fn visit_variable(&mut self, variable: &Variable) -> TypeOption {
        if variable.type_.modifiers.is_extern {
            return TypeOption::None;
        }

        if variable.type_.modifiers.is_const {
            self.code += "const ";
        }
        self.code += &(type_to_string(&variable.type_, false));
        self.code += " MATCHA__";

        if self.cur_mod_name.is_some() {
            self.code += &self.cur_mod_name.clone().unwrap();
            self.code += "__MATCHA__";
        }

        self.code += &self.name(variable.name.clone());

        let mut t = variable.type_.clone();

        while t.get_array_type().is_some() {
            t = t.get_array_type().unwrap().clone();
            self.code += "[]";
        }

        if let Some(val) = &variable.value {
            self.code += " = ";

            val.accept(self);
        }

        TypeOption::None
    }

    fn visit_expression(&mut self, expr: &Expression) -> TypeOption {
        match expr.kind {
            ExpressionKind::Binary(ref binary) => binary.accept(self),
            ExpressionKind::Unary(ref unary) => unary.accept(self),
            ExpressionKind::Literal(ref literal) => literal.accept(self),
            ExpressionKind::Identifier(ref id) => id.accept(self),
            ExpressionKind::Call(ref call) => call.accept(self),
            ExpressionKind::Grouping(ref group) => group.accept(self),
            ExpressionKind::Assignment(ref assign) => assign.accept(self),
            ExpressionKind::StructInit(ref struct_init) => struct_init.accept(self),
            ExpressionKind::Cast(ref cast) => cast.accept(self),
            ExpressionKind::Get(ref get) => get.accept(self),
            ExpressionKind::Set(ref set) => set.accept(self),
            ExpressionKind::Array(ref array) => array.accept(self),
            ExpressionKind::Index(ref index) => index.accept(self),
            _ => TypeOption::None,
        };

        TypeOption::None
    }

    fn visit_statement(&mut self, statement: &Statement) -> TypeOption {
        match statement.kind {
            StatementKind::Variable(ref variable) => {
                variable.accept(self);
                self.code += ";\n";
            }
            StatementKind::Function(ref function) => {
                function.accept(self);
            },
            StatementKind::Expression(ref expression) => {
                expression.accept(self);
                self.code += ";\n";
            },
            StatementKind::Return(ref return_) => {
                return_.accept(self);
                self.code += ";\n";
            },
            StatementKind::Block(ref block) => {
                block.accept(self);
            },
            StatementKind::Struct(ref struct_) => {
                struct_.accept(self);
                self.code += ";\n";
            },
            StatementKind::Enum(ref enum_) => {
                enum_.accept(self);
                self.code += ";\n";
            },
            StatementKind::If(ref if_) => {
                if_.accept(self);
            },
            StatementKind::While(ref while_) => {
                while_.accept(self);
            },
            StatementKind::For(ref for_) => {
                for_.accept(self);
            },
            StatementKind::Break(ref break_) => {
                break_.accept(self);
                self.code += ";\n";
            },
            StatementKind::Continue(ref continue_) => {
                continue_.accept(self);
                self.code += ";\n";
            },
            StatementKind::Export(ref export) => {
                export.accept(self);
            },
            StatementKind::Import(ref import) => {
                import.accept(self);
            },
        };

        TypeOption::None
    }

    fn visit_return(&mut self, return_: &Return) -> TypeOption {
        self.code += "return ";
        if let Some(value) = &return_.value {
            if self.cur_mod_name.is_some() {
                self.code += "MATCHA__";
                self.code += &self.cur_mod_name.clone().unwrap();
                self.code += "__";
            }
            value.accept(self);
        }

        TypeOption::None
    }

    fn visit_block(&mut self, block: &Block) -> TypeOption {
        self.code += "{\n";

        for statement in block.statements.clone() {
            statement.accept(self);
        }

        self.code += "}\n";

        TypeOption::None
    }

    fn visit_struct(&mut self, struct_: &Struct) -> TypeOption {
        let name = self.name(struct_.name.clone());
        self.code += "typedef struct ";
        self.code += "MATCHA__";

        if self.cur_mod_name.is_some() {
            self.code += &self.cur_mod_name.clone().unwrap();
            self.code += "__MATCHA__";
        }

        self.code += &name.clone();
        self.code += " {\n";

        for field in struct_.fields.clone() {
            self.code += &(type_to_string(&field.type_, true));
            self.code += &(" MATCHA__".to_string() + &self.name(field.name));
            self.code += ";\n";
        }

        self.code += "} ";
        self.code += "MATCHA__";

        if self.cur_mod_name.is_some() {
            self.code += &self.cur_mod_name.clone().unwrap();
            self.code += "__MATCHA__";
        }

        self.code += &name.clone();

        TypeOption::None
    }

    fn visit_enum(&mut self, enum_: &Enum) -> TypeOption {
        let name = self.name(enum_.name.clone());
        self.code += "typedef enum ";
        self.code += "MATCHA__";
        
        if self.cur_mod_name.is_some() {
            self.code += &self.cur_mod_name.clone().unwrap();
            self.code += "__MATCHA__";
        }

        self.code += &name.clone();
        self.code += " {\n";

        for variant in enum_.variants.clone() {
            self.code += "MATCHA__";
            self.code += &self.name(variant.name);
            self.code += ",\n";
        }

        self.code += "} ";
        self.code += "MATCHA__";

        if self.cur_mod_name.is_some() {
            self.code += &self.cur_mod_name.clone().unwrap();
            self.code += "__MATCHA__";
        }

        self.code += &name.clone();

        TypeOption::None
    }

    fn visit_if(&mut self, if_: &If) -> TypeOption {
        self.code += "if (";
        if_.condition.accept(self);
        self.code += ") ";
        if_.then_branch.accept(self);

        if let Some(ref else_branch) = if_.else_branch {
            self.code += " else ";
            else_branch.accept(self);
        }

        TypeOption::None
    }

    fn visit_while(&mut self, while_: &While) -> TypeOption {
        self.code += "while (";
        while_.condition.accept(self);
        self.code += ") ";
        while_.body.accept(self);

        TypeOption::None
    }

    fn visit_for(&mut self, for_: &For) -> TypeOption {
        self.code += "for (";

        if let Some(ref initializer) = for_.initializer {
            initializer.accept(self);
        }


        if let Some(ref condition) = for_.condition {
            condition.accept(self);
        }
        self.code += "; ";

        if let Some(ref increment) = for_.increment {
            increment.accept(self);
        }
        self.code += ") ";
        for_.body.accept(self);

        TypeOption::None
    }

    fn visit_break(&mut self, _break: &Break) -> TypeOption {
        self.code += "break";

        TypeOption::None
    }

    fn visit_continue(&mut self, _continue: &Continue) -> TypeOption {
        self.code += "continue";

        TypeOption::None
    }

    fn visit_export(&mut self, _export: &Export) -> TypeOption {
        TypeOption::None
    }

    fn visit_call(&mut self, call: &Call) -> TypeOption {
        let name = self.name(*call.callee.clone());
        
        if self.current_call_object.len() > 0 {
            let obj = self.current_call_object.pop().unwrap();
            let obj_name = self.name(obj.clone());

            self.code += "MATCHA__";
            self.code += &name.clone();
            self.code += "(";
            self.code += "&MATCHA__";
            self.code += &obj_name.clone();

            if call.args.len() > 0 {
                self.code += ", ";
            }
        } else {
            let t_sym = self.symtable.current_mut().find(name.clone());

            if let Some(id) = t_sym {
                let symbol = self.symtable.current_mut().get_function(id.id).unwrap();
    
                if !symbol.type_.modifiers.is_extern {
                    self.code += "MATCHA__";
                }
            }
    
            self.code += &name.clone();
            self.code += "(";
        }

        let mut c = 0;
        for arg in call.args.clone() {
            c += 1;
            arg.accept(self);

            if c < call.args.len() {
            self.code += ", ";
            }
        }

        self.code += ")";

        TypeOption::None
    }

    fn visit_literal(&mut self, literal: &Literal) -> TypeOption {
        match literal.value.kind {
            TokenType::Integer => {
                self.code += &literal.value.lexeme;
            }
            TokenType::Float => {
                self.code += &literal.value.lexeme;
            }
            TokenType::String => {
                self.code += "\"";
                self.code += &literal.value.lexeme;
                self.code += "\"";
            }
            TokenType::Char => {
                self.code += "'";
                self.code += &literal.value.lexeme;
                self.code += "'";
            }
            TokenType::True => {
                self.code += "true";
            }
            TokenType::False => {
                self.code += "false";
            }
            _ => {}
        }

        TypeOption::None
    }

    fn visit_identifier(&mut self, id: &Identifier) -> TypeOption {
        let symbol = self.symtable.current_mut().find(id.name.lexeme.clone());

        if let Some(s) = symbol {
            match s.kind {
                SymbolKind::Constant(c) | SymbolKind::Variable(c) => {
                    if !c.type_.modifiers.is_extern {
                        self.code += "MATCHA__";
                    }
                    self.code += &id.name.lexeme;
    
                    return TypeOption::None;
                }
                _ => {}
            }
        }
        
        self.code += "MATCHA__";
        self.code += &id.name.lexeme;

        TypeOption::None
    }

    fn visit_binary(&mut self, binary: &Binary) -> TypeOption {
        self.code += "(";
        binary.left.accept(self);
        self.code += " ";

        self.code += &binary.operator.lexeme;

        self.code += " ";
        binary.right.accept(self);
        self.code += ")";

        TypeOption::None
    }

    fn visit_unary(&mut self, unary: &Unary) -> TypeOption {
        if unary.is_prefix {
            self.code += &unary.operator.lexeme;
            unary.right.accept(self);
        } else {
            unary.right.accept(self);
            self.code += &unary.operator.lexeme;
        }

        TypeOption::None
    }

    fn visit_assignment(&mut self, assign: &Assignment) -> TypeOption {
        assign.left.accept(self);
        self.code += &assign.operator.lexeme;
        assign.right.accept(self);

        self.code += ";\n";

        TypeOption::None
    }

    fn visit_grouping(&mut self, group: &Grouping) -> TypeOption {
        self.code += "(";
        group.expression.accept(self);
        self.code += ")";

        TypeOption::None
    }

    fn visit_type(&mut self, _type: &Type) -> TypeOption {
        TypeOption::None
    }

    fn visit_array(&mut self, array: &Array) -> TypeOption {
        self.code += "{";

        let mut c = 0;
        for element in array.elements.clone() {
            c += 1;

            element.accept(self);

            if c < array.elements.len() {
                self.code += ", ";
            }
        }

        self.code += "}";
        TypeOption::None
    }

    fn visit_index(&mut self, index: &Index) -> TypeOption {
        index.target.accept(self);
        self.code += "[";
        index.index.accept(self);
        self.code += "]";

        TypeOption::None
    }

    fn visit_struct_init(&mut self, struct_init: &StructInit) -> TypeOption {
        self.code += "{";

        let mut c = 0;
        for field in struct_init.fields.clone() {
            c += 1;
            self.code += ".MATCHA__";
            self.code += &field.0.lexeme;
            self.code += " = ";
            field.1.accept(self);

            if c < struct_init.fields.len() {
                self.code += ", ";
            }
        }

        self.code += "}";

        TypeOption::None
    }

    fn visit_get(&mut self, get: &Get) -> TypeOption {
        let mut is_module = false;
        match &get.object.kind {
            ExpressionKind::Identifier(ident) => {
                // check if the identifier is a module
                let x = self.symtable.current_mut().get_module(ident.name.lexeme.clone());

                if x.is_some() {
                    is_module = true;
                }
            }
            _ => {}
        }

        match &get.name.kind {
            ExpressionKind::Call(c) => {
                if !is_module {
                    let name = self.name(*get.object.clone());
                    let symbol = self.symtable.current_mut().find(name.clone());
    
                    if symbol.is_none() {
                        self.current_call_object.push(*get.object.clone());
                        c.accept(self);
                        return TypeOption::None;
                    }
    
                    let kind = symbol.unwrap().kind;
                    let mut func: Option<Function> = None;
    
                    match kind {
                        SymbolKind::Struct(s) => {
                            for m in s.methods.clone() {
                                if self.name(m.name.clone()) == self.name(*c.callee.clone()) {
                                    func = Some(m);
                                    break;
                                }
                            }
                        }
                        SymbolKind::Enum(e) => {
                            for m in e.methods.clone() {
                                if self.name(m.name.clone()) == self.name(*c.callee.clone()) {
                                    func = Some(m);
                                    break;
                                }
                            }
                        }
                        _ => unreachable!(),
                    }
    
                    if !func.unwrap().type_.modifiers.is_static {
                        self.current_call_object.push(*get.object.clone());
                    }
                }
                
                c.accept(self);
            }
            ExpressionKind::StructInit(_) => {
                get.name.accept(self);
            }
            _ => {
                let name = self.name(*get.object.clone());

                get.object.accept(self);

                if is_module {
                    self.code += "__";
                } else {
                    if self.ptr_access.contains(&name) {
                        self.code += "->";
                    } else {
                        self.code += ".";
                    }
                }
                get.name.accept(self);
            }
        }

        TypeOption::None
    }

    fn visit_set(&mut self, set: &Set) -> TypeOption {
        set.object.accept(self);
        self.code += ".";
        set.name.accept(self);
        self.code += " = ";
        set.value.accept(self);

        TypeOption::None
    }

    fn visit_cast(&mut self, cast: &Cast) -> TypeOption {
        self.code += "(";

        self.code += &(type_to_string(&cast.type_, false));
        self.code += ")";
        cast.value.accept(self);

        TypeOption::None
    }
}