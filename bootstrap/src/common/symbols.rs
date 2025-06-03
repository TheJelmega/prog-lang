#![allow(unused)]

use std::{
    collections::HashMap,
    fmt::{self, Write},
    path::PathBuf,
    sync::{Arc, Weak}
};
use parking_lot::RwLock;

use crate::{common::UsePathKind, lexer::{Punctuation, PuncutationTable}, type_system::{Type, TypeHandle, TypeRef}};

use super::{IndentLogger, LibraryPath, OpType, PathIden, PrecedenceAssocKind, RootUseTable, Scope, SymbolPath, Visibility};

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
    OpSet(OpSetSymbol),
    Operator(OperatorSymbol),
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
            Symbol::ValueGeneric(_) => "value generic",
            Symbol::OpSet(_)       => "operator item",
            Symbol::Operator(_)     => "operator",
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
            Symbol::OpSet(sym)       => &sym.path,
            Symbol::Operator(sym)     => &sym.path,
            
        }
    }

    pub fn get_type(&self) -> Option<&TypeHandle> {
        match self {
            Symbol::Module(sym)       => None,
            Symbol::Precedence(sym)   => None,
            Symbol::Function(sym)     => sym.ty.as_ref(),
            Symbol::TypeAlias(sym)    => sym.ty.as_ref(),
            Symbol::DistinctType(sym) => sym.ty.as_ref(),
            Symbol::OpaqueType(sym)   => sym.ty.as_ref(),
            Symbol::Struct(sym)       => sym.ty.as_ref(),
            Symbol::Union(sym)        => sym.ty.as_ref(),
            Symbol::AdtEnum(sym)      => sym.ty.as_ref(),
            Symbol::FlagEnum(sym)     => sym.ty.as_ref(),
            Symbol::Bitfield(sym)     => sym.ty.as_ref(),
            Symbol::Const(sym)        => sym.ty.as_ref(),
            Symbol::Static(sym)       => sym.ty.as_ref(),
            Symbol::Property(sym)     => sym.ty.as_ref(),
            Symbol::Trait(sym)        => sym.ty.as_ref(),
            Symbol::Impl(sym)         => None,
            Symbol::TypeGeneric(sym)  => sym.ty.as_ref(),
            Symbol::ValueGeneric(sym) => sym.ty.as_ref(),
            Symbol::OpSet(sym)       => None,
            Symbol::Operator(sym)     => None,
            
        }
    }
}

//----------------------------------------------

pub struct ModuleSymbol {
    pub path:      SymbolPath,
    pub file_path: PathBuf,
}

//----------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PrecedenceOrderKind {
    User,
    Lowest,
    Highest,
}

impl fmt::Display for PrecedenceOrderKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrecedenceOrderKind::User    => write!(f, "user"),
            PrecedenceOrderKind::Lowest  => write!(f, "lowest"),
            PrecedenceOrderKind::Highest => write!(f, "highest"),
        }
    }
}

pub struct PrecedenceSymbol {
    pub path:        SymbolPath,
    pub order_kind:  PrecedenceOrderKind,
    pub assoc:       PrecedenceAssocKind,
    pub lower_than:  Option<WeakSymbolRef>,
    pub higher_than: Option<WeakSymbolRef>,

    // dag id
    pub id:          u16,
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

pub struct OpSetSymbol {
    pub path:              SymbolPath,
    pub assoc_trait:       Option<WeakSymbolRef>,
    pub precedence:        Option<WeakSymbolRef>,
    pub bases:             Vec<WeakSymbolRef>,

    pub has_generics:      bool,
    pub has_output_alias:  bool,
    pub parent_has_output: bool,
}

pub struct OperatorSymbol {
    pub path:         SymbolPath,
    pub op_ty:        OpType,
    pub op:           Punctuation,

    pub assoc_method: Option<WeakSymbolRef>
}

//----------------------------------------------

pub type SymbolRef = Arc<RwLock<Symbol>>;
pub type WeakSymbolRef = Weak<RwLock<Symbol>>;

//==============================================================================================================================
#[derive(Clone, Debug)]
pub enum SymbolLookupKind {
    Precedence,
    OperatorSet,
    Symbol,
}

impl fmt::Display for SymbolLookupKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolLookupKind::Precedence => write!(f, "precedence"),
            SymbolLookupKind::OperatorSet   => write!(f, "operator"),
            SymbolLookupKind::Symbol     => write!(f, "symbol"),
        }
    }
}

#[derive(Clone, Debug)]
pub enum SymbolLookupError {
    Unknown { path: Scope, kind: SymbolLookupKind },
    Ambiguous {
        path: Scope,
        possible_paths: Vec<SymbolPath>,
    }
}

impl fmt::Display for SymbolLookupError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SymbolLookupError::Unknown { path ,kind } => write!(f, "Unknown {kind}: {path}"),
            SymbolLookupError::Ambiguous { path, possible_paths } => {
                write!(f, "Ambiguous symbol for '{path}', possible  symbols: ")?;
                for (idx, path) in possible_paths.iter().enumerate() {
                    if idx != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{path}")?;
                }
                Ok(())
            },
        }
    }
}

pub struct SymbolTable {
    symbols:    HashMap<String, Vec<(Vec<String>, SymbolRef)>>,
    sub_tables: HashMap<PathIden, SymbolTable>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
            sub_tables: HashMap::new(),
        }
    }

    fn add_symbol(&mut self, scope: &Scope, iden: PathIden, sym: Symbol) -> SymbolRef {
        let sym = Arc::new(RwLock::new(sym));
        self.add_symbol_(scope, iden, sym.clone());
        sym
    }

    fn add_symbol_(&mut self, scope: &Scope, iden: PathIden, sym: SymbolRef) {
        let sub_table = self.get_or_insert_sub_table(scope.idens());
        let entry = sub_table.symbols.entry(iden.name);
        let syms = entry.or_insert(Vec::new());
        syms.push((Vec::from(iden.params), sym));
    }

    pub fn get_direct_symbol(&self, scope: &Scope, name: &str) -> Option<SymbolRef> {
        let sub_table = self.get_sub_table(scope.idens())?;
        sub_table.get_symbol_from_name(name)
    }

    fn get_sub_table(&self, segments: &[PathIden]) -> Option<&SymbolTable> {
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

    fn get_or_insert_sub_table(&mut self, segments: &[PathIden]) -> &mut SymbolTable {
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
    cur_lib:     LibraryPath,
    tables:      HashMap<LibraryPath, SymbolTable>,
    ty_table:    HashMap<TypeHandle, Vec<SymbolRef>>,
    precedences: HashMap<LibraryPath, HashMap<String, SymbolRef>>,
    operators:   HashMap<LibraryPath, HashMap<String, (SymbolRef, HashMap<String, SymbolRef>)>>,
}

impl RootSymbolTable {
    pub fn new(cur_lib: LibraryPath) -> Self {
        let mut tables = HashMap::new();
        tables.insert(cur_lib.clone(), SymbolTable::new());

        Self {
            cur_lib,
            tables,
            ty_table: HashMap::new(),
            precedences: HashMap::new(),
            operators: HashMap::new(),
        }
    }

    
    pub fn add_module(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, file_path: PathBuf) -> SymbolRef {
        let iden = PathIden::new(name.to_string(), Vec::new(), Vec::new());
        let sym = Symbol::Module(ModuleSymbol{
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            file_path,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_function(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Function(FunctionSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_type_alias(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::TypeAlias(TypeAliasSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_distinct_type(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::DistinctType(DistinctTypeSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_opaque_type(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::OpaqueType(OpaqueTypeSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_struct(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden, kind: StructKind) -> SymbolRef {
        let sym = Symbol::Struct(StructSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            kind,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_union(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Union(UnionSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_adt_enum(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::AdtEnum(AdtEnumSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_flag_enum(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::FlagEnum(FlagEnumSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_bitfield(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Bitfield(BitfieldSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_const(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Const(ConstSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_static(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, kind: StaticKind) -> SymbolRef {
        let iden = PathIden::new(name.to_string(), Vec::new(), Vec::new());
        let sym = Symbol::Static(StaticSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            kind,
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_property(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Property(PropertySymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_trait(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Trait(TraitSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            dag_idx: u32::MAX,
            items: Vec::new(),
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_impl(&mut self, lib: Option<&LibraryPath>, scope: &Scope, iden: PathIden) -> SymbolRef {
        let sym = Symbol::Impl(ImplSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_type_generic(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, in_pack: bool) -> SymbolRef {
        let iden = PathIden::new(name.to_string(), Vec::new(), Vec::new());
        let sym = Symbol::TypeGeneric(TypeGenericSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            in_pack,
        });
        self.add_symbol(scope, iden, sym)
    }

    pub fn add_value_generic(&mut self, lib: Option<&LibraryPath>, scope: &Scope, name: &str, in_pack: bool) -> SymbolRef {
        let iden = PathIden::new(name.to_string(), Vec::new(), Vec::new());
        let sym = Symbol::ValueGeneric(ValueGenericSymbol {
            path: SymbolPath::new(
                lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone()),
                scope.clone(),
                iden.clone(),
            ),
            vis: Visibility::Public, // Placeholder visibility
            ty: None,
            in_pack,
        });
        self.add_symbol(scope, iden, sym)
    }

    fn add_symbol(&mut self, scope: &Scope, iden: PathIden, sym: Symbol) -> SymbolRef {
        // SAFETY: We always add the table for `self.cur_lib`, so we know it exists
        let cur_table = self.tables.get_mut(&self.cur_lib).unwrap();
        cur_table.add_symbol(scope, iden, sym)
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
        table.get_direct_symbol(scope, name)
    }

    // TODO: Go over use table and make sure all paths actually point to valid symbols

    /// Get a symbol, while also searching all available scopes
    /// 
    /// * `cur_scope` - Scope of the symbol being processed
    /// * `cur_sub_scope` - Scope within the symbol being processed (e.g. scope relative to a function), used for resolving all scoped `use` statements
    /// * `sym_path` - Path of the symbol as it occurs within code
    // TODO: lib path
    pub fn get_symbol_with_uses(&self, use_table: &RootUseTable, cur_scope: &Scope, use_cur_sub_scope: Option<&Scope>, sym_path: &Scope) -> Result<SymbolRef, SymbolLookupError> { 
        assert!(!sym_path.is_empty());

        let sym_name = &sym_path.last().unwrap().name;
        let sym_scope = sym_path.parent();

        // Look into the current scope first
        let cur_table = self.tables.get(&self.cur_lib).unwrap();
        if let Some(local_sub_table) = cur_table.get_sub_table(cur_scope.idens()) {
            if let Some(sym) = local_sub_table.get_direct_symbol(&sym_scope, &sym_name) {
                return Ok(sym);
            }
        }

        // The get all possible use paths and try to find it there
        let mut use_lookup_path = cur_scope.clone();
        if let Some(sub_scope) = use_cur_sub_scope {
            use_lookup_path.extend(sub_scope);
        }
        let uses = use_table.get_use_paths(&use_lookup_path);

        let mut found_syms = Vec::new();
        for use_path in uses {
            // We will look into the library pointed by the usepath, so already get it here, as we might need it when generating the search path
            let table = self.tables.get(&use_path.lib_path).expect("If you see this, it means the use table was not validated before being used to look up a symbol");
            
            let mut search_path = use_path.path.clone();
            
            // otherwise we just add the path on the end
            match &use_path.kind {
                UsePathKind::Explicit => {
                    // Explicit paths require the tail of the search path is either:
                    // - if the `sym_scope` is emtpy, the `sym_name`, or
                    // - the root of the `sym_scope`
                    let tail = search_path.last().unwrap();
                    if sym_scope.is_empty() {
                        if tail.name != *sym_name {
                            continue;
                        }    
                    } else {
                        let root = sym_path.root().unwrap();
                        if root == tail {
                            search_path.extend(&sym_scope.sub_path());
                        } else {
                            continue;
                        }
                    }
                },
                UsePathKind::Alias(alias) => {
                    // If the root name matches the alias, we should look for the symbol's path without the matching root,
                    let root = sym_path.root().unwrap();
                    if !root.params.is_empty() || root.name != *alias {
                        continue;
                    }
                    search_path.extend(&sym_scope.sub_path());
                },
                UsePathKind::Wildcard => search_path.extend(&sym_scope),
                UsePathKind::GenericOnly => (),
                UsePathKind::FileRoot => {
                    // File roots are both explicit use paths and wildcards, so first process it as explicit, and then as a wildcard using the default impl
                    let explicit_path = {
                        let mut search_path = search_path.clone();
                        let tail = search_path.last().unwrap();
                        if sym_scope.is_empty() {
                            if tail.name != *sym_name {
                                None
                            } else {
                                search_path.extend(&sym_scope.sub_path());
                                Some(search_path)
                            }
                        } else {
                            let root = sym_path.root().unwrap();
                            if root == tail {
                                search_path.extend(&sym_scope.sub_path());
                                Some(search_path)
                            } else {
                                None
                            }
                        }
                    };
                    if let Some(search_path) = explicit_path {
                        if let Some(sym) = table.get_direct_symbol(&search_path, &sym_name) {
                            if use_path.kind != UsePathKind::GenericOnly || matches!(&*sym.read(), Symbol::TypeGeneric(_) | Symbol::ValueGeneric(_)) {
                                found_syms.push(sym);
                            }
                        }
                    }
                    
                    // Then act as if it's just a wildcard
                    search_path.extend(&sym_scope);
                },
            };
            
            // Now we have a path we can actually use to find the symbol
            if let Some(sym) = table.get_direct_symbol(&search_path, &sym_name) {
                if use_path.kind != UsePathKind::GenericOnly || matches!(&*sym.read(), Symbol::TypeGeneric(_) | Symbol::ValueGeneric(_)) {
                    found_syms.push(sym);
                }
            }

            // If we hit the end of a scope, check for duplicates or return the found symbol
            // TODO: clarify in design that if a symbol is found within a scope, the outer scopes will be ignored
            if use_path.last_in_scope {
                if found_syms.len() == 1 {
                    return Ok(found_syms[0].clone());
                } else if found_syms.len() > 1 {
                    return Err(SymbolLookupError::Ambiguous {
                        path: sym_path.clone(),
                        possible_paths: found_syms.iter()
                            .map(|sym| sym.read().path().clone())
                            .collect(),
                    })
                }
            }
        }

        Err(SymbolLookupError::Unknown { path: sym_path.clone(), kind: SymbolLookupKind::Symbol })
    }

    //--------------------------------------------------------------

    pub fn add_precedence(&mut self, lib: Option<&LibraryPath>, name: String, kind: PrecedenceOrderKind, assoc: PrecedenceAssocKind) -> SymbolRef {
        let lib = lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone());
        let sym = Symbol::Precedence(PrecedenceSymbol {
            path: SymbolPath::new(
                lib.clone(),
                Scope::new(), 
                PathIden::from_name(name.clone()),
            ),
            order_kind: kind,
            assoc,
            lower_than: None,
            higher_than: None,
            id: u16::MAX,
        }); 

        let table = self.precedences.entry(lib).or_default();

        let sym = Arc::new(RwLock::new(sym));
        table.insert(name, sym.clone());
        sym
    }

    pub fn get_direct_precedence(&self, lib: &LibraryPath, name: &str) -> Option<SymbolRef> {
        let table = match self.precedences.get(lib) {
            Some(table) => table,
            None => return None,
        };
        table.get(name).cloned()
    }

    pub fn get_precedence(&self, uses: &RootUseTable, name: &str) -> Result<SymbolRef, SymbolLookupError> {
        let precedence_paths = uses.precedence_paths();

        for use_path in precedence_paths {
            if let Some(precedence) = &use_path.precedence {
                if precedence != name {
                    continue;
                }
            }

            if let Some(sym) = self.get_direct_precedence(&use_path.lib, name) {
                // Use table SHOULD have been validated at this point, so there aren't going to be any duplicate precedences that are possible
                return Ok(sym);
            }
        }

        let mut path = Scope::new();
        path.push(name.to_string());
        Err(SymbolLookupError::Unknown { path, kind: SymbolLookupKind::Precedence })
    }

    pub fn has_precedence_for_lib(&self, lib: &LibraryPath) -> bool {
        self.precedences.contains_key(lib)
    }

    pub fn get_precedences_for_lib(&self, lib: &LibraryPath) -> Option<&HashMap<String, SymbolRef>> {
        self.precedences.get(lib)
    }

    //--------------------------------------------------------------

    pub fn add_op_set(&mut self, lib: Option<&LibraryPath>, name: String) -> SymbolRef {
        let lib = lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone());
        let iden = PathIden::from_name(name.to_string());
        let sym = Symbol::OpSet(OpSetSymbol {
            path: SymbolPath::new(
                lib.clone(),
                Scope::new(),
                iden,
            ),
            assoc_trait: None,
            precedence: None,
            bases: Vec::new(),
            has_generics: false,
            has_output_alias: false,
            parent_has_output: false,
        });

        let table = self.operators.entry(lib).or_default();
        
        let sym = Arc::new(RwLock::new(sym));
        table.insert(name, (sym.clone(), HashMap::new()));
        sym
    }

    pub fn add_operator(&mut self, lib: Option<&LibraryPath>, op_set_name: &str, name: String, op_ty: OpType, op: Punctuation) -> SymbolRef {
        let lib = lib.map_or_else(|| self.cur_lib.clone(), |lib| lib.clone());
        let mut scope = Scope::new();
        scope.push(op_set_name.to_string());
        let iden = PathIden::from_name(name.to_string());
        let sym = Symbol::Operator(OperatorSymbol {
            path: SymbolPath::new(
                lib.clone(),
                scope,
                iden
            ),
            op_ty,
            op,
            assoc_method: None,
        });

        // Operators are always added after their corresponding sets
        let table = self.operators.get_mut(&lib).unwrap();
        let table = table.get_mut(op_set_name).unwrap();

        let sym = Arc::new(RwLock::new(sym));
        table.1.insert(name, sym.clone());

        sym
    }

    pub fn get_direct_op_set(&self, lib: &LibraryPath, op_set: &str) -> Option<SymbolRef> {
        let table = self.operators.get(lib)?;
        table.get(op_set).map(|(sym, _)| sym.clone())
    }

    pub fn get_direct_op_set_and_ops(&self, lib: &LibraryPath, op_set: &str) -> Option<&(SymbolRef, HashMap<String, SymbolRef>)> {
        let table = self.operators.get(lib)?;
        table.get(op_set)
    }

    pub fn get_operator_set(&self, uses: &RootUseTable, name: &str) -> Result<SymbolRef, SymbolLookupError> {
        let op_set_paths = uses.operator_set_paths();
        for op_path in op_set_paths {
            if let Some(op_set) = &op_path.op_set {
                if op_set != name {
                    continue;
                }
            }

            if let Some(sym) = self.get_direct_op_set(&op_path.lib, name) {
                // Use table SHOULD have been validated at this point, so there aren't going to be any duplicate operator sets that are possible
                return Ok(sym);
            }
        }
        let mut path = Scope::new();
        path.push(name.to_string());
        Err(SymbolLookupError::Unknown { path, kind: SymbolLookupKind::OperatorSet })
    }


    pub fn has_op_set_for_lib(&self, lib: &LibraryPath) -> bool {
        self.operators.contains_key(lib)
    }

    pub fn get_op_set_and_ops_for_lib(&self, lib: &LibraryPath) -> Option<&HashMap<String, (SymbolRef, HashMap<String, SymbolRef>)>> {
        self.operators.get(lib)
    }

    //--------------------------------------------------------------

    pub fn log(&self, puncts: &PuncutationTable) {
        let mut logger = IndentLogger::new("    ", "|   ", "+---");
        let end = self.tables.len() - 1;
        for (idx, (lib_path, table)) in self.tables.iter().enumerate() {
            let precedences = self.precedences.get(&lib_path);
            let operator_sets = self.operators.get(&lib_path);

            logger.set_last_at_indent_if(idx == end && self.ty_table.is_empty());

            logger.log_indented("Table", |logger| {
                if let Some(group) = &lib_path.group {
                    logger.prefixed_log_fmt(format_args!("Group: {group}\n"));
                }
                logger.prefixed_log_fmt(format_args!("Package: {}\n", &lib_path.package));
                logger.prefixed_log_fmt(format_args!("Library: {}\n", &lib_path.library));

                SymbolTableLogger::log_table(logger, table, precedences.is_some() || operator_sets.is_some());
                
                if let Some(precedences) = precedences {
                    let end = precedences.len() - 1;
                    for (idx, (_, precedence)) in precedences.iter().enumerate() {
                        logger.set_last_at_indent_if(idx == end && operator_sets.is_none());;
                        SymbolTableLogger::log_symbol(logger, &precedence.read(), None);
                    }
                }

                if let Some(operator_sets) = operator_sets {
                    let end = operator_sets.len() - 1;
                    for (idx, (_, (set, ops))) in operator_sets.iter().enumerate() {
                        logger.set_last_at_indent_if(idx == end);
                        SymbolTableLogger::log_operator_set(logger, &set.read(), ops, puncts);
                    }
                }
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
    fn log_table(logger: &mut IndentLogger, table: &SymbolTable, has_syms_following: bool) {
        for (idx, (name, symbols)) in table.symbols.iter().enumerate() {
            logger.set_last_at_indent_if(!has_syms_following && idx == table.symbols.len() - 1);
            for (_, sym) in symbols {
                let sub_table = table.get_direct_sub_table_from_name(name);
                Self::log_symbol(logger, &sym.read(), sub_table);
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
                logger.prefixed_log_fmt(format_args!("Order kind: {}\n", sym.order_kind));
                logger.prefixed_log_fmt(format_args!("Accociativity: {}\n", sym.assoc));
                if let Some(lower_than) = &sym.lower_than {
                    let lower_than = lower_than.upgrade().unwrap();
                    logger.prefixed_log_fmt(format_args!("Lower than: {}\n", lower_than.read().path()));
                }
                if let Some(higher_than) = &sym.higher_than {
                    let higher_than = higher_than.upgrade().unwrap();
                    logger.prefixed_log_fmt(format_args!("Higher than: {}\n", higher_than.read().path()));
                }
                logger.prefixed_log_fmt(format_args!("DAG id: {}\n", sym.id));
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
            _ => logger.prefixed_logln("<unknown>\n"),
        }

        if let Some(sub_table) = sub_table {
            Self::log_table(logger, sub_table, false);
        }

        logger.pop_indent();
    }

    fn log_operator_set(logger: &mut IndentLogger, sym: &Symbol, operators: &HashMap<String, SymbolRef>, puncts: &PuncutationTable) {
        let Symbol::OpSet(sym) = sym else { unreachable!() };
        logger.prefixed_logln("Operator Set");
        logger.push_indent();
        logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));

        match &sym.precedence {
            Some(precedence) => {
                let precedence = precedence.upgrade().unwrap();
                let precedence = precedence.read();
                logger.prefixed_log_fmt(format_args!("Precedence: {}\n", precedence.path()))
            },
            None => logger.prefixed_logln("Precedence: <none>"),
        }

        logger.log_indented_slice_named("Bases(s)", &sym.bases, |logger, base| {
            let base = base.upgrade().unwrap();
            logger.prefixed_log_fmt(format_args!("{}\n", base.read().path()));
        });

        logger.prefixed_log_fmt(format_args!("Has Generics: {}\n", sym.has_generics));
        logger.prefixed_log_fmt(format_args!("Has Output Alias: {}\n", sym.has_output_alias));

        if let Some(assoc_trait) = &sym.assoc_trait {
            let sym = assoc_trait.upgrade().unwrap();
            let sym = sym.read();
            logger.prefixed_log_fmt(format_args!("Associated Trait: {}\n", sym.path()))
        }

        logger.set_last_at_indent();
        if !operators.is_empty() {
            let ops_end = operators.len() - 1;
            logger.log_indented("Operators", |logger| for (idx, (_, op)) in operators.iter().enumerate() {
                logger.set_last_at_indent_if(idx == ops_end);
                SymbolTableLogger::log_operator(logger, &op.read(), puncts);
            });
        }

        logger.pop_indent();
    }

    fn log_operator(logger: &mut IndentLogger, sym: &Symbol, puncts: &PuncutationTable) {
        let Symbol::Operator(sym) = sym else { unreachable!() };
        logger.prefixed_logln("Operator");
        logger.push_indent();
        logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path));
        logger.prefixed_log_fmt(format_args!("Op type: {}\n", sym.op_ty));
        logger.prefixed_log_fmt(format_args!("Op: {}\n", sym.op.as_str(puncts)));
        
        if let Some(assoc_method) = &sym.assoc_method {
            let sym = assoc_method.upgrade().unwrap();
            let sym = sym.read();
            logger.prefixed_log_fmt(format_args!("Associated Method: {}\n", sym.path()))
        }

        logger.pop_indent();
    }
}

