use std::{fmt, path::PathBuf, sync::{Arc, Mutex, RwLock}};
use crate::{
    ast::{Ast, AstNode, AstNodeRef},
    common::{LibraryPath, OperatorTable, PrecedenceDAG, Scope, SymbolTable},
    error_warning::ErrorCode
};

mod context_setup;
pub use context_setup::*;

mod item_scope_pass;
pub use item_scope_pass::*;

mod module_attrib_resolution;
pub use module_attrib_resolution::*;

mod module_symbol_generation;
pub use module_symbol_generation::*;

mod precedence_passes;
pub use precedence_passes::*;

mod operator_passes;
pub use operator_passes::*;


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
    sym_path: Scope, //< Not really a scope, but good enough for now
}

pub enum ContextNodeData {
    None,
    Module(ModuleContextData),
    Precedence(u16),
    Infix {
        reorder: bool,
    },
}

pub struct ContextNode {
    pub scope: Scope,
    pub data: ContextNodeData,
}

impl ContextNode {
    fn new() -> Self {
        Self {
            scope: Scope::new(),
            data: ContextNodeData::None,
        }
    }
}

pub struct Context {
    pub lib_path: LibraryPath,
    pub errors:   Mutex<Vec<AstError>>,
    ctxs:         Vec<ContextNode>,
    syms:         Arc<RwLock<SymbolTable>>,
    mod_root:     Scope,
    precedences:  Arc<RwLock<PrecedenceDAG>>,
    operators:    Arc<RwLock<OperatorTable>>,
}

impl Context {
    pub fn new(
        lib_path: LibraryPath,
        syms: Arc<RwLock<SymbolTable>>,
        mod_root: Scope,
        ast: &Ast,
        precedences: Arc<RwLock<PrecedenceDAG>>,
        operators: Arc<RwLock<OperatorTable>>
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
            operators,
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

    pub fn add_error(&self, err: AstError) {
        self.errors.lock().unwrap().push(err);
    }
}