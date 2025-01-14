use std::collections::VecDeque;

use crate::common::NameId;

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
    Less,
    Greater,
}

struct PrecedenceNode {
    prev:         Vec<u16>,
    next:         Vec<u16>,
    name:         String,
    precomp_prev: Vec<u16>,
    is_precomped: bool,
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
            prev: Vec::new(),
            next: Vec::new(),
            name,
            precomp_prev: Vec::new(),
            is_precomped: false,
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

        self.nodes[lower as usize].next.push(higher);
        self.nodes[higher as usize].prev.push(lower);
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

            if node.prev.is_empty() {
                to_connect.push((self.lowest, id));
            }
            if node.next.is_empty() {
                to_connect.push((id, self.highest));
            }
        }
        for (lower, higher) in to_connect {
            self.set_order(lower, higher);
        }

        // Now go over the nodes and collect the set of previous nodes, if it's not processed yet, skip them
        let mut to_process = VecDeque::new();
        for id in &self.nodes[self.lowest as usize].next {
            to_process.push_back(*id);
        }

        while let Some(id) = to_process.pop_front() {
            let node = &self.nodes[id as usize];
            
            // Check if we can already process it, if not, push it on the back
            for prev in &node.prev {
                if *prev != self.lowest && self.nodes[*prev as usize].precomp_prev.is_empty() {
                    to_process.push_back(id);
                    continue;
                }
            }

            // Otherwise collect all lower nodes, dedup and sort them
            let mut precomp_prev = Vec::new();
            for prev in &node.prev {
                let prev_node = &self.nodes[*prev as usize];
                for tmp in &prev_node.precomp_prev {
                    precomp_prev.push(*tmp);
                }
            }

            precomp_prev.dedup();
            precomp_prev.sort();

            self.nodes[id as usize].precomp_prev = precomp_prev;
        }
    }

    // returns `None` if there is no relation between the precedences
    // otherwise `Some(x)` where `x` means `pred0` comes before `pred1`
    pub fn get_order(&self, pred0: u16, pred1: u16) -> Option<bool> {
        assert!((pred0 as usize) < self.nodes.len());
        assert!((pred1 as usize) < self.nodes.len());
        
        // See if pred0 comes before pred1
        if self.nodes[pred1 as usize].precomp_prev.contains(&pred0) {
            Some(true)
        } else if self.nodes[pred0 as usize].precomp_prev.contains(&pred1) {
            // See if pred1 comes after pred0
            Some(false)
        } else {
            // otherwise there is no precedence relation
            None
        }
    }

    pub fn log_unordered(&self) {
        let mut logger = Logger::new();

        for (id, node) in self.nodes.iter().enumerate() {
            logger.log_fmt(format_args!("Precedence {id}, path: {}", &node.name));
            if !node.prev.is_empty() {
                logger.log(", prev: ");
                for (idx, id) in node.prev.iter().enumerate() {
                    if idx != 0 {
                        logger.log(", ");
                    }
                    logger.log_fmt(format_args!("{id}"))
                }
            }
            if !node.next.is_empty() {
                logger.log(", next: ");
                for (idx, id) in node.next.iter().enumerate() {
                    if idx != 0 {
                        logger.log(", ");
                    }
                    logger.log_fmt(format_args!("{id}"))
                }
            }

            logger.logln("");
        }
    }
}