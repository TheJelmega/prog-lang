
use std::{fmt, time, hash::Hash};

mod logger;
pub use logger::*;

mod stats;
pub use stats::*;

mod dag;

mod precedence;
pub use precedence::*;

mod operators;
pub use operators::*;

mod names;
pub use names::*;

mod symbol_table;
pub use symbol_table::*;

pub mod uses;
pub use uses::*;

mod scope;
pub use scope::*;

mod span;
pub use span::*;

mod traits;
pub use traits::*;

use crate::hir::Hir;


#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct LibraryPath {
    pub group:  Option<String>,
    pub package: String,
    pub library: String,
}

impl fmt::Display for LibraryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(group) = &self.group {
            write!(f, "{}.", group)?;
        }
        write!(f, "{}:{}", &self.package, &self.library)
    }
}



#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Abi {
    Xenon,
    C,
    Contextless,
}

impl fmt::Display for Abi {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Abi::Xenon       => write!(f, "xenon"),
            Abi::C           => write!(f, "C"),
            Abi::Contextless => write!(f, "contextless"),
        }
    }
}
