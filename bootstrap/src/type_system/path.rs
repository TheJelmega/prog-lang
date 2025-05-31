use std::fmt;

use crate::common::{Scope, SymbolPath, SymbolRef};


pub struct PathType {
    pub path: SymbolPath,
    pub sym:  Option<SymbolRef>
}

impl fmt::Debug for PathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PathType")
            .field("path", &self.path)
        .finish()
    }
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