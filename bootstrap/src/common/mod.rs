
use std::{fmt, hash::Hash};

mod logger;
pub use logger::*;

mod stats;
pub use stats::*;

mod dag;
pub use dag::Dag;

mod precedence;
pub use precedence::*;

mod operators;
pub use operators::*;

mod names;
pub use names::*;

mod symbols;
pub use symbols::*;

pub mod uses;
pub use uses::*;

mod scope;
pub use scope::*;

mod span;
pub use span::*;

mod traits;
pub use traits::*;

mod expr_utils;
pub use expr_utils::*;


#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct LibraryPath {
    pub group:  Option<String>,
    pub package: String,
    pub library: String,
}

impl LibraryPath {
    pub fn new() -> Self {
        Self {
            group: None,
            package: String::new(),
            library: String::new(),
        }
    }
}

impl fmt::Display for LibraryPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(group) = &self.group {
            write!(f, "{}.", group)?;
        }
        write!(f, "{}:{}", &self.package, &self.library)
    }
}

// =============================================================

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

// =============================================================

#[derive(Clone)]
pub enum Visibility {
    Public,
    Package {
        group:   Option<String>,
        package: String,
    },
    Lib(LibraryPath),
    Path(LibraryPath, Scope),
}

impl fmt::Display for Visibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Visibility::Public                     => write!(f, "public"),
            Visibility::Package { group, package } => {
                write!(f, "private(")?;
                if let Some(group) = group {
                    write!(f, "{group}.")?
                };
                write!(f, "{package})")
            },
            Visibility::Lib(lib)                   => write!(f, "private({lib})"),
            Visibility::Path(lib, path)            => write!(f, "private({lib}.{path})"),
        }
    }
}