use std::{collections::HashMap, fmt};

use crate::lexer::{Punctuation, PuncutationTable};

use super::{LibraryPath, Logger, Scope};


#[derive(Clone)]
pub struct OperatorImportPath {
    pub group:   Option<String>,
    pub package: String,
    pub library: String,
    pub op:      Punctuation,
}


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
    pub op_type:      OpType,
    pub op:           Punctuation,
    pub precedence:   Option<String>,
    pub library_path: LibraryPath,
    pub trait_path:   Scope, //< Not really a scope, but good enough for now
    pub func_name:    String,
}

pub struct OperatorTable {
    pub tables: [HashMap<Punctuation, OperatorInfo>; OpType::COUNT],
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
        }
    }

    pub fn add_operator(&mut self, info: OperatorInfo) {
        let table = &mut self.tables[info.op_type as usize];
        table.insert(info.op, info);
    }

    pub fn get(&self, op_type: OpType, op: Punctuation) -> Option<&OperatorInfo> {
        let table = &self.tables[op_type as usize];
        table.get(&op)
    }



    pub fn log(&self, puncts: &PuncutationTable) {
        let logger = Logger::new();

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
                if let Some(precedence) = &op.precedence {
                    logger.log_fmt(format_args!("    - Precedence: {precedence}\n"));
                }
            }
        }
    }
}
