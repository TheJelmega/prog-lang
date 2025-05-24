use std::{
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
    env,
    time,
};
use parking_lot::RwLock;

use clap::Parser as _;
use ast::{Parser, Visitor as _};
use cli::Cli;
use common::{CompilerStats, FormatSpanLoc, LibraryPath, NameTable, OperatorTable, PrecedenceDAG, RootSymbolTable, RootUseTable, Scope, SpanId, SpanRegistry, Symbol, TraitDag};
use hir::{FormatHirError, Visitor as _};
use lexer::{Lexer, PuncutationTable};
use literals::LiteralTable;
use type_system::TypeRegistry;

mod error_warning;
mod literals;

mod common;

mod cli;

mod type_system;

mod lexer;
mod ast;

mod hir;

fn main() {
    let cwd = env::current_dir().unwrap().to_str().unwrap().to_string();
    println!("cwd: {cwd}");

    let cli = Cli::parse();

    let library = match &cli.library {
        Some(library) => library.clone(),
        None => cli.package.clone(),
    };

    let library_path = LibraryPath {
        group: cli.group.clone(),
        package: cli.package.clone(),
        library,
    };

    let total_start = time::Instant::now();

    let mut files_to_process = Vec::new();
    for input_file in &cli.input_files {
        files_to_process.push((input_file.clone(), Scope::new()));
    }
    
    let symbol_table = RootSymbolTable::new(library_path.clone());
    let symbol_table = Arc::new(RwLock::new(symbol_table));

    let type_registry = Arc::new(RwLock::new(TypeRegistry::new()));

    let trait_dag = Arc::new(RwLock::new(TraitDag::new()));

    let precedences = PrecedenceDAG::new();
    let precedences = Arc::new(RwLock::new(precedences));

    let operators = OperatorTable::new();
    let operators = Arc::new(RwLock::new(operators));

    let span_registry = SpanRegistry::new();
    let span_registry = Arc::new(RwLock::new(span_registry));

    let mut asts = Vec::new();

    let literal_table = Arc::new(RwLock::new(LiteralTable::new()));
    let name_table = Arc::new(RwLock::new(NameTable::new()));
    let punct_table = Arc::new(RwLock::new(PuncutationTable::new()));

    let mut stats = CompilerStats::new();

    while let Some((input_file, base_scope)) = files_to_process.pop() {
        println!("================================================================");
        println!("File path: {cwd}/{input_file}");

        stats.add_file();

        let mut file = File::open(&input_file).unwrap();

        let mut buf = Vec::new();
        _ = file.read_to_end(&mut buf).unwrap();
    
        let file_content = unsafe { String::from_utf8_unchecked(buf) };
    
        let lex_start = time::Instant::now();
    
        let tokens = {
            let mut spans = span_registry.write();
            let mut names = name_table.write();
            let mut puncts = punct_table.write();
            let mut lits = literal_table.write();
            let mut lexer = Lexer::new(&input_file, &file_content, &mut lits, &mut names, &mut puncts, &mut spans);
            match lexer.lex() {
                Ok(()) => {},
                Err(mut err) => {
                    err.set_path(input_file.clone());
                    println!("{err}");
                    return;
                },
            };
            let (num_lexed_bytes, num_lexed_chars, num_lexed_lines) = lexer.stats();
            let tokens = lexer.tokens;
            
            if cli.timings {
                let lex_dur = time::Instant::now() - lex_start;
                println!("Lexing {input_file} took {:.2} ms, generating {} tokens", lex_dur.as_secs_f32() * 1000.0, tokens.tokens.len());
                
                stats.add_lex(
                    lex_dur,
                    num_lexed_bytes,
                    num_lexed_chars,
                    num_lexed_lines,
                    tokens.tokens.len() as u64
                );
            }
            tokens
        };
            
        if cli.print_lex_output {
            let spans = span_registry.read();
            let names = name_table.read();
            let puncts = punct_table.read();
            let lits = literal_table.read();
            tokens.log(&lits, &names, &puncts, &spans);
        }
    
        if cli.output_lex_csv {
            let name = Path::new(&input_file).strip_prefix("../").unwrap();
            let mut lex_csv_out_path = PathBuf::new();
            lex_csv_out_path.push("..");
            lex_csv_out_path.push("lex_csv");
            lex_csv_out_path.push(name);
            lex_csv_out_path.set_extension("csv");

            let parent_dir_path = lex_csv_out_path.parent().unwrap();
            if !parent_dir_path.exists() {
                fs::create_dir_all(parent_dir_path).unwrap();
            }

            let spans = span_registry.read();
            let names = name_table.read();
            let puncts = punct_table.read();
            let lits = literal_table.read();
            let mut csv_file = File::create(lex_csv_out_path).unwrap();
            _ = tokens.log_csv(&mut csv_file, &lits, &names, &puncts, &spans);
        }
        
        if cli.lex_only {
            continue;
        }

        let parse_start = time::Instant::now();

        let mut ast = {
            let mut spans = span_registry.write();

            let names = name_table.read();
            let mut parser = Parser::new(&tokens, &names, &mut spans);
            match parser.parse() {
                Ok(_) => {},
                Err(err) => {
                    let tok_meta = &tokens.metadata[err.tok_idx];
                    println!("{}({}): {err}", input_file, FormatSpanLoc{ registry: &spans, span: tok_meta.span_id });
                    return;
                },
            }
            
            parser.ast
        };
            ast.tokens = tokens;
        ast.file = input_file.clone().into();

        if cli.timings {
            let parse_dur = time::Instant::now() - parse_start;
            stats.add_parse(parse_dur, ast.nodes.len() as u64);
            println!("Parsing {input_file} took {:.2} ms, generating {} nodes", parse_dur.as_secs_f32() * 1000.0, ast.nodes.len() );
        }

        if cli.print_parse_output {
            let names = name_table.read();
            let puncts = punct_table.read();
            let lits = literal_table.read();
            ast.log(&names, &lits, &puncts);
        }

        if cli.parse_only {
            continue;
        }

        let ast_start = time::Instant::now();
    
        let mut ast_ctx = ast::passes::Context::new(
            library_path.clone(),
            symbol_table.clone(),
            base_scope.clone(),
            &ast,
            precedences.clone(),
        );
        
        let mut sub_paths = Vec::new();
        {
            let names = name_table.read();
            let lits = literal_table.read();

            do_ast_pass(&cli, &mut stats, &input_file, "Context Setup", || {
                let mut pass = ast::passes::ContextSetup::new(&mut ast_ctx);
                pass.visit(&ast);
            });
            
            do_ast_pass(&cli, &mut stats, &input_file, "Module Scoping", || {
                let mut pass = ast::passes::ModuleScopePass::new(&mut ast_ctx, base_scope.clone(), &names);
                pass.visit(&ast);
            });
            
            do_ast_pass(&cli, &mut stats, &input_file, "Module Attribute Resolve", || {
                let mut pass = ast::passes::ModuleAttributeResolver::new(&mut ast_ctx, &names, &lits);
                pass.visit(&ast);
            });
            
            do_ast_pass(&cli, &mut stats, &input_file, "Module Symbol Generation + Path Collection", || {
                let mut input_path = PathBuf::from(input_file.clone());
                input_path.pop();
                let mut pass = ast::passes::ModulePathResolution::new(&mut ast_ctx, &names, input_path);
                pass.visit(&ast);
                
                sub_paths = pass.collected_paths;
            });
        }

        for err in &*ast_ctx.errors.lock() {
            println!("{err}");
        }

        if cli.timings {
            let parse_dur = time::Instant::now() - ast_start;
            println!("Processing all AST passes for {input_file} took {:.2} ms", parse_dur.as_secs_f32() * 1000.0);
        }

        for (path, scope) in sub_paths {
            println!("Found sub-module at '{}'", path.to_str().unwrap());
            files_to_process.push((path.to_str().unwrap().to_string(), scope));
        }

        asts.push((ast, ast_ctx));
    }


    println!("================================================================");
    println!("Post-parse AST passes:");

    // Precedence

    // Operators

    // let mut imported_operators = Vec::new();
    // do_ast_for_all_passes(&cli, &mut stats, "Operator Import", &mut asts, |ast, ast_ctx| {
    //     let mut pass = ast::passes::OperatorImport::new(ast_ctx, &name_table);
    //     pass.visit(ast);

    //     if !pass.imports.is_empty() {
    //         imported_operators = pass.imports;
    //     }
    // });

    // TODO: External operator importing happens here

    let use_table = Arc::new(RwLock::new(RootUseTable::new()));


    let mut hir = hir::Hir::new();
    do_ast_for_all_passes(&cli, &mut stats, "AST to HIR lowering", &mut asts, |ast, ast_ctx| {
        let spans = span_registry.read();
        let mut names = name_table.write();
        let lits = literal_table.read();
        let mut uses = use_table.write();
        let mut pass = ast::passes::AstToHirLowering::new(ast_ctx, &mut names, &lits, &spans, &mut hir, &mut uses, library_path.clone());
        pass.visit(ast);
    });
    stats.add_ast_hir_lower(&hir);

    println!("================================================================");

    // TODO: implicit prelude

    if cli.print_hir_nodes {
        let names = name_table.read();
        let puncts = punct_table.read();
        let lits = literal_table.read();

        println!("Lowered HIR:");
        let mut hir_logger = hir::NodeLogger::new(&names, &lits, &puncts);
        hir_logger.visit(&mut hir, hir::VisitFlags::all());
        println!("--------------------------------")
    }

    if cli.print_hir_code {
        let names = name_table.read();
        let puncts = punct_table.read();
        let lits = literal_table.read();

        println!("Lowered HIR pseudo-code:");
        let mut hir_printer = hir::CodePrinter::new(&names, &lits, &puncts);
        hir_printer.visit(&mut hir, hir::VisitFlags::all());
        println!("--------------------------------")
    }

    if cli.print_hir_use_table {
        println!("HIR use table");
        use_table.read().log();
        println!("--------------------------------")
    }

    let use_ambiguities = use_table.write().check_non_wildcard_ambiguity();
    if !use_ambiguities.is_empty() {
        println!("Use table ambiguities:");
        for (scope, name) in use_ambiguities {
            println!("- {scope}: {name}");
        }
    }

    {
        let mut ctx = hir::passes::PassContext {
            names: name_table.clone(),
            puncts: punct_table.clone(),
            lits: literal_table.clone(),
            syms: symbol_table.clone(),
            type_reg: type_registry.clone(),
            trait_dag: trait_dag.clone(),
            uses: use_table.clone(),
            precedence_dag: precedences.clone(),
            op_table: operators.clone(),
            lib_path: library_path.clone(),
            errors: Arc::new(RwLock::new(Vec::new())),
        };
        process_hir(&mut hir, &cli, &mut stats, &mut ctx);

        stats.num_types_registered = type_registry.read().type_count();

        if cli.print_hir_code {
            let names = name_table.read();
            let puncts = punct_table.read();
            let lits = literal_table.read();

            println!("--------------------------------");
            println!("Processed HIR pseudo-code:");
            let mut hir_printer = hir::CodePrinter::new(&names, &lits, &puncts);
            hir_printer.visit(&mut hir, hir::VisitFlags::all());
        }

        {
            let spans = span_registry.read();
            
            for err in &*ctx.errors.read() {
                println!("{}", FormatHirError::new(&spans, err.clone()));
            }
        }
    }
    
    println!("================================================================");
    if cli.print_sym_table {
        println!("Symbol table:");
        symbol_table.read().log();
        println!("--------------------------------");
    }

    if cli.print_precedence {
        println!("Precedence DAG Unordered:");
        precedences.read().log_unordered();
        println!("--------------------------------");
    }

    if cli.print_op_table {
        let puncts = punct_table.read();

        println!("Operator table");
        operators.read().log(&puncts);
    }

    if cli.print_trait_dag {
        println!("Trait DAG Unordered:");
        trait_dag.read().log_unordered();
        println!("--------------------------------");
    }

    if cli.print_type_registry {
        type_registry.read().log();
        println!("--------------------------------");
    }

    if cli.timings {
        let total_dur = time::Instant::now() - total_start;

        println!("================================================================");
        stats.log();

        let mut total_time = total_dur.as_secs_f32();
        let hours = (total_time / 3600.0).floor();
        total_time -= hours * 3600.0;
        let minutes = (total_time / 60.0).floor();
        total_time -= minutes * 60.0;
        println!("Compiler took {hours}:{minutes}:{total_time:.3}");
    }
}

fn do_ast_pass<F>(cli: &Cli, stats: &mut CompilerStats, input_file: &str, pass_name: &str, f: F) where 
    F: FnOnce()
{
    let start = time::Instant::now();
    f();
    if cli.pass_timings {
        let pass_dur = time::Instant::now() - start;
        stats.add_ast_pass(pass_dur);
        println!("Processing AST Pass '{pass_name:32}' for '{input_file}' took {:.2} ms", pass_dur.as_secs_f32() * 1000.0);
    }
}

fn do_ast_for_all_passes<F>(cli: &Cli, stats: &mut CompilerStats, pass_name: &str, asts: &mut Vec<(ast::Ast, ast::passes::Context)>, mut f: F) where
    F: FnMut(&ast::Ast, &mut ast::passes::Context)
{
    for (ast, ctx) in asts {
        let start = time::Instant::now();

        f(ast, ctx);
        
        if cli.pass_timings {
            let pass_dur = time::Instant::now() - start;
            stats.add_ast_pass(pass_dur);
            let input_file = ast.file.to_str().unwrap();
            println!("Processing AST Pass '{pass_name:32}' for '{input_file}' took {:.2} ms", pass_dur.as_secs_f32() * 1000.0);
        }
    }
}

fn process_hir(hir: &mut hir::Hir, cli: &Cli, stats: &mut CompilerStats, ctx: &hir::passes::PassContext) -> bool {
    //do_hir_pass(hir, cli, stats, hir::passes::);

    use hir::passes::*;
    
    // base passes
    do_hir_pass(hir, cli, stats, SimplePathGen::new(ctx));
    do_hir_pass(hir, cli, stats, SymbolGeneration::new(ctx));
    do_hir_pass(hir, cli, stats, TraitDagGen::new(ctx));

    {
        let mut trait_dag = ctx.trait_dag.write();

        let cycles = trait_dag.check_cycles();
        if !cycles.is_empty() {
            for cycle in cycles {
                let mut cycle_str = String::new();

                for idx in &cycle {
                    let sym = trait_dag.get(*idx).unwrap().symbol.read();
                    let Symbol::Trait(sym) = &*sym else { unreachable!() };
                    cycle_str.push_str(&format!("{} -> ", sym.path));
                }
                let sym = trait_dag.get(cycle[0]).unwrap().symbol.read();
                let Symbol::Trait(sym) = &*sym else { unreachable!() };
                cycle_str.push_str(&format!("{}", sym.path));

                ctx.add_error(hir::HirError {
                    span: SpanId::INVALID,
                    err: error_warning::HirErrorCode::CycleInTraitDag { cycle: cycle_str },
                })
            }
            
            return false;
        }
        trait_dag.calculate_predecessors();
    }

    // Precedences
    do_hir_pass(hir, cli, stats, PrecedenceAttrib::new(ctx));
    do_hir_pass(hir, cli, stats, PrecedenceCollection::new(ctx));
    do_hir_pass(hir, cli, stats, PrecedenceConnect::new(ctx));

    {
        let mut precedence_dag = ctx.precedence_dag.write();

        let cycles = precedence_dag.check_cycles();
        if !cycles.is_empty() {
            for cycle in cycles {
                let mut cycle_str = String::new();

                for idx in &cycle {
                    let name = precedence_dag.get_name(*idx).unwrap();
                    cycle_str.push_str(&format!("{name} -> "));
                }
                let name = precedence_dag.get_name(cycle[0]).unwrap();
                cycle_str.push_str(&format!("{name}"));

                ctx.add_error(hir::HirError {
                    span: SpanId::INVALID,
                    err: error_warning::HirErrorCode::CycleInPrecedenceDag { cycle: cycle_str },
                })
            }
            
            return false;
        }
        precedence_dag.calculate_order();
    }
    
    // Operators
    do_hir_pass(hir, cli, stats, OpPrecedenceProcessing::new(ctx));
    do_hir_pass(hir, cli, stats, OperatorCollection::new(ctx));
    do_hir_pass(hir, cli, stats, InfixReorder::new(ctx));
    do_hir_pass(hir, cli, stats, OpTagging::new(ctx));
    do_hir_pass(hir, cli, stats, OpTraitGen::new(ctx));

    // Trait
    do_hir_pass(hir, cli, stats, TraitItemProcess::new(ctx));
    
    // Impl trait processing
    do_hir_pass(hir, cli, stats, ImplTraitPathGen::new(ctx));
    do_hir_pass(hir, cli, stats, ImplTraitSymResolve::new(ctx));
    do_hir_pass(hir, cli, stats, ImplTraitItemCollection::new(ctx));
    do_hir_pass(hir, cli, stats, TraitImpl::new(ctx));
    
    // Misc
    do_hir_pass(hir, cli, stats, VisibilityProcess::new(ctx.lib_path.clone()));
    do_hir_pass(hir, cli, stats, SelfTyReplacePass::new(ctx));
    
    // Types
    do_hir_pass(hir, cli, stats, ItemLevelTypeGen::new(ctx));
    do_hir_pass(hir, cli, stats, TypeImplSymbolAssoc::new(ctx));
    
    !ctx.errors.read().is_empty()
}

fn do_hir_pass<T: hir::Pass>(hir: &mut hir::Hir, cli: &Cli, stats: &mut CompilerStats, mut pass: T) {
    let start = time::Instant::now();
    pass.process(hir);
    if cli.pass_timings {
        let pass_dur = time::Instant::now() - start;
        stats.add_hir_pass(pass_dur);
        println!("HIR pass '{:40}' took {:.2} ms", T::NAME, pass_dur.as_secs_f32() * 1000.0);
    }
}