use std::{collections::HashMap, fmt};

use crate::lexer::{Punctuation, PuncutationTable};

use super::{LibraryPath, Logger, Scope};

// TODO: Assign is infix, but special, so make sure we also look at it when looking for infix ops
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OpType {
    Prefix,
    Postfix,
    Infix,
    Assign,
}

impl OpType {
    const COUNT: usize = OpType::Assign as usize + 1;
}

impl fmt::Display for OpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OpType::Prefix    => write!(f, "prefix"),
            OpType::Infix     => write!(f, "infix"),
            OpType::Postfix   => write!(f, "postfix"),
            OpType::Assign    => write!(f, "assign"),
        }
    }
}


pub struct OperatorInfo {
    pub op_type:         OpType,
    pub op:              Punctuation,
    // TODO: name + id
    pub precedence_name: String,
    pub precedence_id:   u16,
    pub library_path:    LibraryPath,
    pub trait_path:      Scope, //< Not really a scope, but good enough for now
    pub func_name:       String,
}

pub struct OperatorTable {
    pub tables: [HashMap<Punctuation, OperatorInfo>; OpType::COUNT],
    pub trait_precedences: HashMap<Scope, (String, u16)>,
}

impl OperatorTable {
    pub fn new() -> Self {
        Self {
            tables: [
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
                HashMap::new(),
            ],
            trait_precedences: HashMap::new(),
        }
    }

    pub fn add_operator(&mut self, info: OperatorInfo) {
        let table = &mut self.tables[info.op_type as usize];
        table.insert(info.op, info);
    }

    pub fn add_trait_precedence(&mut self, scope: Scope, name: String, id: u16) {
        self.trait_precedences.insert(scope, (name, id));
    }

    pub fn get(&self, op_type: OpType, op: Punctuation) -> Option<&OperatorInfo> {
        if op_type == OpType::Infix {
            let table = &self.tables[op_type as usize];
            if let Some(op) = table.get(&op) {
                return Some(op);
            }
            
            // Assign is infix, but special, so make sure we also look at it when looking for infix ops
            let table = &self.tables[OpType::Assign as usize];
            table.get(&op)
        } else {
            let table = &self.tables[op_type as usize];
            table.get(&op)
        }
    }

    pub fn get_trait_precedence(&self, scope: &Scope) -> Option<(&str, u16)> {
        self.trait_precedences.get(scope).map(|(s, id)| (s.as_str(), *id))
    }

    pub fn log(&self, puncts: &PuncutationTable) {
        let logger = Logger::new();

        logger.log("Trait Precedences:\n");
        for (path, (pred, id)) in &self.trait_precedences {
            logger.log_fmt(format_args!("    - {}: {} ({})\n", path, pred, id));
        }

        let op_types = [
            OpType::Prefix,
            OpType::Postfix,
            OpType::Infix,
            OpType::Assign,
        ];

        for op_ty in op_types {
            let table = &self.tables[op_ty as usize];
            if table.is_empty() {
                continue;
            }

            logger.log_fmt(format_args!("Op Type: {op_ty}\n"));
            for op in table.values() {
                let trait_path = op.trait_path.to_string();

                logger.log_fmt(format_args!("    Operator:     {}\n", op.op.as_str(puncts)));
                logger.log_fmt(format_args!("    - Library:    {}\n", op.library_path));
                logger.log_fmt(format_args!("    - Trait:      {trait_path}\n"));
                logger.log_fmt(format_args!("    - Func:       {}\n", op.func_name));
                logger.log_fmt(format_args!("    - Precedence: {} ({})\n", &op.precedence_name, op.precedence_id));
            }
        }
    }
}
