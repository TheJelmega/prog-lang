use std::sync::Arc;

use crate::{
    common::{LibraryPath, NameTable, OperatorTable, PrecedenceDAG, RootSymbolTable, RootUseTable, SpanRegistry, TraitDag, VarInfoMap},
    lexer::{Punctuation, PuncutationTable},
    literals::LiteralTable,
    type_system::TypeRegistry
};

use super::{Hir, HirError, VisitFlags, Visitor};
use parking_lot::RwLock;

mod misc_passes;
pub use misc_passes::*;

mod symbol_generation;
pub use symbol_generation::*;

mod precedence_passes;
pub use precedence_passes::*;

mod operator_passes;
pub use operator_passes::*;

mod trait_passes;
pub use trait_passes::*;


mod type_pass_utils;
mod type_passes;
pub use type_passes::*;

mod path_passes;
pub use path_passes::*;

mod expr_passes;
pub use expr_passes::*;

#[derive(Clone)]
pub struct PassContext {
    pub names:          Arc<RwLock<NameTable>>,
    pub puncts:         Arc<RwLock<PuncutationTable>>,
    pub lits:           Arc<RwLock<LiteralTable>>,
    pub spans:          Arc<RwLock<SpanRegistry>>,

    pub syms:           Arc<RwLock<RootSymbolTable>>,
    pub uses:           Arc<RwLock<RootUseTable>>,
    pub type_reg:       Arc<RwLock<TypeRegistry>>,

    pub trait_dag:      Arc<RwLock<TraitDag>>,

    pub precedence_dag: Arc<RwLock<PrecedenceDAG>>,
    pub op_table:       Arc<RwLock<OperatorTable>>,

    pub var_infos:      Arc<RwLock<VarInfoMap>>,

    pub lib_path:       LibraryPath,

    pub errors:         Arc<RwLock<Vec<HirError>>>,
}

impl PassContext {
    pub fn add_error(&self, err: HirError) {
        self.errors.write().push(err);
    }
}

pub trait Pass: Visitor {
    const NAME: &'static str;

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::all());
    }
}