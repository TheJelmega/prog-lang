use std::{fmt, path::PathBuf, sync::Arc};
use crate::{
    ast::{Ast, AstNodeRef, AstNode, NodeId},
    common::{LibraryPath, PrecedenceDAG, RootSymbolTable, Scope},
    error_warning::AstErrorCode
};
use parking_lot::{Mutex, RwLock};

mod context_setup;
pub use context_setup::*;

mod item_scope_pass;
pub use item_scope_pass::*;

mod module_attrib_resolution;
pub use module_attrib_resolution::*;

mod module_symbol_generation;
pub use module_symbol_generation::*;

mod hir_lower;
pub use hir_lower::*;

pub struct AstError {
    node_id: NodeId,
    err:     AstErrorCode,
}

impl fmt::Display for AstError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.err)
    }
}

pub struct ModuleContextData {
    path:     Option<PathBuf>,
    sym_path: Scope, //< Not really a scope, but good enough for now
}

pub enum ContextNodeData {
    None,
    Module(ModuleContextData),
}

pub struct ContextNode {
    pub module_scope: Scope, // TODO: rename module_scope to actually mentioned what this scope actually is, so this is not used incorrectly when lowering
    pub data:         ContextNodeData,
}

impl ContextNode {
    fn new() -> Self {
        Self {
            module_scope: Scope::new(),
            data: ContextNodeData::None,
        }
    }
}

pub struct Context {
    pub lib_path: LibraryPath,
    pub errors:   Mutex<Vec<AstError>>,
    ctxs:         Vec<ContextNode>,
    syms:         Arc<RwLock<RootSymbolTable>>,
    mod_root:     Scope,
    precedences:  Arc<RwLock<PrecedenceDAG>>,
}

impl Context {
    pub fn new(
        lib_path: LibraryPath,
        syms: Arc<RwLock<RootSymbolTable>>,
        mod_root: Scope,
        ast: &Ast,
        precedences: Arc<RwLock<PrecedenceDAG>>,
    ) -> Self {
        let mut ctxs = Vec::with_capacity(ast.nodes.len());
        ctxs.resize_with(ast.nodes.len(), || ContextNode::new());

        Self {
            lib_path,
            errors: Mutex::new(Vec::new()),
            ctxs,
            syms,
            mod_root,
            precedences,
        }
    }

    pub fn get_node_for_index(&self, id: usize) -> &ContextNode {
        assert!(id < self.ctxs.len());
        &self.ctxs[id]
    }

    pub fn get_node_for<T: AstNode>(&self, node: &AstNodeRef<T>) -> &ContextNode {
        self.get_node_for_index(node.node_id().index())
    }

     
    pub fn get_node_for_index_mut(&mut self, id: usize) -> &mut ContextNode {
        assert!(id < self.ctxs.len());
        &mut self.ctxs[id]
    }

    pub fn get_node_for_mut<T: AstNode>(&mut self, node: &AstNodeRef<T>) -> &mut ContextNode {
        self.get_node_for_index_mut(node.node_id().index())
    }

    pub fn add_error(&self, err: AstError) {
        self.errors.lock().push(err);
    }
}