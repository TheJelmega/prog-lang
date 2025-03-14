use super::{dag::Dag, Logger, Symbol, SymbolRef};


pub struct TraitDagData {
    pub symbol:     SymbolRef,
}

// A dag containing all dependency info for traits
pub struct TraitDag {
    dag: Dag<TraitDagData>,
}

impl TraitDag {
    pub fn new() -> Self {
        Self {
            dag: Dag::new(),
        }
    }

    pub fn add(&mut self, sym: SymbolRef) -> u32 {
        self.dag.add_node(TraitDagData {
            symbol: sym,
        })
    }

    pub fn set_base_dependency(&mut self, idx: u32, base: u32) {
        self.dag.set_predecessor(idx, base);
    }

    pub fn get(&self, idx: u32) -> Option<&TraitDagData> {
        self.dag.get_data(idx)
    }

    pub fn get_base_ids(&self, idx: u32) -> &[u32] {
        self.dag.get_precomputed_predecessor_idxs(idx)
    }

    pub fn calculate_predecessors(&mut self) {
        self.dag.calculate_predecessors();
    }

    pub fn check_cycles(&self) -> Vec<Vec<u32>> {
        self.dag.check_cycles()
    }

    pub fn log_unordered(&self) {
        let logger = Logger::new();

        for (id, node) in self.dag.iter().enumerate() {
            let sym = node.symbol.read();

            let Symbol::Trait(sym) = &*sym else { unreachable!() };

            logger.log_fmt(format_args!("Trait {id:03}, path: {}\n", sym.path));
            let predecessors = self.dag.get_precomputed_predecessor_idxs(id as u32);
            if !predecessors.is_empty() {
                logger.logln("    Depends on:");
                for pred_id in predecessors {
                    let pred = self.dag.get_data(*pred_id).unwrap().symbol.read();
                    let Symbol::Trait(pred) = &*pred else { unreachable!() };
                    logger.log_fmt(format_args!("    - Trait: {pred_id:03}, path: {}\n", pred.path));
                }
            }
        }
    }
}