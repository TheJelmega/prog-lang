use std::path::PathBuf;

use crate::{
    ast::*,
    common::{NameTable, Scope, Symbol},
    error_warning::AstErrorCode
};

use super::{AstError, Context, ContextNodeData};

pub struct ModulePathResolution<'a> {
    ctx:             &'a mut Context,
    names:           &'a NameTable,
    base_path:       PathBuf,
    path_stack:      Vec<PathBuf>,
    pub collected_paths: Vec<(PathBuf, Scope)>,
}

impl<'a> ModulePathResolution<'a> {
    pub fn new(ctx: &'a mut Context, names: &'a NameTable, base_path: PathBuf) -> Self {
        Self {
            ctx,
            names,
            base_path,
            path_stack: Vec::new(),
            collected_paths: Vec::new(),
        }
    }
}

impl Visitor for ModulePathResolution<'_> {

    fn visit_item(&mut self, item: &Item) where Self: Sized {
        match item {
            Item::Module(module) => self.visit_module(module),
            _ => {},
        }
    }

    fn visit_module(&mut self, node: &AstNodeRef<ModuleItem>) where Self: Sized {
        let attr_path = {   
            let ctx_node = &mut self.ctx.get_node_for(node);
            let ContextNodeData::Module(module_data) = &ctx_node.data else { unreachable!() }; 
            match &module_data.path {
                Some(path) => Some(path.clone()),
                None => None,
            }
        };

        
        let mut rel_path = match self.path_stack.last() {
            Some(path) => path.clone(),
            None => PathBuf::new(),
        };

        let mod_name = &self.names[node.name];

        match attr_path {
            Some(path) => {
                for comp in path.components() {
                    match comp {
                        std::path::Component::Prefix(_) => self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidAttributeData { info: format!("Module path attributes may not contain a root") },
                        }),
                        std::path::Component::RootDir => self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidAttributeData { info: format!("Module path attributes may not contain a root") },
                        }),
                        std::path::Component::CurDir => {},
                        std::path::Component::ParentDir => { rel_path.pop(); },
                        std::path::Component::Normal(sub_path) => rel_path.push(sub_path),
                    }
                }
            },
            None => {},
        }

        let mut path = self.base_path.to_path_buf();
        path.push(rel_path.clone());

        if node.block.is_some() {
            self.path_stack.push(path);
            helpers::visit_module(self, node);
            self.path_stack.pop();
            return;
        }

        if rel_path.as_mut_os_str().is_empty() {
            // Check if the path is mod path, meaning:
            // - path is the main 'lib.xn' or 'main.xn' file
            // - path is a 'mod.xn' file
            let ctx_node = self.ctx.get_node_for(node);
            let is_mod_path = if ctx_node.scope.is_empty() {
                true
            } else {
                let syms = self.ctx.syms.read();
                let base_scope = &ctx_node.scope.parent();
                let cur_name = &ctx_node.scope.last().unwrap().name;

                let Some(sym) = syms.get_symbol(None, base_scope, cur_name) else {
                    self.ctx.add_error(AstError {
                        node_id: node.node_id(),
                        err: AstErrorCode::InvalidAttributeData { info: format!("Module path attributes may not contain a root") },
                    });
                    return
                };

                let Symbol::Module(mod_sym) = &*sym.read() else {
                    self.ctx.add_error(AstError {
                        node_id: node.node_id(),
                        err: AstErrorCode::InvalidAttributeData { info: format!("Module path attributes may not contain a root") },
                    });
                    return;
                };

                 mod_sym.path.ends_with("mod.xn")
            };

            // if a mod 'bar' is defined inside of 'foo', it can be in any of the following locations
            // - 'foo/bar.xn'
            // - 'foo/bar/mod.xn'
            // - 'bar.xn' if foo is a 'mod.xn' file, a 'lib.xn' root, or a 'main.xn' root (mod path)

            path.push(mod_name);

            let mut err_paths = Vec::new();
            if is_mod_path {
                path.set_extension("xn");
                if !path.is_file() {
                    err_paths.push(path.to_str().unwrap().to_string());
                    path.set_extension("");


                    path.push("mod.xn");   

                    if !path.is_file() {
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidModulePath { paths: err_paths },
                        });
                        return;
                    }
                }
            } else {
                path.push(mod_name);
                path.set_extension("xn");
                if !path.is_file() {
                    err_paths.push(path.to_str().unwrap().to_string());
                    path.set_extension("");
                    path.push("mod.xn");
                    if !path.is_file() {
                        err_paths.push(path.to_str().unwrap().to_string());
                        self.ctx.add_error(AstError {
                            node_id: node.node_id(),
                            err: AstErrorCode::InvalidModulePath { paths: err_paths },
                        });
                        return;
                    }
                }
            }
        }

        let mod_name = self.names[node.name].to_string();
        {
            let ctx_node = self.ctx.get_node_for_mut(node);
            let ContextNodeData::Module(module_data) = &mut ctx_node.data else { unreachable!() };
            module_data.sym_path = ctx_node.scope.clone();
            module_data.sym_path.push(mod_name.to_string());
        }
        let ctx_node = self.ctx.get_node_for(node);
        
        let mut base_scope = ctx_node.scope.clone();
        base_scope.push(mod_name.clone());
        
        self.ctx.syms.write().add_module(&ctx_node.scope, mod_name.to_string(), path.clone());
        self.collected_paths.push((path.clone(), base_scope));
    }
}