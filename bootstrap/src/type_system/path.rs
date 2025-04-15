use std::fmt;

use crate::common::SymbolRef;


pub struct PathType {
    pub sym: SymbolRef
}

impl fmt::Display for PathType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let sym = self.sym.read();
        let path = sym.path();
        write!(f, "{path}")
    }
}