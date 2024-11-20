use std::{
    collections::HashMap,
    path::PathBuf,
    fmt,
};

use super::IndentLogger;



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
    nodes: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, scope: &[String], name: String, file_path: PathBuf) {
        if !scope.is_empty() {
            let sub_table = self.find_subtable_mut(&scope[0]).unwrap();
            sub_table.add_module(&scope[1..], name, file_path);
        } else {
            let sym = Symbol::Module(ModuleSymbol{
                name: name.clone(),
                path: file_path,
                sub_table: SymbolTable::new(),
            });
            self.nodes.insert(name, sym);
        }
    }

    pub fn add_precedence(&mut self, scope: &[String], name: String, id: u16) {
        if !scope.is_empty() {
            let sub_table = self.find_subtable_mut(&scope[0]).unwrap();
            sub_table.add_precedence(&scope[1..], name, id);
        } else {
            let sym = Symbol::Precedence(PrecedenceSymbol {
                name: name.clone(),
                id,
            }); 
            self.nodes.insert(name, sym);
        }   
    }

    pub fn get_symbol(&self, scope: &[String], name: &str) -> Option<&Symbol> {
        if !scope.is_empty() {
            let sub_table = self.find_subtable(&scope[0])?;
            sub_table.get_symbol(&scope[1..], name)
        } else {
            self.nodes.get(name)
        }
    }

    fn find_subtable(&self, name: &str) -> Option<&SymbolTable> {
        let symbol = self.nodes.get(name)?;
        match symbol {
            Symbol::Module(sym) => Some(&sym.sub_table),
            _ => None,
        }
    }

    fn find_subtable_mut(&mut self, name: &str) -> Option<&mut SymbolTable> {
        let symbol = self.nodes.get_mut(name)?;
        match symbol {
            Symbol::Module(sym) => Some(&mut sym.sub_table),
            _ => None,
        }
    }




    pub fn log(&self) {
        let mut logger = SymbolTableLogger::new();
        logger.log_table(self);
    }
}

struct SymbolTableLogger {
    logger: IndentLogger
}

#[allow(unused)]
impl SymbolTableLogger {
    fn new() -> Self {
        Self {
            logger: IndentLogger::new("    ", "|   ", "+---"),
        }
    }

    pub fn log(&self, s: &str) {
        self.logger.log(s);
    }
    
    pub fn prefixed_log(&self, s: &str) {
        self.logger.prefixed_log(s);
    }
    
    pub fn logln(&self, s: &str) {
        self.logger.logln(s);
    }

    pub fn prefixed_logln(&self, s: &str) {
        self.logger.prefixed_logln(s);
    }

    pub fn log_fmt(&self, args: fmt::Arguments) {
        self.logger.log_fmt(args);
    }

    pub fn prefixed_log_fmt(&self, args: fmt::Arguments) {
        self.logger.prefixed_log_fmt(args);
    }

    pub fn push_indent(&mut self) {
        self.logger.push_indent();
    }

    pub fn pop_indent(&mut self) {
        self.logger.pop_indent();
    }

    pub fn set_last_at_indent(&mut self) {
        self.logger.set_last_at_indent();
    }

    pub fn set_last_at_indent_if(&mut self, cond: bool) {
        self.logger.set_last_at_indent_if(cond);
    }

    pub fn log_indented<F>(&mut self, name: &'static str, f: F) where
        F: Fn(&mut Self)
    {
        self.prefixed_logln(name);
        self.push_indent();
        f(self);
        self.pop_indent();
    }

    fn log_table(&mut self, table: &SymbolTable) {
        for (idx, (_, symbol)) in table.nodes.iter().enumerate() {
            if idx == table.nodes.len() - 1 {
                self.set_last_at_indent();
            }
            self.log_symbol(symbol);
        }
    }

    fn log_symbol(&mut self, sym: &Symbol) {
        match sym {
            Symbol::Module(sym) => self.log_indented("Module", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path.to_str().unwrap()));
                logger.set_last_at_indent();
                logger.log_indented("Sub Table", |logger| {
                    logger.log_table(&sym.sub_table);
                })
            }),
            Symbol::Precedence(sym) => self.log_indented("Precedence", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Id: {}\n", sym.id));
            }),
            _ => {}
        }
    }
}