use std::{
    collections::HashMap,
    fmt, mem
};

use crate::lexer::Punctuation;

use super::{IndentLogger, LibraryPath, PathIden, RootSymbolTable, Scope};

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum UsePathKind {
    Explicit,
    Alias(String),
    Wildcard,
    GenericOnly,
    FileRoot,
}

#[derive(Clone, Debug)]
pub struct UsePath {
    // User defined library path or defaulted to current library
    pub lib_path:      LibraryPath,
    pub path:          Scope,
    pub kind:          UsePathKind,
    pub last_in_scope: bool,
}

impl PartialEq for UsePath {
    fn eq(&self, other: &Self) -> bool {
        self.lib_path == other.lib_path &&
        self.path == other.path &&
        self.kind == other.kind
    }
}

impl fmt::Display for UsePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", &self.lib_path)?;
        if ! self.path.is_empty() {
            write!(f, ".{}", &self.path)?;
        }

        match &self.kind {
            UsePathKind::Explicit => Ok(()),
            UsePathKind::Alias(alias) => write!(f, " as {alias}"),
            UsePathKind::Wildcard => write!(f, ".*"),
            UsePathKind::GenericOnly => write!(f, " (generics only)"),
            UsePathKind::FileRoot => write!(f, " (file use root)"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct PrecedenceUsePath {
    pub lib:        LibraryPath,
    pub precedence: Option<String>,
}

impl fmt::Display for PrecedenceUsePath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.lib)?;
        if let Some(precedence) = &self.precedence {
            write!(f, ".{precedence}")?;
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

#[derive(Clone, Debug)]
pub enum UseTableError {
    InvalidPrecedences { paths: Vec<PrecedenceUsePath> },
    InvalidSymbols{ paths: Vec<(Scope, UsePath)> },
    AmbiguousPrecedences { ambiguities: Vec<(String, Vec<PrecedenceUsePath>)> },
    AmbiguousUses { ambiguities: Vec<(Scope, String, Vec<UsePath>)> }
}

impl fmt::Display for UseTableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPrecedences { paths } => {
                writeln!(f, "Use table contains ambiguous precedences")?;
                for path in paths {
                    writeln!(f, "- {path}")?;
                }
                Ok(())
            },
            UseTableError::InvalidSymbols { paths } => {
                writeln!(f, "Use table contains ambiguous symbols:")?;
                for (scope, path) in paths {
                    writeln!(f, "- {scope}: {path}")?;
                }
                Ok(())
            },
            Self::AmbiguousPrecedences { ambiguities } => {
                writeln!(f, "Use table contains ambiguous precedences (may be included via a wildcard)")?;
                for (name, paths) in ambiguities {
                    writeln!(f, "- '{name}' via:")?;
                    for path in paths {
                        writeln!(f, "   - {path}")?;
                    }
                }
                Ok(())
            }
            Self::AmbiguousUses { ambiguities } => {
                writeln!(f, "Use table contains ambiguous uses:")?;
                for (scope, name, paths) in ambiguities {
                    writeln!(f, "- '{name}' in {scope}:")?;
                    for path in paths {
                        writeln!(f, "   - {path}")?;
                    }
                }
                Ok(())
            },
        }
    }
}

pub struct RootUseTable {
    lib_path:         LibraryPath,
    // Uses defined within a given scope
    uses:             Vec<UsePath>,
    wildcards:        Vec<UsePath>,
    generic:          Option<UsePath>,
    sub_tables:       HashMap<PathIden, UseTable>,
    op_paths:         Vec<OpUsePath>,
    precedence_paths: Vec<PrecedenceUsePath>
}

impl RootUseTable {
    pub fn new(lib: LibraryPath) -> Self {
        Self {
            lib_path: lib.clone(),
            uses: Vec::new(),
            wildcards: Vec::new(),
            generic: None,
            sub_tables: HashMap::new(),
            op_paths: Vec::new(),
            precedence_paths: vec![PrecedenceUsePath { lib, precedence: None }],
        }
    }

    pub fn add_file_use_root(&mut self, file_scope: &Scope) {
        let scope = file_scope.clone();
        let segments = file_scope.idens();
        let lib_path = self.lib_path.clone();
        let sub_table = self.get_or_add_sub_table(segments[0].clone());
        sub_table.add_file_use_root(lib_path, &segments[1..], scope);
    }

    pub fn add_generic_use(&mut self, scope: Scope) {
        assert!(!scope.is_empty());

        let segments = scope.idens();
        let lib_path = self.lib_path.clone();
        let sub_table = self.get_or_add_sub_table(segments[0].clone());
        sub_table.add_generic_use(lib_path, &segments[1..], scope.clone());
    }

    pub fn add_use(&mut self, scope: &Scope, use_path: UsePath) {
        self.add_use_(scope.idens(), use_path);
    }

    fn add_use_(&mut self, scope: &[PathIden], use_path: UsePath) {
        if scope.is_empty() {
            match &use_path.kind {
                UsePathKind::Wildcard => self.wildcards.push(use_path),
                UsePathKind::GenericOnly => panic!("Generic use paths cannot be added using RootUseTable::add_use"),
                UsePathKind::FileRoot => panic!("File use root paths cannot be added using RootUseTable::add_use"),
                _ => self.uses.push(use_path),
            }
        } else {
            let sub_table = self.get_or_add_sub_table(scope[0].clone());
            sub_table.add_use_(&scope[1..], use_path);
        }
    }

    // TODO: matching precedence names in 2 'wildcard' paths should result in an error
    pub fn finalize_precedences(&mut self, sym_table: &RootSymbolTable) -> Result<(), UseTableError> {
        // Check for invalid precedence paths
        let mut invalid_precedences = Vec::new();
        for use_path in &self.precedence_paths {
            match &use_path.precedence {
                Some(name) => if sym_table.get_direct_precedence(&use_path.lib, name).is_none() {
                    invalid_precedences.push(use_path.clone());
                },
                None => if !sym_table.has_precedence_for_lib(&use_path.lib) {
                    invalid_precedences.push(use_path.clone());
                },
            }
        }
        if !invalid_precedences.is_empty() {
            return Err(UseTableError::InvalidPrecedences { paths: invalid_precedences });
        }
 
        // Remove overlapping precedence paths (i.e. explicit and wildcard)
        // Also check for ambiguity between explicit precedence uses
        let mut precedences = Vec::<PrecedenceUsePath>::new();
        for path in mem::take(&mut self.precedence_paths) {
            match &path.precedence {
                Some(name) => {
                    let is_dup = precedences.iter().find(|prec| prec.lib != path.lib && 
                        (prec.precedence.is_none() || prec.precedence.as_ref() == Some(name))
                    ).is_some();
                    if !is_dup {
                        precedences.push(path);
                    }
                },
                None => {
                    precedences.retain(|prec| prec.lib != path.lib);
                    precedences.push(path);
                },
            }
        }
        self.precedence_paths = precedences;

        // Check if there are no duplicate precedences included via wildcards
        let mut collected_paths = HashMap::<String, Vec<PrecedenceUsePath>>::new();
        for path in &self.precedence_paths {
            match &path.precedence {
                Some(name) => {
                    let entry = collected_paths.entry(name.clone()).or_default();
                    entry.push(path.clone());
                },
                None => for (name, _) in sym_table.get_precedences_for_lib(&path.lib).unwrap() {
                    let entry = collected_paths.entry(name.clone()).or_default();
                    entry.push(path.clone());
                },
            }
        }

        let mut ambiguities = Vec::new();
        for (name, paths) in collected_paths {
            if paths.len() > 1 {
                ambiguities.push((name, paths));
            }
        }

        if !ambiguities.is_empty() {
            return Err(UseTableError::AmbiguousPrecedences { ambiguities })
        }
        Ok(())
    }

    // TODO: matching precedence names in 2 'wildcard' paths should result in an error
    pub fn finalize_operators(&mut self, sym_table: &RootSymbolTable) -> Result<(), UseTableError> {
        let mut invalid_op_sets = Vec::new();
        for op_use in &self.op_paths {
            match &op_use.op_set {
                Some(op_set) => if sym_table.get_direct_op_set(&op_use.lib, op_set).is_none() {
                    invalid_op_sets.push(op_use.clone());
                },
                None =>  if !sym_table.has_op_set_for_lib(&op_use.lib) {
                    invalid_op_sets.push(op_use.clone());
                },
            }
        }

        if invalid_op_sets.is_empty() {
            Ok(())
        } else {
            Err(UseTableError::InvalidOperators { paths: invalid_op_sets })
        }
    }

    pub fn finalize(&mut self, sym_table: &RootSymbolTable) -> Result<(), UseTableError> {
        self.validate_paths(sym_table).map_err(|paths| UseTableError::InvalidSymbols{ paths })?;
        self.check_non_wildcard_ambiguity().map_err(|ambiguities| UseTableError::AmbiguousUses { ambiguities })?;
        self.remove_dup_uses();
        Ok(())
    }
    
    fn check_non_wildcard_ambiguity(&self) -> Result<(), Vec<(Scope, String, Vec<UsePath>)>> {
        let mut ambiguities = Vec::new();
        self.check_non_wildcard_ambiguity_(Scope::new(), &mut ambiguities);
        if ambiguities.is_empty() {
            Ok(())
        } else {
            Err(ambiguities)
        }
    }

    fn check_non_wildcard_ambiguity_(&self, scope: Scope, ambiguities: &mut Vec<(Scope, String, Vec<UsePath>)>) {
        let mut check_map = HashMap::<String, Vec<UsePath>>::new();

        for use_path in &self.uses {
            let name = match &use_path.kind {
                UsePathKind::Explicit => &use_path.path.last().unwrap().name,
                UsePathKind::Alias(alias) => alias,
                _ => continue,
            };

            match check_map.get_mut(name) {
                Some(entry) => {
                    entry.push(use_path.clone());
                },
                None => (),
            }
            check_map.insert(name.clone(), vec![use_path.clone()]);
        }

        for (name, paths) in check_map {
            if paths.len() > 1 {
                ambiguities.push((scope.clone(), name, paths));
            }
        }

        for (segment, table) in &self.sub_tables {
            let mut sub_scope = scope.clone();
            sub_scope.push_iden(segment.clone());
            table.check_non_wildcard_ambiguity_(sub_scope, ambiguities);
        }
    }

    // If we have 2 use paths, which refer to the same use, or one that overlaps with a wildcard, we don't need them, as they will be looked at anyway.
    // So this allows us to just remove those duplicates beforehand
    fn remove_dup_uses(&mut self) {
        // Exact duplicate paths are paths with the same:
        // - libary path
        // - path
        // - kind
        // - alias
        //
        // explicit paths that are duplicate with a wildcard, if they have:
        // - same library path
        // - a path parent that is the same as a wildcard path

        // First remove duplicates in the wildcards
        let mut wildcards = Vec::new();
        for wildcard in mem::take(&mut self.wildcards) {
            let is_dup = wildcards.iter().find(|use_path| **use_path == wildcard).is_some();
            if !is_dup {
                wildcards.push(wildcard);
            }
        }
        self.wildcards = wildcards;

        // Then handle explicit paths
        let mut uses = Vec::new();
        for use_path in mem::take(&mut self.uses) {
            let mut is_dup = uses.iter().find(|cur_use| **cur_use == use_path).is_some();
            if !is_dup {
                is_dup = self.wildcards.iter().find(|wildcard| {
                    use_path.kind == UsePathKind::Wildcard &&
                    use_path.lib_path == wildcard.lib_path &&
                    use_path.path.parent() == wildcard.path
                }).is_some();
            }
            if !is_dup {
                uses.push(use_path);
            }
        }
        self.uses = uses;

        for (_, sub_table) in &mut self.sub_tables {
            sub_table.remove_dup_uses();
        }

        let mut precedences = Vec::new();
        for path in &self.precedence_paths {
            match &path.precedence {
                Some(precedence) => {
                    let is_dup = self.precedence_paths.iter().find(|path| 
                        path.precedence.is_none() ||
                        path.precedence.as_ref() == Some(precedence)
                    ).is_some();
                    if !is_dup {
                        precedences.push(path)
                    }
                },
                None => {
                    precedences.retain(|precedence| precedence.lib != path.lib);
                    precedences.push(path);
                },
            }
        }
    }

    fn validate_paths(&self, sym_table: &RootSymbolTable) -> Result<(), Vec<(Scope, UsePath)>> {
        let mut invalid_symbols = Vec::new();

        for use_path in &self.uses {
            let scope = use_path.path.parent();
            let name = use_path.path.last().unwrap().name.clone();
            if sym_table.get_symbol(Some(&use_path.lib_path), &scope, &name).is_none() {
                invalid_symbols.push((Scope::new(), use_path.clone()));
            }
        }
        for wildcard in &self.wildcards {
            let scope = wildcard.path.parent();
            let name = wildcard.path.last().unwrap().name.clone();
            if sym_table.get_symbol(Some(&wildcard.lib_path), &scope, &name).is_none() {
                invalid_symbols.push((Scope::new(), wildcard.clone()));
            }
        }

        let mut scope = Scope::new();
        for (segment, sub_table) in &self.sub_tables {
            scope.push_iden(segment.clone());
            sub_table.validate_paths(sym_table, &mut scope, &mut invalid_symbols);
            scope.pop();
        }

        if invalid_symbols.is_empty() {
            Ok(())
        } else {
            Err(invalid_symbols)
        }
    }

    // Find all use scopes that a symbol could possible be found in, up to the 'root' of the file
    pub fn get_use_paths(&self, scope: &Scope) -> Vec<UsePath> {
        let mut use_paths = Vec::new();
        
        // Get all use paths in the current scope, and if possible, the inner most file use root
        let file_root = if !scope.is_empty() {
            let segments = scope.idens();
            if let Some(sub_table) = self.sub_tables.get(&segments[0]) {
                sub_table.get_use_paths(&segments[1..], &mut use_paths)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(file_use_root) = file_root {
            // If there was a file root lower down, we in a sub module, so just append that path to the front
            use_paths.insert(0, file_use_root);
        } else {
            // No file root was found, which means the current library is the file use root
            // So we add this file root + the uses within the root table
            use_paths.insert(0, UsePath {
                lib_path: self.lib_path.clone(),
                path: Scope::new(),
                kind: UsePathKind::FileRoot,
                last_in_scope: false,
            });
            use_paths.extend_from_slice(&self.uses);
            use_paths.extend_from_slice(&self.wildcards);
        }

        if let Some(generic_use_path) = &self.generic {
            use_paths.push(generic_use_path.clone());
        }

        // Make sure to set this as the last symbol within a scope, this is required to handle resolving the symbol correctly
        use_paths.last_mut().unwrap().last_in_scope = true;
        use_paths
    }

    fn get_or_add_sub_table(&mut self, segment: PathIden) -> &mut UseTable {
        let name = segment.name.clone();
        let entry = self.sub_tables.entry(segment);
        entry.or_insert_with(|| UseTable::new(name))
    }

    //==============================================================

    pub fn add_op_use(&mut self, use_path: OpUsePath) {
        self.op_paths.push(use_path);
    }

    //==============================================================

    pub fn add_precedence_use(&mut self, precedence_path: PrecedenceUsePath) {
        self.precedence_paths.push(precedence_path);
    }

    pub fn precedence_paths(&self) -> &Vec<PrecedenceUsePath> {
        &self.precedence_paths
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
    name:          String,
    // Root uses for a file (i.e. all scopes inside of this root can access all symbols within this root)
    file_use_root: Option<UsePath>,
    uses:          Vec<UsePath>,
    wildcards:     Vec<UsePath>,
    sub_tables:    HashMap<PathIden, UseTable>,
}

impl UseTable {
    pub fn new(name: String) -> Self {
        Self {
            name,
            file_use_root: None,
            uses: Vec::new(),
            wildcards: Vec::new(),
            sub_tables: HashMap::new(),

        }
    }

    fn get_or_add_sub_table(&mut self, segment: PathIden) -> &mut UseTable {
        let name = segment.name.clone();
        let entry = self.sub_tables.entry(segment);
        entry.or_insert_with(|| UseTable::new(name))
    }

    fn add_file_use_root(&mut self, lib_path: LibraryPath, segments: &[PathIden], scope: Scope) {
        if segments.is_empty() {
            self.file_use_root = Some(UsePath {
                lib_path,
                path: scope,
                kind: UsePathKind::FileRoot,
                last_in_scope: false,
            });
        } else {
            let sub_table = self.get_or_add_sub_table(segments[0].clone());
            sub_table.add_file_use_root(lib_path, &segments[1..], scope);
        }
    }

    fn add_generic_use(&mut self, lib_path: LibraryPath, segments: &[PathIden], scope: Scope) {
        if segments.is_empty() {
            self.uses.push(UsePath {
                lib_path,
                path: scope,
                kind: UsePathKind::GenericOnly,
                last_in_scope: false,
            })
        } else {
            let sub_table = self.get_or_add_sub_table(segments[0].clone());
            sub_table.add_generic_use(lib_path, &segments[1..], scope);
        }
    }

    fn add_use_(&mut self, scope: &[PathIden], use_path: UsePath) {
        if scope.is_empty() {
            match &use_path.kind {
                UsePathKind::Wildcard => self.wildcards.push(use_path),
                UsePathKind::GenericOnly => panic!("Generic use paths cannot be added using RootUseTable::add_use"),
                _ => self.uses.push(use_path),
            }
        } else {
            let sub_table = self.get_or_add_sub_table(scope[0].clone());
            sub_table.add_use_(&scope[1..], use_path);
        }
    }

    // Check if we have any path that might be ambiguous with an alias, or 2 aliases with each other
    pub fn check_non_wildcard_ambiguity_(&self, scope: Scope, ambiguities: &mut Vec<(Scope, String, Vec<UsePath>)>) {
        let mut check_map = HashMap::<String, Vec<UsePath>>::new();

        for use_path in &self.uses {
            let name = match &use_path.kind {
                UsePathKind::Explicit => &use_path.path.last().unwrap().name,
                UsePathKind::Alias(alias) => alias,
                _ => continue,
            };

            match check_map.get_mut(name) {
                Some(entry) => {
                    entry.push(use_path.clone());
                },
                None => (),
            }
            check_map.insert(name.clone(), vec![use_path.clone()]);
        }

        for (name, paths) in check_map {
            if paths.len() > 1 {
                ambiguities.push((scope.clone(), name, paths));
            }
        }

        for (segment, table) in &self.sub_tables {
            let mut sub_scope = scope.clone();
            sub_scope.push_iden(segment.clone());
            table.check_non_wildcard_ambiguity_(sub_scope, ambiguities);
        }
    }

    // Has the exact same logic as the function within the root use table
    fn remove_dup_uses(&mut self) {
        // First remove duplicates in the wildcards
        let mut wildcards = Vec::new();
        for wildcard in mem::take(&mut self.wildcards) {
            let is_dup = wildcards.iter().find(|use_path| **use_path == wildcard).is_some();
            if !is_dup {
                wildcards.push(wildcard);
            }
        }
        self.wildcards = wildcards;

        // Then handle explicit paths
        let mut uses = Vec::new();
        for use_path in mem::take(&mut self.uses) {
            let mut is_dup = uses.iter().find(|cur_use| **cur_use == use_path).is_some();
            if !is_dup {
                is_dup = self.wildcards.iter().find(|wildcard| {
                    use_path.kind == UsePathKind::Wildcard &&
                    use_path.lib_path == wildcard.lib_path &&
                    use_path.path.parent() == wildcard.path
                }).is_some();
            }
            if !is_dup {
                uses.push(use_path);
            }
        }
        self.uses = uses;


        for (_, sub_table) in &mut self.sub_tables {
            sub_table.remove_dup_uses();
        }
    }

    fn validate_paths(&self, sym_table: &RootSymbolTable, scope: &mut Scope, invalid_paths: &mut Vec<(Scope, UsePath)>) {
        for use_path in &self.uses {
            let scope = use_path.path.parent();
            let name = use_path.path.last().unwrap().name.clone();
            if sym_table.get_symbol(Some(&use_path.lib_path), &scope, &name).is_none() {
                invalid_paths.push((Scope::new(), use_path.clone()));
            }
        }
        for wildcard in &self.wildcards {
            let scope = wildcard.path.parent();
            let name = wildcard.path.last().unwrap().name.clone();
            if sym_table.get_symbol(Some(&wildcard.lib_path), &scope, &name).is_none() {
                invalid_paths.push((Scope::new(), wildcard.clone()));
            }
        }

        for (segment, sub_table) in &self.sub_tables {
            scope.push_iden(segment.clone());
            sub_table.validate_paths(sym_table, scope, invalid_paths);
            scope.pop();
        }
    }

    fn get_use_paths(&self, scope: &[PathIden], use_paths: &mut Vec<UsePath>) -> Option<UsePath> {
        // First get the inner uses
        let file_root = if !scope.is_empty() {
            if let Some(sub_table) = self.sub_tables.get(&scope[0]) {
                sub_table.get_use_paths(&scope[1..], use_paths)
            } else {
                None
            }
        } else {
            None
        };

        // If we already hit an inner file root, no need to look further
        if file_root.is_some() {
            return file_root
        }

        // Otherwise we just add our scope-local uses
        use_paths.extend_from_slice(&self.uses);
        use_paths.extend_from_slice(&self.wildcards);
        
        // Make sure to mark the last use in this scope, this is use when resolving the symbol and determining if there is a duplicate symbols in the same scope
        if let Some(last_use_path) = use_paths.last_mut() {
            last_use_path.last_in_scope = true;
        }

        self.file_use_root.clone()
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