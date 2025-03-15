use std::fmt;

use super::{dag::Dag, Logger};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrecedenceAssocKind {
    None,
    Left,
    Right,
}

impl fmt::Display for PrecedenceAssocKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrecedenceAssocKind::None  => write!(f, "none"),
            PrecedenceAssocKind::Left  => write!(f, "left"),
            PrecedenceAssocKind::Right => write!(f, "right"),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrecedenceOrder {
    None,
    Higher,
    Same,
    Lower,
}

struct PrecedenceNode {
    higher: Vec<u16>,
    lower:  Vec<u16>,
    name:   String,
}

pub struct PrecedenceDAG {
    lowest:  u16,
    highest: u16,
    dag:     Dag<PrecedenceNode>,
}

impl PrecedenceDAG {
    pub fn new() -> Self {
        Self {
            lowest: u16::MAX,
            highest: u16::MAX,
            dag: Dag::new(),
        }
    }

    pub(crate) fn set_highest(&mut self, id: u16) {
        self.highest = id;
    }

    pub(crate) fn set_lowest(&mut self, id: u16) {
        self.lowest = id;
    }

    pub fn add_precedence(&mut self, name: String) -> u16 {
        self.dag.add_node(PrecedenceNode {
            higher: Vec::new(),
            lower: Vec::new(),
            name,
        }) as u16
    }

    pub fn get(&self, name: &str) -> Option<u16> {
        self.dag.find_map(|id, data| if data.name == name {
            Some(id as u16)
        } else {
            None
        })
    }

    pub fn get_name(&self, idx: u32) -> Option<&str> {
        self.dag.get_data(idx).map(|data| data.name.as_str())
    }

    pub fn set_order(&mut self, lower: u16, higher: u16) {
        self.dag.get_data_mut(lower as u32).unwrap().higher.push(higher);
        self.dag.get_data_mut(higher as u32).unwrap().lower.push(lower);
        self.dag.set_predecessor(lower as u32, higher as u32);
    }

    pub fn calculate_order(&mut self) {
        // Before precomputing the order, check and fixup (if needed) the following
        // - lowest cannot have any predecessors
        // - highest cannot have any successors
        // - if not lowest and no predecessor exists, assign lowest as predecessor
        // - if not hightest and no successor exists, assign highest as successor
        let mut to_connect = Vec::new();
        for (idx, node) in self.dag.iter().enumerate() {
            let id = idx as u16;
            if id == self.lowest || id == self.highest {
                continue;
            }

            if node.higher.is_empty() {
                to_connect.push((id, self.highest));
            }
            if node.lower.is_empty() {
                to_connect.push((self.lowest, id));
            }
        }
        for (lower, higher) in to_connect {
            self.set_order(lower, higher);
        }

        // Now let the dag do it's work
        self.dag.calculate_predecessors();
    }

    pub fn check_cycles(&self) -> Vec<Vec<u32>> {
        self.dag.check_cycles()
    }

    pub fn get_order(&self, pred0: u16, pred1: u16) -> PrecedenceOrder {
        // Either node has no precedence, so there is no order
        if pred0 == u16::MAX || pred1 == u16::MAX {
            return PrecedenceOrder::None;
        }
        

        if pred0 == pred1 {
            PrecedenceOrder::Same
        } else if self.dag.has_predecessor(pred1 as u32, pred0 as u32) {
            // See if pred0 comes before pred1
            PrecedenceOrder::Higher
        } else if self.dag.has_predecessor(pred0 as u32, pred1 as u32) {
            // See if pred1 comes after pred0
            PrecedenceOrder::Lower
        } else {
            // otherwise there is no precedence relation
            PrecedenceOrder::None
        }
    }

    pub fn log_unordered(&self) {
        let logger = Logger::new();

        for (id, node) in self.dag.iter().enumerate() {
            logger.log_fmt(format_args!("Precedence {id}, path: {}\n", &node.name));
            if !node.higher.is_empty() {
                logger.log("    - lower than: ");
                for (idx, id) in node.higher.iter().enumerate() {
                    if idx != 0 {
                        logger.log(", ");
                    }
                    logger.log_fmt(format_args!("{id}"))
                }
                logger.logln("");
            }
            if !node.lower.is_empty() {
                logger.log("    - higher than: ");
                for (idx, id) in node.lower.iter().enumerate() {
                    if idx != 0 {
                        logger.log(", ");
                    }
                    logger.log_fmt(format_args!("{id}"))
                }
                logger.logln("");
            }

            let all_higher = self.dag.get_precomputed_predecessor_idxs(id as u32);
            if !all_higher.is_empty() {
                logger.log("    - precomputed lower than: ");
                for (idx, id) in all_higher.iter().enumerate() {
                    if idx != 0 {
                        logger.log(", ");
                    }
                    logger.log_fmt(format_args!("{id}"))
                }
                logger.logln("");
            }

        }
    }
}