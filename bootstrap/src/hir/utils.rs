use crate::common::{NameId, NameTable};

use super::{
    *,
    visitor::Visitor
};


// =============================================================================================================================

#[derive(Clone, Copy)]
pub struct CollectedPatternIden {
    pub span:   SpanId,
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
            span: node.span,
            name: node.name,
            is_mut: node.is_mut,
            is_ref: node.is_ref,
        });
    }

    fn visit_struct_pattern(&mut self, node: &mut StructPattern) {
        for field in &node.fields {
            match field {
                StructPatternField::Iden { span, node_id, is_ref, is_mut, iden, bound } => {
                    self.is_mut_and_names.push(CollectedPatternIden {
                        span: *span,
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

pub fn simple_path_to_scope(path: &SimplePath, names: &NameTable) -> Scope {
    let mut scope = Scope::new();
    for name in &path.names {
        let name = names[*name].to_string();
        scope.push(name);
    }
    scope
}

pub fn type_path_to_scope(path: &TypePath, names: &NameTable) -> Scope {
    let mut scope = Scope::new();
    for segment in &path.segments {
        match segment {
            TypePathSegment::Plain { span, name } => {
                let name = names[*name].to_string();
                scope.push(name);
            },
            TypePathSegment::GenArg { span, name, gen_args } => todo!(),
            TypePathSegment::Fn { span, name, params, ret } => todo!(),
        }
    }
    scope
}