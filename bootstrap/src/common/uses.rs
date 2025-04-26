use std::{collections::HashMap, fmt};

use crate::lexer::Punctuation;

use super::{IndentLogger, LibraryPath, Scope, ScopeSegment};


pub struct UsePath {
    // User defined library path or defaulted to current library
    pub lib_path: LibraryPath,
    pub path:     Scope,
    pub wildcard: bool,
    pub alias:    Option<String>,
}

impl fmt::Display for UsePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.lib_path)?;
        if ! self.path.is_empty() {
            write!(f, ".{}", &self.path)?;
        }
        if self.wildcard {
            write!(f, ".*")?;
        } else if let Some(alias) = &self.alias {
            write!(f, " as {alias}")?;
        }
        Ok(())
    }
}

#[allow(unused)]
#[derive(Clone)]
pub struct OpUsePath {
    pub lib_path: LibraryPath,
    pub op:       Punctuation,
}

#[allow(unused)]
#[derive(Clone)]
pub struct PrecedenceUsePath {
    pub lib_path:   LibraryPath,
    pub precedence: String,
}

pub struct RootUseTable {
    uses:             Vec<UsePath>,
    wildcards:        Vec<UsePath>,
    sub_tables:       HashMap<ScopeSegment, UseTable>,
    op_paths:         Vec<OpUsePath>,
    precedence_paths: Vec<PrecedenceUsePath>
}

impl RootUseTable {
    pub fn new() -> Self {
        Self {
            uses: Vec::new(),
            wildcards: Vec::new(),
            sub_tables: HashMap::new(),
            op_paths: Vec::new(),
            precedence_paths: Vec::new(),
        }
    }

    pub fn add_use(&mut self, scope: &Scope, use_path: UsePath) {
        self.add_use_(scope.segments(), use_path);
    }

    fn add_use_(&mut self, scope: &[ScopeSegment], use_path: UsePath) {
        if scope.is_empty() {
            if use_path.wildcard {
                self.wildcards.push(use_path);
            } else {
                self.uses.push(use_path);
            }
        } else {
            let sub_table = match self.sub_tables.get_mut(&scope[0]) {
                Some(table) => table,
                None => {
                    self.sub_tables.insert(scope[0].clone(), UseTable::new(scope[0].name.clone()));
                    self.sub_tables.get_mut(&scope[0]).unwrap()
                },
            };
            sub_table.add_use_(&scope[1..], use_path);
        }
    }

    pub fn check_non_wildcard_ambiguity(&self) -> Vec<(Scope, String)> {
        let mut ambiguities = Vec::new();
        self.check_non_wildcard_ambiguity_(Scope::new(), &mut ambiguities);
        ambiguities
    }

    pub fn check_non_wildcard_ambiguity_(&self, scope: Scope, ambiguities: &mut Vec<(Scope, String)>) {
        let mut check_map = HashMap::<String, bool>::new();

        for use_path in &self.uses {
            let name = use_path.alias.as_ref().unwrap_or_else(|| {
                &use_path.path.last().unwrap().name
            });

            match check_map.get_mut(name) {
                Some(val) => {
                    if !*val {
                        ambiguities.push((scope.clone(), name.clone()));
                    }
                    *val = true;
                },
                None => (),
            }
            check_map.insert(name.clone(), false);
        }

        for (segment, table) in &self.sub_tables {
            let mut sub_scope = scope.clone();
            sub_scope.push_segment(segment.clone());
            table.check_non_wildcard_ambiguity_(sub_scope, ambiguities);
        }
    }

    pub fn with_uses<F>(&self, scope: &Scope, mut f: F) where 
        F: FnMut(&UsePath) -> bool,
    {
        self.with_uses_(scope.segments(), &mut f);
    }

    
    fn with_uses_<F>(&self, scope: &[ScopeSegment], f: &mut F) -> bool where 
        F: FnMut(&UsePath) -> bool,
    {
        if !scope.is_empty() {
            if let Some(sub_table) = self.sub_tables.get(&scope[0]) {
                let found = sub_table.with_uses_(&scope[1..], f);
                if found {
                    return true;
                }
            }
        }

        let mut found = false;
        for use_path in &self.uses {
            found = f(use_path);
            if found {
                break;
            }
        }
        for use_path in &self.wildcards {
            found = f(use_path);
        }
        
        found
    }

    //==============================================================

    pub fn add_op_use(&mut self, use_path: OpUsePath) {
        self.op_paths.push(use_path);
    }

    //==============================================================

    pub fn add_precedence_us(&mut self, precedence_path: PrecedenceUsePath) {
        self.precedence_paths.push(precedence_path);
    }

    //==============================================================

    pub fn log(&self) {
        let mut logger = IndentLogger::new("    ", "|    ", "+---");
        self.log_(&mut logger);
    }

    pub fn log_(&self, logger: &mut IndentLogger) {
        logger.set_last_at_indent_if(self.wildcards.is_empty() && self.sub_tables.is_empty());
        logger.log_indented_slice_named("Direct Use Paths", &self.uses, |logger, use_path| {
            logger.prefixed_log_fmt(format_args!(" {}\n", &use_path))
        });
        logger.set_last_at_indent_if(self.sub_tables.is_empty());
        logger.log_indented_slice_named("Wildcard Use Paths", &self.wildcards, |logger, use_path| {
            logger.prefixed_log_fmt(format_args!(" {}\n", &use_path))
        });
        
        if !self.sub_tables.is_empty() {
            logger.set_last_at_indent();
            logger.prefixed_logln("Sub-tables");
            logger.push_indent();

            let end = self.sub_tables.len() - 1;
            for (idx, (_, sub_table)) in self.sub_tables.iter().enumerate() {
                logger.set_last_at_indent_if(idx == end);
                sub_table.log_(logger);
            }

            logger.pop_indent();
        }
    }
}

pub struct UseTable {
    name:       String,
    uses:       Vec<UsePath>,
    wildcards:  Vec<UsePath>,
    sub_tables: HashMap<ScopeSegment, UseTable>,
}

impl UseTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            uses: Vec::new(),
            wildcards: Vec::new(),
            sub_tables: HashMap::new(),

        }
    }

    fn add_use_(&mut self, scope: &[ScopeSegment], use_path: UsePath) {
        if scope.is_empty() {
            if use_path.wildcard {
                self.wildcards.push(use_path);
            } else {
                self.uses.push(use_path);
            }
        } else {
            let sub_table = match self.sub_tables.get_mut(&scope[0]) {
                Some(table) => table,
                None => {
                    self.sub_tables.insert(scope[0].clone(), UseTable::new(scope[0].name.clone()));
                    self.sub_tables.get_mut(&scope[0]).unwrap()
                },
            };
            sub_table.add_use_(&scope[1..], use_path);
        }
    }

    pub fn check_non_wildcard_ambiguity_(&self, scope: Scope, ambiguities: &mut Vec<(Scope, String)>) {
        let mut check_map = HashMap::<String, bool>::new();

        for use_path in &self.uses {
            let name = use_path.alias.as_ref().unwrap_or_else(|| {
                &use_path.path.last().unwrap().name
            });

            match check_map.get_mut(name) {
                Some(val) => {
                    if !*val {
                        ambiguities.push((scope.clone(), name.clone()));
                    }
                    *val = true;
                },
                None => (),
            }
            check_map.insert(name.clone(), false);
        }

        for (segment, table) in &self.sub_tables {
            let mut sub_scope = scope.clone();
            sub_scope.push_segment(segment.clone());
            table.check_non_wildcard_ambiguity_(sub_scope, ambiguities);
        }
    }

    fn with_uses_<F>(&self, scope: &[ScopeSegment], f: &mut F) -> bool where 
        F: FnMut(&UsePath) -> bool,
    {
        if !scope.is_empty() {
            if let Some(sub_table) = self.sub_tables.get(&scope[0]) {
                let found = sub_table.with_uses_(&scope[1..], f);
                if found {
                    return true;
                }
            }
        }

        let mut found = false;
        for use_path in &self.uses {
            found = f(use_path);
            if found {
                break;
            }
        }
        for use_path in &self.wildcards {
            found = f(use_path);
        }
        
        found
    }

    pub fn log_(&self, logger: &mut IndentLogger) {
        logger.prefixed_log_fmt(format_args!("Table: {}\n", &self.name));
        logger.push_indent();

        logger.set_last_at_indent_if(self.wildcards.is_empty() && self.sub_tables.is_empty());
        logger.log_indented_slice_named("Direct Use Paths", &self.uses, |logger, use_path| {
            logger.prefixed_log_fmt(format_args!(" {}\n", &use_path))
        });
        logger.set_last_at_indent_if(self.sub_tables.is_empty());
        logger.log_indented_slice_named("Wildcard Use Paths", &self.wildcards, |logger, use_path| {
            logger.prefixed_log_fmt(format_args!(" {}\n", &use_path))
        });

        if !self.sub_tables.is_empty() {
            logger.set_last_at_indent();
            logger.prefixed_logln("Sub-tables");
            logger.push_indent();
            let end = self.sub_tables.len() - 1;
            for (idx, (_, sub_table)) in self.sub_tables.iter().enumerate() {
                logger.set_last_at_indent_if(idx == end);
                sub_table.log_(logger);
            }
            logger.pop_indent();
        }

        logger.pop_indent();
    }
}