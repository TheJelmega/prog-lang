use std::collections::VecDeque;

use super::Logger;

#[derive(Clone)]
pub struct PrecedenceImportPath {
    pub group:   Option<String>,
    pub package: String,
    pub library: String,
    pub name:    String,
}

impl PrecedenceImportPath {
    pub fn new(group: Option<String>, package: String, library: String, name: String) -> Self {
        Self {
            group,
            package,
            library,
            name,
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
    higher:         Vec<u16>,
    lower:         Vec<u16>,
    name:         String,
    precomp_higher: Vec<u16>,
}

pub struct PrecedenceDAG {
    nodes:   Vec<PrecedenceNode>,
    lowest:  u16,
    highest: u16,
}

impl PrecedenceDAG {
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            lowest: u16::MAX,
            highest: u16::MAX,
        }
    }

    pub(crate) fn set_highest(&mut self, id: u16) {
        self.highest = id;
    }

    pub(crate) fn set_lowest(&mut self, id: u16) {
        self.lowest = id;
    }

    pub fn add_precedence(&mut self, name: String) -> u16 {
        let id = self.nodes.len() as u16;
        self.nodes.push(PrecedenceNode {
            higher: Vec::new(),
            lower: Vec::new(),
            name,
            precomp_higher: Vec::new(),
        });
        id
    }

    pub fn get(&self, name: &str) -> Option<u16> {
        self.nodes.iter().enumerate().find_map(|(id, node)| if node.name == name {
            Some(id as u16)
        } else {
            None
        })
    }

    pub fn get_id(&self, name: &str) -> u16 {
        self.nodes.iter().enumerate().find_map(|(id, node)| if node.name == name {
            Some(id as u16)
        } else {
            None
        }).unwrap()
    }

    pub fn set_order(&mut self, lower: u16, higher: u16) {
        assert!((lower as usize) < self.nodes.len());
        assert!((higher as usize) < self.nodes.len());

        self.nodes[lower as usize].higher.push(higher);
        self.nodes[higher as usize].lower.push(lower);
    }

    pub fn precompute_order(&mut self) {
        // Before precomputing the order, check and fixup (if needed) the following
        // - lowest cannot have any predecessors
        // - highest cannot have any successors
        // - if not lowest and no predecessor exists, assign lowest as predecessor
        // - if not hightest and no successor exists, assign highest as successor
        let mut to_connect = Vec::new();
        for (idx, node) in self.nodes.iter().enumerate() {
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

        // Now go over the nodes and collect the set of higher nodes, if it's not processed yet, skip them
        let mut to_process = VecDeque::new();
        for id in &self.nodes[self.highest as usize].lower {
            to_process.push_back(*id);
        }  

         while let Some(id) = to_process.pop_front() {
            let node = &self.nodes[id as usize];
            // Skip if we already processed this (happens when node have multiple sources)
            if !node.precomp_higher.is_empty() {
                continue;
            }
            
            // Check if we can already process it, if not, push it on the back
            for higher in &node.higher {
                if *higher != self.highest && self.nodes[*higher as usize].precomp_higher.is_empty() {
                    to_process.push_back(id);
                    continue;
                }
            }

            // Otherwise collect all higher nodes, dedup and sort them
            let mut precomp_higher = Vec::new();
            for higher in &node.higher {
                let node = &self.nodes[*higher as usize];
                for tmp in &node.precomp_higher {
                    precomp_higher.push(*tmp);
                }
                precomp_higher.push(*higher);
            }

            precomp_higher.dedup();
            precomp_higher.sort();
            
            // add lower nodes to process
            for id in &node.lower {
                to_process.push_back(*id);
            }

            self.nodes[id as usize].precomp_higher = precomp_higher;
        }
    }

    pub fn get_order(&self, pred0: u16, pred1: u16) -> PrecedenceOrder {
        // Either node has no precedence, so there is no order
        if pred0 == u16::MAX || pred1 == u16::MAX {
            return PrecedenceOrder::None;
        }

        assert!((pred0 as usize) < self.nodes.len());
        assert!((pred1 as usize) < self.nodes.len());

        if pred0 == pred1 {
            PrecedenceOrder::Same
        } else if self.nodes[pred1 as usize].precomp_higher.contains(&pred0) {
            // See if pred0 comes before pred1
            PrecedenceOrder::Higher
        } else if self.nodes[pred0 as usize].precomp_higher.contains(&pred1) {
            // See if pred1 comes after pred0
            PrecedenceOrder::Lower
        } else {
            // otherwise there is no precedence relation
            PrecedenceOrder::None
        }
    }

    pub fn log_unordered(&self) {
        let logger = Logger::new();

        for (id, node) in self.nodes.iter().enumerate() {
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
            if !node.precomp_higher.is_empty() {
                logger.log("    - precomputed lower than: ");
                for (idx, id) in node.precomp_higher.iter().enumerate() {
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