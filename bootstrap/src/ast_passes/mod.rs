use std::{fmt, path::PathBuf, sync::{Arc, RwLock}};
use crate::{
    ast::{Ast, AstNode, AstNodeRef},
    common::SymbolTable,
    error_warning::ErrorCode
};

mod context_setup;
pub use context_setup::*;

mod module_scope_pass;
pub use module_scope_pass::*;

mod module_attrib_resolution;
pub use module_attrib_resolution::*;

mod module_symbol_generation;
pub use module_symbol_generation::*;

pub struct AstError {
    node_id: usize,
    err:     ErrorCode,
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

pub struct ModuleContextData {
    path:     Option<PathBuf>,
    sym_path: Vec<String>,
    sym_idx:  usize,
}

pub enum ContextNodeData {
    None,
    Module(ModuleContextData),
}

pub struct ContextNode {
    pub scope: Vec<String>,
    pub data: ContextNodeData,
}

impl ContextNode {
    fn new() -> Self {
        Self {
            scope: Vec::new(),
            data: ContextNodeData::None,
        }
    }
}

pub struct Context {
    pub errors:   Vec<AstError>,
    ctxs:         Vec<ContextNode>,
    syms:         Arc<RwLock<SymbolTable>>,
    mod_root:     Vec<String>,
}

impl Context {
    pub fn new(syms: Arc<RwLock<SymbolTable>>, mod_root: Vec<String>, ast: &Ast) -> Self {
        let mut ctxs = Vec::with_capacity(ast.nodes.len());
        ctxs.resize_with(ast.nodes.len(), || ContextNode::new());

        Self {
            errors: Vec::new(),
            ctxs,
            syms,
            mod_root,
        }
    }

    pub fn get_node_for_index(&self, id: usize) -> &ContextNode {
        assert!(id < self.ctxs.len());
        &self.ctxs[id]
    }

    pub fn get_node_for<T>(&self, id: AstNodeRef<T>) -> &ContextNode {
        self.get_node_for_index(id.index())
    }

    
    pub fn get_node_for_index_mut(&mut self, id: usize) -> &mut ContextNode {
        assert!(id < self.ctxs.len());
        &mut self.ctxs[id]
    }

    pub fn get_node_for_mut<T>(&mut self, id: AstNodeRef<T>) -> &mut ContextNode {
        self.get_node_for_index_mut(id.index())
    }
}