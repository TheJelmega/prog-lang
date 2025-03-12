
use std::{fmt, time, hash::Hash};

mod logger;
pub use logger::*;

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




pub struct CompilerStats {
    // Lexer
    pub file_count:                           u64,
    pub lex_time:                             time::Duration,
    pub bytes_parsed:                         u64,
    pub chars_parsed:                         u64,
    pub lines_parsed:                         u64,
    pub tokens_generated:                     u64,

    // Parser
    pub parse_time:                           time::Duration,
    pub ast_nodes_generated:                  u64,

    // Ast
    pub ast_pass_time:                        time::Duration,

    // AST -> HIR lower
    pub ast_functions_lowered:                u64,
    pub ast_extern_functions_no_body_lowered: u64,
    pub ast_type_aliases_lowered:             u64,
    pub ast_distinct_types_lowered:           u64,
    pub ast_opaque_types_lowered:             u64,
    pub ast_structs_lowered:                  u64,
    pub ast_tuple_structs_lowered:            u64,
    pub ast_unit_structs_lowered:             u64,
    pub ast_unions_lowered:                   u64,
    pub ast_adt_enums_lowered:                u64,
    pub ast_flag_enums_lowered:               u64,
    pub ast_bitfields_lowered:                u64,
    pub ast_consts_lowered:                   u64,
    pub ast_statics_lowered:                  u64,
    pub ast_tls_statics_lowered:              u64,
    pub ast_extern_statics_lowered:           u64,
    pub ast_triats_lowered:                   u64,
    pub ast_trait_type_aliases_lowered:       u64,
    pub ast_trait_consts_lowered:             u64,
    pub ast_trait_functions_lowered:          u64,
    pub ast_trait_properties_lowered:         u64,
    pub ast_impls_lowered:                    u64,
    pub ast_impl_functions_lowered:           u64,
    pub ast_impl_methods_lowered:             u64,
    pub ast_impl_type_aliases_lowered:        u64,
    pub ast_impl_consts_lowered:              u64,
    pub ast_impl_statics_lowered:             u64,
    pub ast_impl_tls_statics_lowered:         u64,
    pub ast_impl_properties_lowered:          u64,
    pub ast_op_trait_lowered:                 u64,
    pub ast_op_functions_lowered:             u64,
    pub ast_op_specializations_lowered:       u64,
    pub ast_op_contracts_lowered:             u64,

    // HIR
    pub num_hir_passes:                       u64,
    pub hir_pass_time:                        time::Duration,
    
}

impl CompilerStats {
    pub fn new() -> Self {
        Self {
            file_count:                           0,
            lex_time:                             time::Duration::default(),
            bytes_parsed:                         0,
            chars_parsed:                         0,
            lines_parsed:                         0,
            tokens_generated:                     0,

            parse_time:                           time::Duration::default(),
            ast_nodes_generated:                  0,

            ast_pass_time:                        time::Duration::default(),

            ast_functions_lowered:                0,
            ast_extern_functions_no_body_lowered: 0,
            ast_type_aliases_lowered:             0,
            ast_distinct_types_lowered:           0,
            ast_opaque_types_lowered:             0,
            ast_structs_lowered:                  0,
            ast_tuple_structs_lowered:            0,
            ast_unit_structs_lowered:             0,
            ast_unions_lowered:                   0,
            ast_adt_enums_lowered:                0,
            ast_flag_enums_lowered:               0,
            ast_bitfields_lowered:                0,
            ast_consts_lowered:                   0,
            ast_statics_lowered:                  0,
            ast_tls_statics_lowered:              0,
            ast_extern_statics_lowered:           0,
            ast_triats_lowered:                   0,
            ast_trait_type_aliases_lowered:       0,
            ast_trait_consts_lowered:             0,
            ast_trait_functions_lowered:          0,
            ast_trait_properties_lowered:         0,
            ast_impls_lowered:                    0,
            ast_impl_functions_lowered:           0,
            ast_impl_methods_lowered:             0,
            ast_impl_type_aliases_lowered:        0,
            ast_impl_consts_lowered:              0,
            ast_impl_statics_lowered:             0,
            ast_impl_tls_statics_lowered:         0,
            ast_impl_properties_lowered:          0,
            ast_op_trait_lowered:                 0,
            ast_op_functions_lowered:             0,
            ast_op_specializations_lowered:       0,
            ast_op_contracts_lowered:             0,

            num_hir_passes:                       0,
            hir_pass_time:                        time::Duration::default(),
        }
    }
    
    pub fn add_file(&mut self) {
        self.file_count += 1;
    }

    pub fn add_lex(&mut self, time: time::Duration, num_bytes: u64, num_chars: u64, num_lines: u64, num_tokens: u64) {
        self.lex_time += time;
        self.bytes_parsed += num_bytes;
        self.chars_parsed += num_chars;
        self.lines_parsed += num_lines;
        self.tokens_generated += num_tokens;
    }

    pub fn add_parse(&mut self, time: time::Duration, num_nodes: u64) {
        self.parse_time += time;
        self.ast_nodes_generated += num_nodes;
    }    

    pub fn add_ast_pass(&mut self, time: time::Duration) {
        self.ast_pass_time += time;
    }

    pub fn add_ast_hir_lower(&mut self, hir: &Hir) {
        self.ast_functions_lowered = hir.functions.len() as u64;
        self.ast_extern_functions_no_body_lowered = hir.extern_functions_no_body.len() as u64;
        self.ast_type_aliases_lowered = hir.type_aliases.len() as u64;
        self.ast_distinct_types_lowered = hir.distinct_types.len() as u64;
        self.ast_opaque_types_lowered = hir.opaque_types.len() as u64;
        self.ast_structs_lowered = hir.structs.len() as u64;
        self.ast_tuple_structs_lowered = hir.tuple_structs.len() as u64;
        self.ast_unit_structs_lowered = hir.unit_structs.len() as u64;
        self.ast_unions_lowered = hir.unions.len() as u64;
        self.ast_adt_enums_lowered = hir.adt_enums.len() as u64;
        self.ast_flag_enums_lowered = hir.flag_enums.len() as u64;
        self.ast_bitfields_lowered = hir.bitfields.len() as u64;
        self.ast_consts_lowered = hir.consts.len() as u64;
        self.ast_statics_lowered = hir.statics.len() as u64;
        self.ast_tls_statics_lowered = hir.tls_statics.len() as u64;
        self.ast_extern_statics_lowered = hir.extern_statics.len() as u64;
        self.ast_triats_lowered = hir.traits.len() as u64;
        self.ast_trait_type_aliases_lowered = hir.trait_type_alias.len() as u64;
        self.ast_trait_consts_lowered = hir.trait_consts.len() as u64;
        self.ast_trait_functions_lowered = hir.trait_functions.len() as u64;
        self.ast_trait_properties_lowered = hir.trait_properties.len() as u64;
        self.ast_impls_lowered = hir.impls.len() as u64;
        self.ast_impl_functions_lowered = hir.impl_functions.len() as u64;
        self.ast_impl_methods_lowered = hir.methods.len() as u64;
        self.ast_impl_type_aliases_lowered = hir.impl_type_aliases.len() as u64;
        self.ast_impl_consts_lowered = hir.impl_consts.len() as u64;
        self.ast_impl_statics_lowered = hir.impl_statics.len() as u64;
        self.ast_impl_tls_statics_lowered = hir.impl_tls_statics.len() as u64;
        self.ast_impl_properties_lowered = hir.properties.len() as u64;
        self.ast_op_trait_lowered = hir.op_traits.len() as u64;
        self.ast_op_functions_lowered = hir.op_functions.len() as u64;
        self.ast_op_specializations_lowered = hir.op_specializations.len() as u64;
        self.ast_op_contracts_lowered = hir.op_contracts.len() as u64;
    }

    pub fn add_hir_pass(&mut self, time: time::Duration) {
        self.hir_pass_time += time;
        self.num_hir_passes += 1;
    }


    pub fn log(&self) {
        let logger = Logger::new();
        logger.log_fmt(format_args!("Files processed: {}\n", self.file_count));

        logger.logln("- Lexer:");
        logger.log_fmt(format_args!("    Time: {:.2}ms\n", self.lex_time.as_secs_f32() * 1000.0));
        logger.log_fmt(format_args!("    Bytes processed:  {} bytes ", self.bytes_parsed));

        const KIB: u64 = 1024;
        const MIB: u64 = 1024 * KIB;
        const GIB: u64 = 1024 * MIB;
        if self.bytes_parsed > GIB {
            logger.log_fmt(format_args!("({:.2}GiB)\n", self.bytes_parsed as f32 / GIB as f32))
        } else if self.bytes_parsed > MIB {
            logger.log_fmt(format_args!("({:.2}MiB)\n", self.bytes_parsed as f32 / MIB as f32))
        } else if self.bytes_parsed > KIB {
            logger.log_fmt(format_args!("({:.1}KiB)\n", self.bytes_parsed as f32 / KIB as f32))
        } else {
            logger.logln("");
        }

        logger.log_fmt(format_args!("    Chars processed:  {}\n", self.chars_parsed));
        logger.log_fmt(format_args!("    Lines processed:  {}\n", self.lines_parsed));
        logger.log_fmt(format_args!("    Tokens generated: {}\n", self.tokens_generated));

        logger.logln("- Parser:");
        logger.log_fmt(format_args!("    Time: {:.2}ms\n", self.parse_time.as_secs_f32() * 1000.0));
        logger.log_fmt(format_args!("    AST nodes generated: {}\n", self.ast_nodes_generated));

        logger.logln("- AST passes:");
        logger.log_fmt(format_args!("    Time: {:.2}ms\n", self.ast_pass_time.as_secs_f32() * 1000.0));

        logger.logln("- AST to HIR lowering");
        logger.log_fmt(format_args!("    Functions lowered:                  {}\n", self.ast_functions_lowered));
        logger.log_fmt(format_args!("    Extern functions (no body) lowered: {}\n", self.ast_extern_functions_no_body_lowered));
        logger.log_fmt(format_args!("    Type aliases lowered:               {}\n", self.ast_type_aliases_lowered));
        logger.log_fmt(format_args!("    Distinct types lowered:             {}\n", self.ast_distinct_types_lowered));
        logger.log_fmt(format_args!("    Opaque types lowered:               {}\n", self.ast_opaque_types_lowered));
        logger.log_fmt(format_args!("    Structs lowered:                    {}\n", self.ast_structs_lowered));
        logger.log_fmt(format_args!("    Tuple structs lowered:              {}\n", self.ast_tuple_structs_lowered));
        logger.log_fmt(format_args!("    Unit structs lowered:               {}\n", self.ast_unit_structs_lowered));
        logger.log_fmt(format_args!("    Unions lowered:                     {}\n", self.ast_unions_lowered));
        logger.log_fmt(format_args!("    ADT enums lowered:                  {}\n", self.ast_adt_enums_lowered));
        logger.log_fmt(format_args!("    Flag enums lowered:                 {}\n", self.ast_flag_enums_lowered));
        logger.log_fmt(format_args!("    Bitfields lowered:                  {}\n", self.ast_bitfields_lowered));
        logger.log_fmt(format_args!("    Consts lowered:                     {}\n", self.ast_consts_lowered));
        logger.log_fmt(format_args!("    Statics lowered:                    {}\n", self.ast_statics_lowered));
        logger.log_fmt(format_args!("    TLS statics lowered:                {}\n", self.ast_tls_statics_lowered));
        logger.log_fmt(format_args!("    Extern statics lowered:             {}\n", self.ast_extern_statics_lowered));

        logger.log_fmt(format_args!("    Traits lowered:                     {}\n", self.ast_triats_lowered));
        logger.log_fmt(format_args!("    Trait type aliases lowered:         {}\n", self.ast_trait_type_aliases_lowered));
        logger.log_fmt(format_args!("    Trait consts lowered:               {}\n", self.ast_trait_consts_lowered));
        logger.log_fmt(format_args!("    Trait functions lowered:            {}\n", self.ast_trait_functions_lowered));
        logger.log_fmt(format_args!("    Trait properties lowered:           {}\n", self.ast_trait_properties_lowered));

        logger.log_fmt(format_args!("    Impls lowered:                      {}\n", self.ast_impls_lowered));
        logger.log_fmt(format_args!("    Impl functions lowered:             {}\n", self.ast_impl_functions_lowered));
        logger.log_fmt(format_args!("    Impl methods lowered:               {}\n", self.ast_impl_methods_lowered));
        logger.log_fmt(format_args!("    Impl type aliases lowered:          {}\n", self.ast_impl_type_aliases_lowered));
        logger.log_fmt(format_args!("    Impl consts lowered:                {}\n", self.ast_impl_consts_lowered));
        logger.log_fmt(format_args!("    Impl statics lowered:               {}\n", self.ast_impl_statics_lowered));
        logger.log_fmt(format_args!("    Impl TLS statics lowered:           {}\n", self.ast_impl_tls_statics_lowered));
        logger.log_fmt(format_args!("    Impl properties lowered:            {}\n", self.ast_impl_properties_lowered));

        logger.log_fmt(format_args!("    Op traits lowered:                  {}\n", self.ast_op_trait_lowered));
        logger.log_fmt(format_args!("    Op functions lowered:               {}\n", self.ast_op_functions_lowered));
        logger.log_fmt(format_args!("    Op specialization lowered:          {}\n", self.ast_op_specializations_lowered));
        logger.log_fmt(format_args!("    Op contract lowered:                {}\n", self.ast_op_contracts_lowered));
        
        logger.logln("- HIR passes:");
        logger.log_fmt(format_args!("    Num passes: {}\n", self.num_hir_passes));
        logger.log_fmt(format_args!("    Time:       {:.2}ms\n", self.hir_pass_time.as_secs_f32() * 1000.0));
    }
}