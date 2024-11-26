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

struct PrecedenceNode {
    prev: Vec<u16>,
    next: Vec<u16>,
    name: String,
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