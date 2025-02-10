use std::{
    collections::HashMap,
    fmt::{self, Write},
    path::PathBuf,
    sync::Arc
};
use parking_lot::RwLock;

use super::{IndentLogger, LibraryPath, RootUseTable, Scope, ScopeSegment, UseTable};

// =============================================================

pub enum Symbol {
    Module(ModuleSymbol),
    Precedence(PrecedenceSymbol),
    Function(FunctionSymbol),
    TypeAlias(TypeAliasSymbol),
    DistinctType(DistinctTypeSymbol),
    OpaqueType(OpaqueTypeSymbol),
    Struct(StructSymbol),
    Union(UnionSymbol),
    AdtEnum(AdtEnumSymbol),
    FlagEnum(FlagEnumSymbol),
    Bitfield(BitfieldSymbol),
    Const(ConstSymbol),
    Static(StaticSymbol),
    Property(PropertySymbol),
    Trait(TraitSymbol),
    Impl(ImplSymbol),
}

pub struct ModuleSymbol {
    pub scope:     Scope,
    pub name:      String,
    pub path:      PathBuf,
}

pub struct PrecedenceSymbol {
    pub scope: Scope,
    pub name:  String,
    pub id:    u16,
}

pub struct FunctionSymbol {
    pub scope:     Scope,
    pub name:      String,

    pub sub_table: SymbolTable,
}

pub struct TypeAliasSymbol {
    pub scope: Scope,
    pub name:  String,

}

pub struct DistinctTypeSymbol {
    pub scope: Scope,
    pub name:  String,
    
}

pub struct OpaqueTypeSymbol {
    pub scope: Scope,
    pub name:  String,
    
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StructKind {
    Normal,
    Tuple,
    Unit,
}

impl fmt::Display for StructKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Tuple  => write!(f, "tuple"),
            Self::Unit   => write!(f, "union"),
        }
    }
}

pub struct StructSymbol {
    pub scope: Scope,
    pub name:  String,
    pub kind:  StructKind,
}

pub struct UnionSymbol {
    pub scope: Scope,
    pub name:  String,

}

pub struct AdtEnumSymbol {
    pub scope: Scope,
    pub name:  String,

}

pub struct FlagEnumSymbol {
    pub scope: Scope,
    pub name:  String,

}

pub struct BitfieldSymbol {
    pub scope: Scope,
    pub name:  String,

}

pub struct ConstSymbol {
    pub scope: Scope,
    pub name:  String,

}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StaticKind {
    Normal,
    Extern,
    Tls,
}

impl fmt::Display for StaticKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Extern => write!(f, "static"),
            Self::Tls    => write!(f, "tls"),
        }
    }
}

pub struct StaticSymbol {
    pub scope: Scope,
    pub name:  String,
    pub kind:  StaticKind,
}

pub struct PropertySymbol {
    pub scope: Scope,
    pub name:  String,
}

pub struct TraitSymbol {
    pub scope:     Scope,
    pub name:      String,
}

pub struct ImplSymbol {
    pub scope:     Scope,
    pub name:      String,
}

pub type SymbolRef = Arc<RwLock<Symbol>>;

pub struct SymbolTable {
    symbols:    HashMap<String, Vec<(Vec<String>, SymbolRef)>>,
    sub_tables: HashMap<ScopeSegment, SymbolTable>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            sub_tables: HashMap::new(),
        }
    }

    fn add_symbol(&mut self, scope: &Scope, name: &str, params: &[String], sym: Symbol) -> SymbolRef {
        let sym = Arc::new(RwLock::new(sym));
        self.add_symbol_(scope, name, params, sym.clone());
        sym
    }

    fn add_symbol_(&mut self, scope: &Scope, name: &str, params: &[String], sym: SymbolRef) {
        let sub_table = self.get_or_insert_sub_table(scope.segments());
        let entry = sub_table.symbols.entry(name.to_string());
        let syms = entry.or_insert(Vec::new());
        syms.push((Vec::from(params), sym));
    }

    pub fn get_symbol(&self, scope: &Scope, name: &str) -> Option<SymbolRef> {
        let sub_table = self.get_sub_table(scope.segments())?;
        sub_table.get_symbol_from_name(name)
    }

    fn get_sub_table(&self, segments: &[ScopeSegment]) -> Option<&SymbolTable> {
        if segments.is_empty() {
            return Some(self);
        }

        let sub_table = self.sub_tables.get(&segments[0])?;
        sub_table.get_sub_table(&segments[1..])
    }

    fn get_direct_sub_table_from_name(&self, name: &str) -> Option<&SymbolTable> {
        for (segment, table) in &self.sub_tables {
            if segment.name == name {
                return Some(table);
            }
        }
        None
    }

    fn get_or_insert_sub_table(&mut self, segments: &[ScopeSegment]) -> &mut SymbolTable {
        if segments.is_empty() {
            return self;
        }
        
        let entry = self.sub_tables.entry(segments[0].clone());
        let sub_table = entry.or_insert(SymbolTable::new());
        sub_table.get_or_insert_sub_table(&segments[1..])
    }

    // Get symbol from name alone, will only return reference if only 1 symbol with the name exists, regardless of func parameters
    fn get_symbol_from_name(&self, name: &str) -> Option<SymbolRef> {
        let possible_syms = self.symbols.get(name)?;
        if possible_syms.len() == 1 {
            Some(possible_syms[0].1.clone())
        } else {
            None
        }
    }
}
pub struct RootSymbolTable {
    cur_lib: LibraryPath,
    tables:  HashMap<LibraryPath, SymbolTable>
}

impl RootSymbolTable {
    pub fn new(cur_lib: LibraryPath) -> Self {
        let mut tables = HashMap::new();
        tables.insert(cur_lib.clone(), SymbolTable::new());

        Self {
            cur_lib,
            tables,
        }
    }

    
    pub fn add_module(&mut self, scope: &Scope, name: String, file_path: PathBuf) -> SymbolRef {
        let sym = Symbol::Module(ModuleSymbol{
            scope: scope.clone(),
            name: name.clone(),
            path: file_path,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_precedence(&mut self, scope: &Scope, name: String, id: u16) -> SymbolRef {
        let sym = Symbol::Precedence(PrecedenceSymbol {
            scope: scope.clone(),
            name: name.clone(),
            id,
        }); 
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_function(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Function(FunctionSymbol {
            scope: scope.clone(),
            name: name.clone(),

            sub_table: SymbolTable::new(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_type_alias(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::TypeAlias(TypeAliasSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_distinct_type(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::DistinctType(DistinctTypeSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_opaque_type(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::OpaqueType(OpaqueTypeSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_struct(&mut self, scope: &Scope, name: String, kind: StructKind) -> SymbolRef {
        let sym = Symbol::Struct(StructSymbol {
            scope: scope.clone(),
            name: name.clone(),
            kind,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_union(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Union(UnionSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_adt_enum(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::AdtEnum(AdtEnumSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_flag_enum(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::FlagEnum(FlagEnumSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_bitfield(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Bitfield(BitfieldSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_const(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Const(ConstSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_static(&mut self, scope: &Scope, name: String, kind: StaticKind) -> SymbolRef {
        let sym = Symbol::Static(StaticSymbol {
            scope: scope.clone(),
            name: name.clone(),
            kind,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_property(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Property(PropertySymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_trait(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Trait(TraitSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_impl(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Impl(ImplSymbol {
            scope: scope.clone(),
            name: name.clone(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    fn add_symbol(&mut self, scope: &Scope, name: &str, params: &[String], sym: Symbol) -> SymbolRef {
        // SAFETY: We always add the table for `self.cur_lib`, so we know it exists
        let cur_table = self.tables.get_mut(&self.cur_lib).unwrap();
        cur_table.add_symbol(scope, name, params, sym)
    }

    pub fn get_symbol(&self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> Option<SymbolRef> {
        let lib = lib.unwrap_or(&self.cur_lib);
        let table = self.tables.get(lib)?;
        table.get_symbol(scope, name)
    }

    /// Get a symbol, while also searching all available scopes
    /// 
    /// * `cur_scope` - Scope of the symbol being processed
    /// * `cur_sub_scope` - Scope within the symbol being processed (e.g. scope relative to a function)
    /// * `sym_path` - Path of the symbol as it occurs within code
    // TODO: lib path
    pub fn get_symbol_with_uses(&self, use_table: &RootUseTable, cur_scope: &Scope, cur_sub_scope: &Scope, sym_path: &Scope) -> Option<SymbolRef> { 
        assert!(!sym_path.is_empty());

        let sym_name = &sym_path.last().unwrap().name;
        let sym_scope = sym_path.parent();

        // Look into the current scope first
        let cur_table = self.tables.get(&self.cur_lib).unwrap();
        if let Some(local_sub_table) = cur_table.get_sub_table(cur_scope.segments()) {
            if let Some(sym) = local_sub_table.get_symbol(&sym_scope, &sym_name) {
                return Some(sym);
            }
        }

        // Then look into the use table
        let mut use_loc_path = cur_scope.clone();
        use_loc_path.extend(cur_sub_scope);
        let mut found_sym = None;
        use_table.with_uses(&use_loc_path, |use_path| {
            let root = sym_path.root().unwrap();
            let mut act_sym_path = use_path.path.clone();
            if let Some(alias) = &use_path.alias {
                if !root.params.is_empty() || root.name != *alias {
                    return false;
                }
                act_sym_path.extend(&sym_path.sub_path());
            } else {
                act_sym_path.extend(sym_path);
            };

            let Some(table) = self.tables.get(&use_path.lib_path) else {
                return false;
            };

            if let Some(sym) = table.get_symbol(&act_sym_path.parent(), sym_name) {
                if found_sym.is_some() {
                    todo!("Error, ambiguous symbol");
                }
                found_sym = Some(sym);
                return true;
            }
            false
        });

        found_sym
    }

    pub fn log(&self) {
        let mut logger = IndentLogger::new("    ", "|   ", "+---");
        for (lib_path, table) in &self.tables {
            logger.log_indented("Table", |logger| {
                if let Some(group) = &lib_path.group {
                    logger.prefixed_log_fmt(format_args!("Group: {group}\n"));
                }
                logger.prefixed_log_fmt(format_args!("Package: {}\n", &lib_path.package));
                logger.prefixed_log_fmt(format_args!("Library: {}\n", &lib_path.library));

                logger.set_last_at_indent();
                logger.log_indented("Tables", |logger| SymbolTableLogger::log_table(logger, table))
            });
        }

    }
}



struct SymbolTableLogger {
}

#[allow(unused)]
impl SymbolTableLogger {
    fn log_table(logger: &mut IndentLogger, table: &SymbolTable) {
        for (idx, (name, symbols)) in table.symbols.iter().enumerate() {
            if idx == table.symbols.len() - 1 {
                logger.set_last_at_indent();
            }

            if symbols.len() == 1 && symbols[0].0.is_empty() {
                let sub_table = table.get_direct_sub_table_from_name(name);
                Self::log_symbol(logger, &symbols[0].1.read(), sub_table);
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

                    let segment = ScopeSegment {
                        name: name.clone(),
                        params: params.clone(),
                    };
                    let sub_table = table.get_sub_table(&[segment]);
                    logger.log_indented(&params_s, |logger| Self::log_symbol(logger, &sym.read(), sub_table));
                }
            }

        }
    }

    fn log_symbol(logger: &mut IndentLogger, sym: &Symbol, sub_table: Option<&SymbolTable>) {
        match sym {
            Symbol::Module(sym) =>
            {
                logger.prefixed_logln("Module");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path.to_str().unwrap()));
            },
            Symbol::Precedence(sym) =>
            {
                logger.prefixed_logln("Precedence");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Id: {}\n", sym.id));
            },
            Symbol::Function(sym) =>
            {
                logger.prefixed_logln("Function");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::TypeAlias(sym) =>
            {
                logger.prefixed_logln("Type Alias");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::DistinctType(sym) =>
            {
                logger.prefixed_logln("Distinct Type");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::OpaqueType(sym) =>
            {
                logger.prefixed_logln("Opaque Type");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::Struct(sym) =>
            {
                logger.prefixed_logln("Struct");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Kind: {}\n", sym.kind));
            }, 
            Symbol::Union(sym) =>
            {   
                logger.prefixed_logln("Union");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::AdtEnum(sym) =>
            {
                logger.prefixed_logln("ADT enum");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::FlagEnum(sym) =>
            {
                logger.prefixed_logln("Flag");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::Bitfield(sym) =>
            {
                logger.prefixed_logln("Bitfield");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::Const(sym) =>
            {
                logger.prefixed_logln("Const");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::Static(sym) =>
            {
                logger.prefixed_logln("Static");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Kind: {}\n", sym.kind));
            },
            Symbol::Property(sym) =>
            {
                logger.prefixed_logln("Property");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::Trait(sym) =>
            {
                logger.prefixed_logln("Trait");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
            Symbol::Impl(sym) =>
            {
                logger.prefixed_logln("Impl");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            },
        }

        if let Some(sub_table) = sub_table {
            Self::log_table(logger, sub_table);
        }

        logger.pop_indent();
    }
}

