#![allow(unused)]

use std::sync::Arc;

use parking_lot::RwLock;

use crate::type_system::TypeHandle;

use super::{IndentLogger, Logger, NameId, SpanId, SpanRegistry, SymbolRef};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VarScopeId(usize);

impl VarScopeId {
    pub const PROCESS_INITIAL: VarScopeId = VarScopeId(0);
    pub const INVALID: VarScopeId = VarScopeId(usize::MAX);
}

pub struct VariableScope {
    pub span:   SpanId,
    pub parent: Option<VarScopeId>,
}

pub struct VariableEntry {
    pub scope:       VarScopeId,
    /// Span where variable was declared
    pub decl_span:   SpanId,
    /// Span where variable was shadowed by other variable
    pub shadow_span: Option<SpanId>,
    pub name:        NameId,
    pub is_const:    bool,
    pub is_mut:      bool,
    pub ty:          Option<TypeHandle>,


    debug_name:     String,
}

pub struct VariableInfo {
    sym:     SymbolRef,
    scopes:  Vec<VariableScope>,
    entries: Vec<VariableEntry>,
}

impl VariableInfo {
    pub fn add_scope(&mut self, span: SpanId, parent: Option<VarScopeId>) -> VarScopeId {
        let idx = self.scopes.len();
        self.scopes.push(VariableScope {
            span,
            parent,
        });
        VarScopeId(idx)
    }

    pub fn get_scope(&self, id: VarScopeId) -> &VariableScope {
        assert!(id.0 < self.scopes.len());
        &self.scopes[id.0]
    }

    pub fn add_var(&mut self, scope: VarScopeId, name: NameId, debug_name: String, span: SpanId, is_mut: bool, is_const: bool) {
        self.entries.push(VariableEntry {
            scope,
            decl_span: span,
            shadow_span: None,
            name,
            is_const,
            is_mut,
            ty: None,
            debug_name,
        });
    }

    pub fn get_var(&self, scope_id: VarScopeId, name: NameId, span: SpanId, span_registry: &SpanRegistry) -> Option<&VariableEntry> {
        let span = &span_registry[span];

        // PERF: not the most perfomant as we are going over all entries multiple times
        let mut scope_id_opt = Some(scope_id);
        while let Some(scope_id) = scope_id_opt {
            for var in &self.entries {
                // Same scope
                if var.scope != scope_id {
                    continue;
                }
                // Same name
                if var.name != name {
                    continue;
                }

                // Was it declared before the current scope
                let decl_span = &span_registry[var.decl_span];
                if span < decl_span {
                    continue;
                }

                // Check if it the variable wasn't shadowed yet
                if let Some(shadow_span) = var.shadow_span {
                    let shadow_span = &span_registry[shadow_span];
                    if span > shadow_span {
                        continue;
                    }
                }

                return Some(var)
            }
            scope_id_opt = self.scopes[scope_id.0].parent;
        }

        None
    }

    pub fn log(&self) {
        let mut logger = IndentLogger::new("    ", "    ", "    ");
        self.log_scope(&mut logger, 0);
    }

    fn log_scope(&self, logger: &mut IndentLogger, scope_id: usize) {
        logger.prefixed_log_fmt(format_args!("{scope_id}: {{\n"));
        logger.push_indent();
        for entry in &self.entries {
            if entry.scope.0 == scope_id {
                logger.prefixed_log_fmt(format_args!("{}\n", entry.debug_name))
            }
        }
        for (idx, scope) in self.scopes.iter().enumerate() {
            if let Some(parent) = scope.parent {
                if parent.0 == scope_id {
                    self.log_scope(logger, idx)
                }
            }
        }
        logger.pop_indent();
        logger.prefixed_logln("}");
    }
}

pub type VarInfoHandle = Arc<RwLock<VariableInfo>>;


pub struct VariableInfoScopeBuilder {
    scopes:  Vec<VariableScope>,
}

impl VariableInfoScopeBuilder {
    pub fn new() -> Self {
        Self { scopes: Vec::new() }
    }

    pub fn finalize(self, sym: SymbolRef) -> VariableInfo {
        VariableInfo {
            sym,
            scopes: self.scopes,
            entries: Vec::new(),
        }
    }

    pub fn add_scope(&mut self, span: SpanId, parent: Option<VarScopeId>) -> VarScopeId {
        let idx = self.scopes.len();
        self.scopes.push(VariableScope {
            span,
            parent,
        });
        VarScopeId(idx)
    }
}

impl Default for VariableInfoScopeBuilder {
    fn default() -> Self {
        Self::new()
    }
}


#[derive(Clone, Copy, PartialEq, Eq)]
pub struct VarInfoId(usize);

impl VarInfoId {
    pub const INVALID: VarInfoId = VarInfoId(usize::MAX);
}

pub struct VarInfoMap {
    infos: Vec<VarInfoHandle>
}

impl VarInfoMap {
    pub fn new() -> Self {
        Self { infos: Vec::new() }
    }

    pub fn add(&mut self, info: VariableInfo) -> VarInfoId {
        let id = self.infos.len();
        self.infos.push(Arc::new(RwLock::new(info)));
        VarInfoId(id)
    }

    pub fn get(&self, id: VarInfoId) -> VarInfoHandle {
        assert!(id.0 < self.infos.len());
        self.infos[id.0].clone()
    }


    pub fn log(&self) {
        let logger = Logger::new();
        for info in &self.infos {
            let info = info.read();
            logger.log_fmt(format_args!("{}:\n", info.sym.read().path()));
            info.log();
        }
    }
}