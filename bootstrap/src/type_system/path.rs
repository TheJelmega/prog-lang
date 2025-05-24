use std::fmt;

use crate::common::{Scope, SymbolRef};


pub struct PathType {
    pub path: Scope,
    pub sym:  Option<SymbolRef>
}

impl fmt::Display for PathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.sym {
            Some(sym) => {
                let sym = sym.read();
                let path = sym.path();
                write!(f, "{path}")
            },
            None => {
                write!(f, "{}", &self.path)
            },
        }

        
    }
}