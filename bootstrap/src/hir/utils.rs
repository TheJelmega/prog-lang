use crate::common::NameId;

use super::{
    *,
    visitor::Visitor
};


// =============================================================================================================================

#[derive(Clone, Copy)]
pub struct CollectedPatternIden {
    pub name:   NameId,
    pub is_mut: bool,
    pub is_ref: bool,
}

pub struct PatternIdenCollection {
    pub is_mut_and_names: Vec<CollectedPatternIden>,
}

impl PatternIdenCollection {
    pub fn new() -> Self {
        Self {
            is_mut_and_names: Vec::new(),
        }
    }
}

impl Visitor for PatternIdenCollection {
    fn visit_iden_pattern(&mut self, node: &mut IdenPattern) {
        self.is_mut_and_names.push(CollectedPatternIden {
            name: node.name,
            is_mut: node.is_mut,
            is_ref: node.is_ref,
        });
    }

    fn visit_struct_pattern(&mut self, node: &mut StructPattern) {
        for field in &node.fields {
            match field {
                StructPatternField::Iden { node_id, is_ref, is_mut, iden, bound } => {
                    self.is_mut_and_names.push(CollectedPatternIden {
                        name: *iden,
                        is_mut: *is_mut,
                        is_ref: *is_ref,
                    })
                },
                _ => (),
            }
        }
    }
}


// =============================================================================================================================
