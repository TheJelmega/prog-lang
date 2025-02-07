use crate::{
    ast::*,
    common::{NameTable, OperatorImportPath},
    error_warning::AstErrorCode,
};

use super::{AstError, Context};

pub struct OperatorImport<'a> {
    ctx:         &'a Context,
    names:       &'a NameTable,
    top_level:   bool,
    pub imports: Vec<OperatorImportPath>,
}

impl<'a> OperatorImport<'a> {
    pub fn new(ctx: &'a Context, names: &'a NameTable) -> Self {
        let top_level = ctx.mod_root.is_empty();
        Self {
            ctx,
            names,
            top_level,
            imports: Vec::new(),
        }
    }
}

impl Visitor for OperatorImport<'_> {
    fn visit_module(&mut self, node: &AstNodeRef<ModuleItem>) where Self: Sized {
        self.top_level = false;
        helpers::visit_module(self, node);
    }

    fn visit_op_use(&mut self, node: &AstNodeRef<OpUse>) where Self: Sized {
        if !self.top_level {
            let scope = &self.ctx.get_node_for(node).scope;
            let path = scope.to_string();

            self.ctx.add_error(AstError {
                node_id: node.node_id(),
                err: AstErrorCode::NotTopLevel { 
                    path,
                    info: "Operator use".to_string(),
                 }
            });
            return;
        }

        let group = node.group.map(|group| self.names[group].to_string());
        let package = match node.package {
            Some(package) => self.names[package].to_string(),
            None          => self.ctx.lib_path.package.clone()
        };
        let library = match node.library {
            Some(library) => self.names[library].to_string(),
            None          => package.clone(),
        };

        for op in &node.operators {
            let import_path = OperatorImportPath {
                group: group.clone(),
                package: package.clone(),
                library: library.clone(),
                op: *op,
            };
            self.imports.push(import_path);
        }
    }
}