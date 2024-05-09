// Ethan Olchik
// src/utils/imports.rs

//> Imports

use crate::{
    ast::ast::{
        Expression,
        ExpressionKind,
        Import,
    },
    utils::{
        module::{
            MatchaModule,
            DependencyTable
        },
        compile::parse
    }
};

use std::fs;

//> Definitions

const MATCHA_EXT: &str = ".mt";

/// ImportHandler(filename) struct<br>
/// This struct holds all the imports in a file and is used to resolve them.
pub struct ImportHandler {
    pub unresolved: Vec<Import>,
    pub resolved: Vec<MatchaModule>,

    pub dependency_table: DependencyTable,

    filename: String
}

//> Implementations

impl ImportHandler {
    pub fn new(filename: String) -> Self {
        Self {
            unresolved: Vec::new(),
            resolved: Vec::new(),
            dependency_table: DependencyTable::new(),
            filename
        }
    }

    pub fn add_import(&mut self, path: Import) {
        self.unresolved.push(path);
    }

    pub fn resolve_all(&mut self) {
        for i in 0..self.unresolved.len() {
            self.resolve_idx(i);
        }
    }

    pub fn resolve_idx(&mut self, idx: usize) {
        let import = &self.unresolved[idx];
        let module = self.resolve_import(import.clone(), &mut vec![]);

        self.dependency_table.push_module(module.clone());
        self.resolved.push(module);
    }

    /// Takes an import AST, breaks it down into a path, and then resolves it into a MatchaModule.<br>
    /// e.g.<br>
    /// In: Import { .., path: Identifier { 'std.net.http' }}, cycle: [[...]] (keeps track of paths to detect cyclic imports) <br> 
    /// Out: MatchaModule { name: 'http', dependencies: [[...]], path: std/net/http.mt, exported_symbols: [[...]] }
    // TODO: allow std.net.http.{Request}, std.net.http.{Request, Response}
    fn resolve_import(&self, import: Import, cycle: &mut Vec<String>) -> MatchaModule {
        // First, we need to get the path of the file, then we need to use that to build a MatchaModule.
        let path = self.modulename_to_path(import.path.clone());
        if cycle.contains(&path.0.clone()) {
            panic!("Cyclic dependency.")
        }
        cycle.push(path.0.clone());

        let name: String;

        if import.alias.is_some() {
            name = import.alias.unwrap()
        } else {
            match import.path.kind {
                ExpressionKind::Identifier(id) => {
                    name = id.name.lexeme.clone().split(".").last().unwrap().to_string();
                }
                _ => unreachable!()
            };
        }

        let mut module = MatchaModule {
            name,
            dependencies: Vec::new(),
            path: path.clone().0,
            exported_symbols: Vec::new()
        };

        // Now we need to parse the file and get the exported symbols
        if path.1 {
            // If the path is a directory, we need to get all the files in the directory
            // and resolve them as modules.

            let files = fs::read_dir(&path.0).unwrap();

            for file in files {
                let file = file.unwrap();
                let file_name = file.file_name().into_string().unwrap();

                if file_name.ends_with(MATCHA_EXT) {
                    let file_path = file.path().to_str().unwrap().to_string();
                    let program = parse(file_path).unwrap();

                    for i in program.imports {
                        module.dependencies.push(self.resolve_import(i, cycle));
                    }
                }
            }
        } else {
            // If the path is a file, we can just parse it and get the dependencies.

            let program = parse(path.clone().0).unwrap();

            for i in program.imports {
                module.dependencies.push(self.resolve_import(i, cycle));
            }

            for s in program.statements {
                todo!();
            }
        }

        module
    }

    /// modulename_to_path -> (path, is_dir)
    fn modulename_to_path(&self, expr: Expression) -> (String, bool) {
        let name = match expr.kind {
            ExpressionKind::Identifier(id) => id.name.lexeme,
            _ => unreachable!()
        };

        // e.g. std.net.http -> [std, net, http]
        let sections = name.split(".").collect::<Vec<&str>>();

        match sections.len() {
            0 => {
                unreachable!()
            }
            _ => {
                // In Matcha, A MatchaModule is a directory or file with the same name as the MatchaModule.
                // e.g. std.net.http -> std/net/http.mt
                // However, what if std.net.http is a directory?
                // E.g.
                // std/
                //  net/
                //   http/
                //    requests.mt
                //    ...
                // In this case, we need to get all of the files in the directory std/net/http/
                // and check if there is a file with the same name as the directory, 'http.mt'.
                // If there is, we use that file. If there isn't, we use the directory.
                // If there is a 'http.mt' file in std/net as well as a directory 'http/',
                // the directory takes priority (Folders always appear before files in the filesystem).
                
                let path = format!("{}/{}", self.filename, sections.join("/"));

                if fs::metadata(&path).unwrap().is_dir() {
                    let files = fs::read_dir(&path).unwrap();

                    let mut found = false;
                    let mut found_path = String::new();

                    for file in files {
                        let file = file.unwrap();
                        let file_name = file.file_name().into_string().unwrap();

                        if file_name == format!("{}{}", sections.last().unwrap(), MATCHA_EXT) {
                            found = true;
                            found_path = file.path().to_str().unwrap().to_string();
                            break;
                        }
                    }

                    if found {
                        // e.g. std/net/http/http.mt
                        return (found_path, false);
                    } else {
                        // e.g. std/net/http
                        return (path, true);
                    }
                } else {
                    // e.g. std/net/http.mt
                    return (format!("{}{}", path, MATCHA_EXT), false);
                }
            }
        }
    }

    fn get_stdlib_path(&self) -> String {
        // TODO: Generalise this.
        return String::from("C:/Users/OLCHIK/Matcha/std");
    }
}