use crate::{ast::*, common::{NameTable, PrecedenceImportPath, Symbol}, error_warning::ErrorCode};

use super::{AstError, Context};




pub struct PrecedenceCollection<'a> {
    ctx:     &'a Context,
    names:   &'a NameTable,
}

impl<'a> PrecedenceCollection<'a> {
    pub fn new(ctx: &'a mut Context, names: &'a NameTable) -> Self {
        Self {
            ctx,
            names,
        }
    }
}

impl Visitor for PrecedenceCollection<'_> {
    fn visit_precedence(&mut self, ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        let ctx_node = self.ctx.get_node_for(node_id);
        let scope = &ctx_node.scope;

        let node = &ast[node_id];
        let name = self.names[node.name].to_string();

        let id = {
            let mut precedences = self.ctx.precedences.write().unwrap();
            precedences.add_precedence(name.to_string())
        };

        let mut syms = self.ctx.syms.write().unwrap();
        syms.add_precedence(&scope, name, id);
    }
}

pub struct PrecedenceImportCollection<'a> {
    ctx:         &'a Context,
    names:       &'a NameTable,
    package:     String,
    top_level:   bool,
    pub imports: Vec<PrecedenceImportPath>
}

impl<'a> PrecedenceImportCollection<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable, package: String) -> Self {
        let top_level = ctx.mod_root.is_empty();
        Self {
            ctx,
            names,
            package,
            top_level,
            imports: Vec::new(),
        }
    }
}

impl Visitor for PrecedenceImportCollection<'_> {
    fn visit_module(&mut self, ast: &Ast, node_id: AstNodeRef<ModuleItem>) where Self: Sized {
        self.top_level = false;
        helpers::visit_module(self, ast, node_id);
    }

    fn visit_precedence_use(&mut self, ast: &Ast, node_id: AstNodeRef<PrecedenceUse>) where Self: Sized {
        if !self.top_level {
            let scope = &self.ctx.get_node_for(node_id).scope;

            let mut path = String::new();
            for (idx, segment) in scope.iter().enumerate() {
                if idx != 0 {
                    path.push('.');
                }
                path.push_str(segment);
            }

            self.ctx.add_error(AstError {
                node_id: node_id.index(),
                err: ErrorCode::AstNotTopLevel { 
                    path,
                    info: "Precedence use".to_string(),
                 }
            });
            return;
        }

        let node = &ast[node_id];
        
        let group = node.group.map(|group| self.names[group].to_string());
        let package = match node.package {
            Some(package) => self.names[package].to_string(),
            None          => self.package.clone(),
        };
        let library = match node.library {
            Some(library) => self.names[library].to_string(),
            None => package.clone(),
        };

        for precedence in &node.precedences {
            let name = self.names[*precedence].to_string();

            let import_path = PrecedenceImportPath::new(group.clone(), package.clone(), library.clone(), name.clone());
            self.imports.push(import_path);
            self.ctx.precedences.write().unwrap().add_precedence(name);
        }
    }
}

pub struct PrecedenceConnection<'a> {
    ctx:   &'a Context,
    names: &'a NameTable,
}

impl<'a> PrecedenceConnection<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable) -> Self {
        Self {
            ctx,
            names,
        }
    }
}

impl Visitor for PrecedenceConnection<'_> {
    fn visit_precedence(&mut self, ast: &Ast, node_id: AstNodeRef<Precedence>) where Self: Sized {
        let node = &ast[node_id];
        let name = &self.names[node.name];

        let ctx_node = self.ctx.get_node_for(node_id);

        let syms = self.ctx.syms.read().unwrap();
        let sym = syms.get_symbol(&ctx_node.scope, name).unwrap();
        let Symbol::Precedence(sym) = sym else {
            self.ctx.add_error(AstError {
                node_id: node_id.index(),
                err: ErrorCode::InternalError("Expected Precedence symbol when accessing symbol associated to a precedencenode ")
            });
            return;
        };

        let mut precedence_dag = self.ctx.precedences.write().unwrap();

        if let Some(lower) = &node.lower_than {
            let lower_id = precedence_dag.get_id(&self.names[*lower]);
            precedence_dag.set_order(lower_id, sym.id);
        }

        if let Some(higher) = &node.higher_than {
            let higher_id = precedence_dag.get_id(&self.names[*higher]);
            precedence_dag.set_order(sym.id, higher_id);
        }
    }
}