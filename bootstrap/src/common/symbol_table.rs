use std::{
    collections::HashMap,
    fmt::{self, Write},
    path::PathBuf,
    sync::Arc
};
use parking_lot::{MappedRwLockReadGuard, MappedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard};

use super::{IndentLogger, Scope, ScopeSegment};

pub struct SymbolPathIden {
    name:   String,
    params: Vec<String>,
}

pub struct SymbolPath {
    idens: Vec<SymbolPathIden>
}

impl SymbolPath {
    
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
}

pub struct ModuleSymbol {
    pub scope:     Scope,
    pub name:      String,
    pub path:      PathBuf,
    pub sub_table: SymbolTable
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

    pub sub_table: SymbolTable,
}

pub struct ImplSymbol {
    pub scope:     Scope,
    pub name:      String,

    pub sub_table: SymbolTable,
}

pub type SymbolRef = Arc<RwLock<Symbol>>;

pub struct SymbolTable {
    nodes: HashMap<String, Vec<(Vec<String>, SymbolRef)>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn add_module(&mut self, scope: &Scope, name: String, file_path: PathBuf) -> SymbolRef {
        let sym = Symbol::Module(ModuleSymbol{
            scope: scope.clone(),
            name: name.clone(),
            path: file_path,
            sub_table: SymbolTable::new(),
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

            sub_table: SymbolTable::new(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    pub fn add_impl(&mut self, scope: &Scope, name: String) -> SymbolRef {
        let sym = Symbol::Impl(ImplSymbol {
            scope: scope.clone(),
            name: name.clone(),

            sub_table: SymbolTable::new(),
        });
        self.add_symbol(scope, &name, &[], sym)
    }

    fn add_symbol(&mut self, scope: &Scope, name: &str, params: &[String], sym: Symbol) -> SymbolRef {
        let sym = Arc::new(RwLock::new(sym));
        self.add_symbol_(scope, name, params, sym.clone());
        sym
    }

    fn add_symbol_(&mut self, scope: &Scope, name: &str, params: &[String], sym: SymbolRef) {
        if !scope.is_empty() {
            let mut sub_table = self.find_subtable_mut(&scope.root().unwrap()).unwrap();
            sub_table.add_symbol_(&scope.sub_path(), name, params, sym);
        } else {
            let params = Vec::from(params);
            let sub_syms = if let Some(syms) = self.nodes.get_mut(name) {
                syms
            } else {
                self.nodes.insert(name.to_string(), Vec::new());
                self.nodes.get_mut(name).unwrap()
            };
            sub_syms.push((Vec::from(params), sym));
        }
    }

    pub fn get_symbol(&self, scope: &Scope, name: &str) -> Option<SymbolRef> {
        if !scope.is_empty() {
            let sub_table = self.find_subtable(scope.root().unwrap())?;
            sub_table.get_symbol(&scope.sub_path(), name)
        } else {
            let sub_syms = self.nodes.get(name)?;
            if sub_syms.len() == 1 {
                Some(sub_syms[0].1.clone())
            } else {
                let mut tmp = None;
                for (params, sym) in sub_syms {
                    if params.is_empty() {
                        tmp = Some(sym.clone());
                        break;
                    }
                }
                tmp
            }
        }
    }

    fn find_subtable(&self, segment: &ScopeSegment) -> Option<MappedRwLockReadGuard<SymbolTable>> {
        let sub_syms = self.nodes.get(&segment.name)?;
        let symbol = if sub_syms.len() == 1 {
            &sub_syms[0].1
        } else {
            let mut tmp = None;
            for (params, sym) in sub_syms {
                if &segment.params == params {
                    tmp = Some(sym);
                    break;
                }
            }
            tmp?
        };

        RwLockReadGuard::try_map(symbol.read(), |sym| match sym {
            Symbol::Module(sym) => Some(&sym.sub_table),
            Symbol::Trait(sym)  => Some(&sym.sub_table),
            Symbol::Impl(sym)   => Some(&sym.sub_table),
            _ => None,
        }).ok()
    }

    fn find_subtable_mut(&mut self, segment: &ScopeSegment) -> Option<MappedRwLockWriteGuard<SymbolTable>> {
        let sub_syms = self.nodes.get_mut(&segment.name)?;
        let symbol = if sub_syms.len() == 1 {
            &mut sub_syms[0].1
        } else {
            let mut tmp = None;
            for (params, sym) in sub_syms {
                if &segment.params == params {
                    tmp = Some(sym);
                    break;
                }
            }
            tmp?
        };

        RwLockWriteGuard::try_map(symbol.write(), |sym| match sym {
            Symbol::Module(sym) => Some(&mut sym.sub_table),
            Symbol::Trait(sym)  => Some(&mut sym.sub_table),
            Symbol::Impl(sym)   => Some(&mut sym.sub_table),
            _ => None,
        }).ok()
    }




    pub fn log(&self) {
        let mut logger = IndentLogger::new("    ", "|   ", "+---");
        SymbolTableLogger::log_table(&mut logger, self);
    }
}

struct SymbolTableLogger {
    logger: IndentLogger
}

#[allow(unused)]
impl SymbolTableLogger {
    fn log_table(logger: &mut IndentLogger, table: &SymbolTable) {
        for (idx, (_, symbols)) in table.nodes.iter().enumerate() {
            if idx == table.nodes.len() - 1 {
                logger.set_last_at_indent();
            }

            if symbols.len() == 1 && symbols[0].0.is_empty() {
                Self::log_symbol(logger, &symbols[0].1.read());
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

                    logger.log_indented(&params_s, |logger| Self::log_symbol(logger, &sym.read()))
                }
            }

        }
    }

    fn log_symbol(logger: &mut IndentLogger, sym: &Symbol) {
        match sym {
            Symbol::Module(sym) => logger.log_indented("Module", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Path: {}\n", sym.path.to_str().unwrap()));
                logger.set_last_at_indent();
                logger.log_indented("Sub Table", |logger| {
                    Self::log_table(logger, &sym.sub_table);
                });
            }),
            Symbol::Precedence(sym) => logger.log_indented("Precedence", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Id: {}\n", sym.id));
            }),
            Symbol::Function(sym) => logger.log_indented("Function", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));


                logger.set_last_at_indent();
                logger.log_indented("Sub Table", |logger| {
                    Self::log_table(logger, &sym.sub_table);
                });
            }),
            Symbol::TypeAlias(sym) => logger.log_indented("Type Alias", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::DistinctType(sym) => logger.log_indented("Distinct Type", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::OpaqueType(sym) => logger.log_indented("Opaque Type", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::Struct(sym) => logger.log_indented("Struct", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Kind: {}\n", sym.kind));
            }), 
            Symbol::Union(sym) => logger.log_indented("Union", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::AdtEnum(sym) => logger.log_indented("ADT enum", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::FlagEnum(sym) => logger.log_indented("Flag", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::Bitfield(sym) => logger.log_indented("Bitfield", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::Const(sym) => logger.log_indented("Const", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::Static(sym) => logger.log_indented("Static", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
                logger.prefixed_log_fmt(format_args!("Kind: {}\n", sym.kind));
            }),
            Symbol::Property(sym) => logger.log_indented("Property", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));
            }),
            Symbol::Trait(sym) => logger.log_indented("Trait", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));


                logger.set_last_at_indent();
                logger.log_indented("Sub Table", |logger| {
                    Self::log_table(logger, &sym.sub_table);
                });
            }),
            Symbol::Impl(sym) => logger.log_indented("Impl", |logger| {
                logger.prefixed_log_fmt(format_args!("Name: {}\n", sym.name));


                logger.set_last_at_indent();
                logger.log_indented("Sub Table", |logger| {
                    Self::log_table(logger, &sym.sub_table);
                });
            }),
            
        }
    }
}