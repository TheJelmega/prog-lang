use std::{
    collections::HashMap,
    path::PathBuf,
    fmt::Write,
    fmt,
};

use super::{IndentLogger, Scope, ScopeSegment};



pub enum Symbol {
    Module(ModuleSymbol),
    Precedence(PrecedenceSymbol),
}

pub struct ModuleSymbol {
    pub name:      String,
    pub path:      PathBuf,
    pub sub_table: SymbolTable
}

pub struct PrecedenceSymbol {
    pub name: String,
    pub id:   u16,
}


pub struct SymbolTable {
    nodes: HashMap<String, Vec<(Vec<String>, Symbol)>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, scope: &Scope, name: String, file_path: PathBuf) {
        let sym = Symbol::Module(ModuleSymbol{
            name: name.clone(),
            path: file_path,
            sub_table: SymbolTable::new(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_precedence(&mut self, scope: &Scope, name: String, id: u16) {
        let sym = Symbol::Precedence(PrecedenceSymbol {
            name: name.clone(),
            id,
        }); 
        self.add_symbol(scope, &name, &[], sym)
    }

    fn add_symbol(&mut self, scope: &Scope, name: &str, params: &[String], sym: Symbol) {
        if !scope.is_empty() {
            let sub_table = self.find_subtable_mut(&scope.root().unwrap()).unwrap();
            sub_table.add_symbol(&scope.sub_path(), name, params, sym);
        } else {
            let params = Vec::from(params);
            let sub_syms = if let Some(syms) = self.nodes.get_mut(name) {
                syms
            } else {
                self.nodes.insert(name.to_string(), Vec::new());
                self.nodes.get_mut(name).unwrap()
            };
            sub_syms.push((Vec::from(params), sym));
        }
    }

    pub fn get_symbol(&self, scope: &Scope, name: &str) -> Option<&Symbol> {
        if !scope.is_empty() {
            let sub_table = self.find_subtable(scope.root().unwrap())?;
            sub_table.get_symbol(&scope.sub_path(), name)
        } else {
            let sub_syms = self.nodes.get(name)?;
            if sub_syms.len() == 1 {
                Some(&sub_syms[0].1)
            } else {
                let mut tmp = None;
                for (params, sym) in sub_syms {
                    if params.is_empty() {
                        tmp = Some(sym);
                        break;
                    }
                }
                tmp
            }
        }
    }

    fn find_subtable(&self, segment: &ScopeSegment) -> Option<&SymbolTable> {
        let sub_syms = self.nodes.get(&segment.name)?;
        let symbol = if sub_syms.len() == 1 {
            &sub_syms[0].1
        } else {
            let mut tmp = None;
            for (params, sym) in sub_syms {
                if &segment.params == params {
                    tmp = Some(sym);
                    break;
                }
            }
            tmp?
        };

        match symbol {
            Symbol::Module(sym) => Some(&sym.sub_table),
            _ => None,
        }
    }

    fn find_subtable_mut(&mut self, segment: &ScopeSegment) -> Option<&mut SymbolTable> {
        let sub_syms = self.nodes.get_mut(&segment.name)?;
        let symbol = if sub_syms.len() == 1 {
            &mut sub_syms[0].1
        } else {
            let mut tmp = None;
            for (params, sym) in sub_syms {
                if &segment.params == params {
                    tmp = Some(sym);
                    break;
                }
            }
            tmp?
        };

        match symbol {
            Symbol::Module(sym) => Some(&mut sym.sub_table),
            _ => None,
        }
    }




    pub fn log(&self) {
        let mut logger = IndentLogger::new("    ", "|   ", "+---");
        SymbolTableLogger::log_table(&mut logger, self);
    }
}

struct SymbolTableLogger {
    logger: IndentLogger
}

#[allow(unused)]
impl SymbolTableLogger {
    fn log_table(logger: &mut IndentLogger, table: &SymbolTable) {
        for (idx, (_, symbols)) in table.nodes.iter().enumerate() {
            if idx == table.nodes.len() - 1 {
                logger.set_last_at_indent();
            }

            if symbols.len() == 1 && symbols[0].0.is_empty() {
                Self::log_symbol(logger, &symbols[0].1);
            } else { 
                for (params, sym) in symbols {
                    let mut params_s = String::new();
                    if !params.is_empty() {
                        write!(&mut params_s, "(");
                        for (idx, param) in params.iter().enumerate() {
                            if idx != 0 {
                                write!(&mut params_s, ", ");
                            }
                            write!(&mut params_s, "{}", param);
                        }
                        write!(&mut params_s, ")");
                    }

                    logger.log_indented(&params_s, |logger| Self::log_symbol(logger, sym))
                }
            }

        }
    }

    fn log_symbol(logger: &mut IndentLogger, sym: &Symbol) {
        match sym {
            Symbol::Module(sym) => logger.log_indented("Module", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path.to_str().unwrap()));
                logger.set_last_at_indent();
                logger.log_indented("Sub Table", |logger| {
                    Self::log_table(logger, &sym.sub_table);
                })
            }),
            Symbol::Precedence(sym) => logger.log_indented("Precedence", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Id: {}\n", sym.id));
            }),
            _ => {}
        }
    }
}