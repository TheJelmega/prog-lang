#![allow(unused)]

use std::{
    collections::HashMap,
    fmt::{self, Write},
    path::PathBuf,
    sync::Arc
};
use parking_lot::RwLock;

use crate::type_system::{Type, TypeHandle, TypeRef};

use super::{IndentLogger, LibraryPath, RootUseTable, Scope, ScopeSegment, Visibility};


// =============================================================

#[derive(Clone, Debug)]
pub struct SymbolPath {
    pub lib:   LibraryPath,
    pub scope: Scope,
    pub name:  String,
}

impl SymbolPath {
    pub fn new() -> Self {
        Self {
            lib: LibraryPath::new(),
            scope: Scope::new(),
            name: String::new(),
        }
    }
}

impl fmt::Display for SymbolPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}{}{}", self.lib, self.scope, if self.scope.is_empty() { "" } else  { "." }, self.name)
    }
}

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
    TypeGeneric(TypeGenericSymbol),
    ValueGeneric(ValueGenericSymbol),
}

impl Symbol {
    pub fn kind_str(&self) -> &'static str {
        match self {
            Symbol::Module(_)       => "module",
            Symbol::Precedence(_)   => "precedence",
            Symbol::Function(_)     => "function",
            Symbol::TypeAlias(_)    => "type alias",
            Symbol::DistinctType(_) => "distinct type",
            Symbol::OpaqueType(_)   => "opaque type",
            Symbol::Struct(_)       => "struct",
            Symbol::Union(_)        => "union",
            Symbol::AdtEnum(_)      => "ADT enum",
            Symbol::FlagEnum(_)     => "flag enum",
            Symbol::Bitfield(_)     => "bitfield",
            Symbol::Const(_)        => "const",
            Symbol::Static(_)       => "static",
            Symbol::Property(_)     => "property",
            Symbol::Trait(_)        => "trait",
            Symbol::Impl(_)         => "impl",
            Symbol::TypeGeneric(_)  => "type generic",
            Symbol::ValueGeneric(_) => "value generic"
        }
    }

    pub fn path(&self) -> &SymbolPath {
        match self {
            Symbol::Module(sym)       => &sym.path,
            Symbol::Precedence(sym)   => &sym.path,
            Symbol::Function(sym)     => &sym.path,
            Symbol::TypeAlias(sym)    => &sym.path,
            Symbol::DistinctType(sym) => &sym.path,
            Symbol::OpaqueType(sym)   => &sym.path,
            Symbol::Struct(sym)       => &sym.path,
            Symbol::Union(sym)        => &sym.path,
            Symbol::AdtEnum(sym)      => &sym.path,
            Symbol::FlagEnum(sym)     => &sym.path,
            Symbol::Bitfield(sym)     => &sym.path,
            Symbol::Const(sym)        => &sym.path,
            Symbol::Static(sym)       => &sym.path,
            Symbol::Property(sym)     => &sym.path,
            Symbol::Trait(sym)        => &sym.path,
            Symbol::Impl(sym)         => &sym.path,
            Symbol::TypeGeneric(sym)  => &sym.path,
            Symbol::ValueGeneric(sym) => &sym.path,
        }
    }

    pub fn get_type(&self) -> Option<&TypeHandle> {
        match self {
            Symbol::Module(sym) => None,
            Symbol::Precedence(sym) => None,
            Symbol::Function(sym) => sym.ty.as_ref(),
            Symbol::TypeAlias(sym) => sym.ty.as_ref(),
            Symbol::DistinctType(sym) => sym.ty.as_ref(),
            Symbol::OpaqueType(sym) => sym.ty.as_ref(),
            Symbol::Struct(sym) => sym.ty.as_ref(),
            Symbol::Union(sym) => sym.ty.as_ref(),
            Symbol::AdtEnum(sym) => sym.ty.as_ref(),
            Symbol::FlagEnum(sym) => sym.ty.as_ref(),
            Symbol::Bitfield(sym) => sym.ty.as_ref(),
            Symbol::Const(sym) => sym.ty.as_ref(),
            Symbol::Static(sym) => sym.ty.as_ref(),
            Symbol::Property(sym) => sym.ty.as_ref(),
            Symbol::Trait(sym) => sym.ty.as_ref(),
            Symbol::Impl(sym) => None,
            Symbol::TypeGeneric(sym) => sym.ty.as_ref(),
            Symbol::ValueGeneric(sym) => sym.ty.as_ref(),
        }
    }
}

//----------------------------------------------

pub struct ModuleSymbol {
    pub path:      SymbolPath,
    pub file_path: PathBuf,
}

//----------------------------------------------

pub struct PrecedenceSymbol {
    pub path:  SymbolPath,
    pub id:    u16,
}

//----------------------------------------------

pub struct FunctionSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,
}

//----------------------------------------------

pub struct TypeAliasSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,
}

pub struct DistinctTypeSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,
    
}

pub struct OpaqueTypeSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,
    
}

//----------------------------------------------

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
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub kind: StructKind,
    pub ty:   Option<TypeHandle>,
}

//----------------------------------------------

pub struct UnionSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,

}


pub struct AdtEnumSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,

}

//----------------------------------------------

pub struct FlagEnumSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,

}

//----------------------------------------------

pub struct BitfieldSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,

}

//----------------------------------------------

pub struct ConstSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,
}

//----------------------------------------------

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
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub kind: StaticKind,
    pub ty:   Option<TypeHandle>,
}

//----------------------------------------------

pub struct PropertySymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
    pub ty:   Option<TypeHandle>,
}


//----------------------------------------------

#[derive(Clone)]
pub enum TraitItemKind {
    Function,
    Method,
    TypeAlias,
    Const,
    Property {
        get:     bool,
        ref_get: bool,
        mut_set: bool,
        set:     bool,
    },
}

#[derive(Clone)]
pub struct TraitItemRecord {
    pub name:        String,
    pub kind:        TraitItemKind,
    pub has_default: bool,
    pub idx:         usize,
}

pub struct TraitSymbol {
    pub path:    SymbolPath,
    pub vis:     Visibility,
    pub ty:      Option<TypeHandle>,
    pub dag_idx: u32,
    pub items:   Vec<TraitItemRecord>,
}

//----------------------------------------------

pub struct ImplSymbol {
    pub path: SymbolPath,
    pub vis:  Visibility,
}

//----------------------------------------------

pub struct TypeGenericSymbol {
    pub path:    SymbolPath,
    pub vis:     Visibility,
    pub ty:      Option<TypeHandle>,
    pub in_pack: bool,
}

pub struct ValueGenericSymbol {
    pub path:    SymbolPath,
    pub vis:     Visibility,
    pub ty:      Option<TypeHandle>,
    pub in_pack: bool,
}

//----------------------------------------------

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

//==============================================================================================================================

pub struct RootSymbolTable {
    cur_lib:  LibraryPath,
    tables:   HashMap<LibraryPath, SymbolTable>,
    ty_table: HashMap<TypeHandle, Vec<SymbolRef>>,
}

impl RootSymbolTable {
    pub fn new(cur_lib: LibraryPath) -> Self {
        let mut tables = HashMap::new();
        tables.insert(cur_lib.clone(), SymbolTable::new());

        Self {
            cur_lib,
            tables,
            ty_table: HashMap::new(),
        }
    }

    
    pub fn add_module(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, file_path: PathBuf) -> SymbolRef {
        let sym = Symbol::Module(ModuleSymbol{
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            file_path,
        });
        self.add_symbol(scope, name, &[], sym)
    }

    pub fn add_precedence(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Precedence(PrecedenceSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            id: u16::MAX,
        }); 
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_function(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Function(FunctionSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_type_alias(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::TypeAlias(TypeAliasSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_distinct_type(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::DistinctType(DistinctTypeSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_opaque_type(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::OpaqueType(OpaqueTypeSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_struct(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, kind: StructKind) -> SymbolRef {
        let sym = Symbol::Struct(StructSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            kind,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_union(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Union(UnionSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_adt_enum(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::AdtEnum(AdtEnumSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_flag_enum(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::FlagEnum(FlagEnumSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_bitfield(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Bitfield(BitfieldSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_const(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Const(ConstSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_static(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, kind: StaticKind) -> SymbolRef {
        let sym = Symbol::Static(StaticSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            kind,
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_property(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Property(PropertySymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_trait(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Trait(TraitSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            dag_idx: u32::MAX,
            items: Vec::new(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_impl(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str) -> SymbolRef {
        let sym = Symbol::Impl(ImplSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_type_generic(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, in_pack: bool) -> SymbolRef {
        let sym = Symbol::TypeGeneric(TypeGenericSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            in_pack,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_value_generic(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, in_pack: bool) -> SymbolRef {
        let sym = Symbol::ValueGeneric(ValueGenericSymbol {
            path: SymbolPath {
                lib: lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope: scope.clone(),
                name: name.to_string(),
            },
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            in_pack,
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    fn add_symbol(&mut self, scope: &Scope, name: &str, params: &[String], sym: Symbol) -> SymbolRef {
        // SAFETY: We always add the table for `self.cur_lib`, so we know it exists
        let cur_table = self.tables.get_mut(&self.cur_lib).unwrap();
        cur_table.add_symbol(scope, name, params, sym)
    }


    pub fn associate_impl_with_ty(&mut self, ty: TypeHandle, sym: SymbolRef) {
        assert!(matches!(*sym.read(), Symbol::Impl(_)));

        let entry = self.ty_table.entry(ty);
        let assoc_impls = entry.or_default();
        assoc_impls.push(sym);
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
    pub fn get_symbol_with_uses(&self, use_table: &RootUseTable, cur_scope: &Scope, cur_sub_scope: Option<&Scope>, sym_path: &Scope) -> Option<SymbolRef> { 
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
        if let Some(sub_scope) = cur_sub_scope {
            use_loc_path.extend(sub_scope);
        }
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
        let end = self.tables.len() - 1;
        for (idx, (lib_path, table)) in self.tables.iter().enumerate() {
            logger.set_last_at_indent_if(idx == end && self.ty_table.is_empty());

            logger.log_indented("Table", |logger| {
                if let Some(group) = &lib_path.group {
                    logger.prefixed_log_fmt(format_args!("Group: {group}\n"));
                }
                logger.prefixed_log_fmt(format_args!("Package: {}\n", &lib_path.package));
                logger.prefixed_log_fmt(format_args!("Library: {}\n", &lib_path.library));

                SymbolTableLogger::log_table(logger, table)
            });
        }

        if !self.ty_table.is_empty() {
            let end = self.ty_table.len() - 1;
            for (idx, (ty, impls)) in self.ty_table.iter().enumerate() {
                logger.set_last_at_indent_if(idx == end);
                
                logger.log_indented("Type <-> Impl Symbol association", |logger| {
                    logger.prefixed_log_fmt(format_args!("Type: {ty}\n"));
                    logger.log_indented_slice(impls, |logger, sym| {
                        let sym = sym.read();
                        logger.prefixed_log_fmt(format_args!("Impl: {}\n", sym.path()))
                    });
                })
            }
        }
    }
}



struct SymbolTableLogger;

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
            Symbol::Module(sym) => {
                logger.prefixed_logln("Module");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("File Path: {}\n", sym.file_path.to_str().unwrap()));
            },
            Symbol::Precedence(sym) => {
                logger.prefixed_logln("Precedence");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Id: {}\n", sym.id));
            },
            Symbol::Function(sym) => {
                logger.prefixed_logln("Function");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::TypeAlias(sym) => {
                logger.prefixed_logln("Type Alias");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::DistinctType(sym) => {
                logger.prefixed_logln("Distinct Type");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::OpaqueType(sym) => {
                logger.prefixed_logln("Opaque Type");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::Struct(sym) => {
                logger.prefixed_logln("Struct");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
                logger.prefixed_log_fmt(format_args!("Kind: {}\n", sym.kind));
            }, 
            Symbol::Union(sym) => {   
                logger.prefixed_logln("Union");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::AdtEnum(sym) => {
                logger.prefixed_logln("ADT enum");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::FlagEnum(sym) => {
                logger.prefixed_logln("Flag");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::Bitfield(sym) => {
                logger.prefixed_logln("Bitfield");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::Const(sym) => {
                logger.prefixed_logln("Const");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::Static(sym) => {
                logger.prefixed_logln("Static");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
                logger.prefixed_log_fmt(format_args!("Kind: {}\n", sym.kind));
            },
            Symbol::Property(sym) => {
                logger.prefixed_logln("Property");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::Trait(sym) => {
                logger.prefixed_logln("Trait");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::Impl(sym) => {
                logger.prefixed_logln("Impl");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
            },
            Symbol::TypeGeneric(sym) => {
                logger.prefixed_logln("Type Generic");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
                logger.prefixed_log_fmt(format_args!("In Parameter Pack: {}\n", sym.in_pack));
            },
            Symbol::ValueGeneric(sym) => {
                logger.prefixed_logln("Value Generic");
                logger.push_indent();
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
                logger.prefixed_log_fmt(format_args!("Visibility: {}\n", sym.vis));
                logger.prefixed_log_fmt(format_args!("In Parameter Pack: {}\n", sym.in_pack));
            },
        }

        if let Some(sub_table) = sub_table {
            Self::log_table(logger, sub_table);
        }

        logger.pop_indent();
    }
}

