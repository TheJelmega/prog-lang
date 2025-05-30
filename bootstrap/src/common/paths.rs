#![allow(unused)]

use core::fmt;

use crate::type_system::TypeHandle;

use super::LibraryPath;

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub enum PathGeneric {
    Type {
        ty: TypeHandle,
    },
    Value {

    }
}

impl fmt::Display for PathGeneric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PathGeneric::Type { ty } => write!(f, "{ty}"),
            PathGeneric::Value {  }  => write!(f, "{{}}"),
        }
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct PathIden {
    pub name:     String,
    /// Parameter names for path elements that refer to functions
    pub params:   Vec<String>,
    pub gen_args: Vec<PathGeneric>,
}

impl PathIden {
    pub fn new(name: String, params: Vec<String>, gen_args: Vec<PathGeneric>) -> Self {
        Self {
            name,
            params,
            gen_args,
        }
    }
}

impl Default for PathIden {
    fn default() -> Self {
        Self {
            name: String::new(),
            params: Vec::new(),
            gen_args: Vec::new()
        }
    }
}

impl fmt::Display for PathIden {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)?;
        if !self.params.is_empty() {
            write!(f, "(")?;
            for (idx, param) in self.params.iter().enumerate() {
                write!(f, "{}{param}", if idx != 0 { ", " } else { "" });
            }
            for (idx, arg) in self.gen_args.iter().enumerate() {
                write!(f, "{}{arg}", if idx != 0 { ", " } else { "" });
            }
            write!(f, ")")?;
        }
        Ok(())
    }
}

//==============================================================================================================================

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Scope {
    segments: Vec<PathIden>,
}

#[allow(unused)]
impl Scope {
    pub fn new() -> Self {
        Self {
            segments: Vec::new(),
        }
    }

    pub fn push(&mut self, name: String) {
        self.segments.push(PathIden::new(name, Vec::new(), Vec::new()));
    }

    pub fn push_with_params(&mut self, name: String, params: Vec<String>) {
        self.segments.push(PathIden::new(name, params, Vec::new()));
    }

    pub fn push_iden(&mut self, segment: PathIden) {
        self.segments.push(segment);
    }

    pub fn extend(&mut self, extension: &Scope) {
        for segment in &extension.segments {
            self.push_iden(segment.clone());
        }
    }

    pub fn pop(&mut self) -> Option<PathIden> {
        self.segments.pop()
    }

    pub fn idens(&self) -> &Vec<PathIden> {
        &self.segments
    }

    pub fn mut_idens(&mut self) -> &mut Vec<PathIden> {
        &mut self.segments
    }

    pub fn is_empty(&self) -> bool {
        self.segments.is_empty()
    }

    pub fn len(&self) -> usize {
        self.segments.len()
    }
    
    pub fn parent(&self) -> Scope {
        if self.segments.len() <= 1 {
            return Scope::new();
        }

        let mut parent = Scope::new();
        for segment in &self.segments[..self.segments.len() - 1] {
            parent.segments.push(segment.clone());
        }
        parent
    }

    // Get the path without it's root
    pub fn sub_path(&self) -> Scope {
        if self.segments.len() <= 1 {
            return Scope::new();
        }

        let mut sub_path = Scope::new();
        for segment in &self.segments[1..] {
            sub_path.segments.push(segment.clone());
        }
        sub_path
    }

    pub fn root(&self) -> Option<&PathIden> {
        self.segments.first()
    }

    pub fn last(&self) -> Option<&PathIden> {
        self.segments.last()
    }
}

impl fmt::Display for Scope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (idx, iden) in self.segments.iter().enumerate() {
            write!(f, "{}{iden}", if idx != 0 { "," } else { "" })?;
        }
        Ok(())
    }
}


//==============================================================================================================================


#[derive(Clone, Debug)]
pub struct SymbolPath {
    lib:   LibraryPath,
    scope: Scope,
    iden: PathIden,
}

impl SymbolPath {
    pub fn new(lib: LibraryPath, scope: Scope, iden: PathIden) -> Self {
        Self {
            lib,
            scope,
            iden,
        }
    }

    pub fn lib(&self) -> &LibraryPath {
        &self.lib
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn iden(&self) -> &PathIden {
        &self.iden
    }
}

impl SymbolPath {
    pub fn from_scope(lib: LibraryPath, mut scope: Scope) -> Option<Self> {
        if scope.is_empty() {
            return None;
        }

        let iden = scope.pop().unwrap();
        Some(Self {
            lib,
            scope,
            iden,
        })
    }

    pub fn to_full_scope(&self) -> Scope {
        let mut scope = self.scope.clone();
        scope.push_iden(self.iden.clone());
        scope
    }
}

impl Default for SymbolPath {
    fn default() -> Self {
        Self { lib: LibraryPath::new(), scope: Scope::new(), iden: PathIden::default() }
    }
}

impl fmt::Display for SymbolPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}{}{}",
            self.lib,
            self.scope,
            if self.scope.is_empty() { "" } else { "." },
            self.iden
        )
    }
}

