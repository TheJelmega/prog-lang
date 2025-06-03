use std::{collections::HashMap, fmt};

use crate::lexer::{Punctuation, PuncutationTable};

use super::{LibraryPath, Logger, RootSymbolTable, RootUseTable, Scope, Symbol, SymbolRef};

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

    pub fn has_generics(&self) -> bool {
        matches!(self, OpType::Infix | OpType::Assign)
    }

    pub fn has_output(&self) -> bool {
        matches!(self, OpType::Prefix | OpType::Postfix | OpType::Infix)
    }

    pub fn is_binary(&self) -> bool {
        matches!(self, OpType::Infix | OpType::Assign)
    }
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
    pub precedence_name: String,
    pub precedence_id:   u16,
    pub library_path:    LibraryPath,
    pub trait_path:      Scope, //< Not really a scope, but good enough for now
    pub func_name:       String,
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

    pub fn build_from_symbols(&mut self, syms: &RootSymbolTable, uses: &RootUseTable) {
        let op_set_paths = uses.operator_set_paths();

        let mut process = |op_set: &SymbolRef, ops: &HashMap<String, SymbolRef>| {
            let op_set = op_set.read();
            let Symbol::OpSet(op_set) = &*op_set else { unreachable!() };

            let (precedence_name, precedence_id) = if let Some(precedence) = &op_set.precedence {
                let precedence = precedence.upgrade().unwrap();
                let precedence = precedence.read();
                let Symbol::Precedence(precedence) = &* precedence else { unreachable!() };
                let name = precedence.path.iden().name.clone();
                (name, precedence.id)
            } else {
                (String::new(), u16::MAX)
            };

            for (name, op) in ops {
                let op = op.read();
                let Symbol::Operator(op) = &*op else { unreachable!() };
            
                let info = OperatorInfo {
                    op_type: op.op_ty,
                    op: op.op,
                    precedence_name: precedence_name.clone(),
                    precedence_id,
                    library_path: op.path.lib().clone(),
                    trait_path: op_set.path.clone().to_full_scope(),
                    func_name: name.clone(),
                };
                self.add_operator(info);
            }
        };

        for use_path in op_set_paths {
            match &use_path.op_set {
                Some(name) => {
                    let (op_set, ops) = syms.get_direct_op_set_and_ops(&use_path.lib, name).unwrap();
                    process(op_set, ops);
                },
                None => for (_, (op_set, ops)) in syms.get_op_set_and_ops_for_lib(&use_path.lib).unwrap() {
                    process(op_set, ops);
                },
            }
        }
    }

    pub fn add_operator(&mut self, info: OperatorInfo) {
        let table = &mut self.tables[info.op_type as usize];
        table.insert(info.op, info);
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
                if op.precedence_id != u16::MAX {
                    logger.log_fmt(format_args!("    - Precedence: {} ({})\n", &op.precedence_name, op.precedence_id));
                }
            }
        }
    }
}
