use super::{Hir, VisitFlags, Visitor};


mod symbol_generation;
pub use symbol_generation::*;

mod precedence_passes;
pub use precedence_passes::*;

mod operator_passes;
pub use operator_passes::*;



pub trait Pass: Visitor {
    const NAME: &'static str;

    fn process(&mut self, hir: &mut Hir) {
        self.visit(hir, VisitFlags::all());
    }
}