use crate::common::Dag;

use super::{TypeHandle, TypeRef};



struct NodeData {
    ty: TypeHandle
}

pub struct DependencyDag {
    dag: Dag<NodeData>,
}

impl DependencyDag {
    pub fn new() -> Self {
        Self {
            dag: Dag::new(),
        }
    }

    pub fn add(&mut self, ty: TypeHandle) {
        let idx = self.dag.add_node(NodeData {
            ty: ty.clone(),
        });
        ty.handle.write().dag_idx = idx;
    }

    pub fn set_dependency(&mut self, idx: u32, base: u32) {
        self.dag.set_predecessor(idx, base);
    }

    pub fn log_nodes(&self) {
        self.dag.log_nodes(true, |data| &data.ty)
    }
}