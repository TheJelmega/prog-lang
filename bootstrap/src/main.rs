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
use common::{CompilerStats, FormatSpanLoc, LibraryPath, NameTable, OperatorTable, PrecedenceDAG, SymbolTable, Scope, SpanRegistry, UseTable};
use hir::Visitor as _;
use lexer::{Lexer, PuncutationTable};
use literals::LiteralTable;

mod error_warning;
mod literals;

mod common;

mod cli;

mod type_system;

mod lexer;
mod ast;
mod ast_passes;

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
    
    let symbol_table = SymbolTable::new();
    let symbol_table = Arc::new(RwLock::new(symbol_table));

    let precedences = PrecedenceDAG::new();
    let precedences = Arc::new(RwLock::new(precedences));

    let operators = OperatorTable::new();
    let operators = Arc::new(RwLock::new(operators));

    let span_registry = SpanRegistry::new();
    let span_registry = Arc::new(RwLock::new(span_registry));

    let mut asts = Vec::new();

    let mut literal_table = LiteralTable::new();
    let mut name_table = NameTable::new();
    let mut punct_table = PuncutationTable::new();

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
            let mut lexer = Lexer::new(&input_file, &file_content, &mut literal_table, &mut name_table, &mut punct_table, &mut spans);
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
            tokens.log(&literal_table, &name_table, &punct_table, &spans);
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
            let mut csv_file = File::create(lex_csv_out_path).unwrap();
            _ = tokens.log_csv(&mut csv_file, &literal_table, &name_table, &punct_table, &spans);
        }
        
        if cli.lex_only {
            continue;
        }

        let parse_start = time::Instant::now();

        let mut parser = Parser::new(&tokens, &name_table);
        match parser.parse() {
            Ok(_) => {},
            Err(err) => {
                let tok_meta = &tokens.metadata[err.tok_idx];
                let spans = span_registry.read();
                println!("{}({}): {err}", input_file, FormatSpanLoc{ registry: &spans, span: tok_meta.span_id });
                return;
            },
        }

        let mut ast = parser.ast;
        ast.tokens = tokens;
        ast.file = input_file.clone().into();

        if cli.timings {
            let parse_dur = time::Instant::now() - parse_start;
            stats.add_parse(parse_dur, ast.nodes.len() as u64);
            println!("Parsing {input_file} took {:.2} ms, generating {} nodes", parse_dur.as_secs_f32() * 1000.0, ast.nodes.len() );
        }

        if cli.print_parse_output {
            ast.log(&name_table, &literal_table, &punct_table);
        }

        if cli.parse_only {
            continue;
        }

        let ast_start = time::Instant::now();
    
        let mut ast_ctx = ast_passes::Context::new(
            library_path.clone(),
            symbol_table.clone(),
            base_scope.clone(),
            &ast,
            precedences.clone(),
            operators.clone(),
        );

        do_ast_pass(&cli, &mut stats, &input_file, "Context Setup", || {
            let mut pass = ast_passes::ContextSetup::new(&mut ast_ctx);
            pass.visit(&ast);
        });

        do_ast_pass(&cli, &mut stats, &input_file, "Module Scoping", || {
            let mut pass = ast_passes::ModuleScopePass::new(&mut ast_ctx, base_scope.clone(), &name_table);
            pass.visit(&ast);
        });
        
        do_ast_pass(&cli, &mut stats, &input_file, "Module Attribute Resolve", || {
            let mut pass = ast_passes::ModuleAttributeResolver::new(&mut ast_ctx, &name_table, &literal_table);
            pass.visit(&ast);
        });

        let mut sub_paths = Vec::new();
        do_ast_pass(&cli, &mut stats, &input_file, "Module Symbol Generation + Path Collection", || {
            let mut input_path = PathBuf::from(input_file.clone());
            input_path.pop();
            let mut pass = ast_passes::ModulePathResolution::new(&mut ast_ctx, &name_table, input_path);
            pass.visit(&ast);

            sub_paths = pass.collected_paths;
        });

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

    do_ast_for_all_passes(&cli, &mut stats, "Precedence Collection", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceCollection::new(ast_ctx, &name_table);
        pass.visit(ast);
    });

    let mut imported_precedences = Vec::new();
    do_ast_for_all_passes(&cli, &mut stats, "Precedence Import", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceImportCollection::new(ast_ctx, &name_table);
        pass.visit(ast);

        if !pass.imports.is_empty() {
            imported_precedences = pass.imports;
        }
    });

    // TODO: External precedences importing happens here

    do_ast_for_all_passes(&cli, &mut stats, "Precedence Connection", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceConnection::new(ast_ctx, &name_table);
        pass.visit(ast);
    });

    do_ast_for_all_passes(&cli, &mut stats, "Precedence Attribute", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceAttribute::new(ast_ctx, &name_table, &literal_table);
        pass.visit(ast);
    });

    precedences.write().precompute_order();

    // Operators

    // let mut imported_operators = Vec::new();
    // do_ast_for_all_passes(&cli, &mut stats, "Operator Import", &mut asts, |ast, ast_ctx| {
    //     let mut pass = ast_passes::OperatorImport::new(ast_ctx, &name_table);
    //     pass.visit(ast);

    //     if !pass.imports.is_empty() {
    //         imported_operators = pass.imports;
    //     }
    // });

    // TODO: External operator importing happens here

    let mut use_table = UseTable::new();


    let mut hir = hir::Hir::new();
    do_ast_for_all_passes(&cli, &mut stats, "AST to HIR lowering", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::AstToHirLowering::new(ast_ctx, &mut name_table, &literal_table, &mut hir, &mut use_table, library_path.clone());
        pass.visit(ast);
    });
    stats.add_ast_hir_lower(&hir);

    // TODO: implicit prelude

    if cli.print_hir_nodes {
        println!("Lowered HIR:");
        let mut hir_logger = hir::NodeLogger::new(&name_table, &literal_table, &punct_table);
        hir_logger.visit(&mut hir, hir::VisitFlags::all());
        println!("--------------------------------")
    }

    if cli.print_hir_code {
        println!("Lowered HIR pseudo-code:");
        let mut hir_printer = hir::CodePrinter::new(&name_table, &literal_table, &punct_table);
        hir_printer.visit(&mut hir, hir::VisitFlags::all());
        println!("--------------------------------")
    }

    if cli.print_hir_use_table {
        println!("HIR use table");
        use_table.log();
        println!("--------------------------------")
    }

    let use_ambiguities = use_table.check_non_wildcard_ambiguity();
    if !use_ambiguities.is_empty() {
        println!("Use table ambiguities:");
        for (scope, name) in use_ambiguities {
            println!("- {scope}: {name}");
        }
    }

    {
        let mut sym_table = symbol_table.write();
        let precedence_dag = precedences.read();
        let mut op_table = operators.write();
        let ctx = HirProcessCtx {
            names: &name_table,
            puncts: &punct_table,
            sym_table: &mut sym_table,
            precedence_dag: &precedence_dag,
            op_table: &mut op_table,
            uses: &mut use_table,

            lib_path: library_path.clone(),

            errors: Vec::new(),
        };
        process_hir(&mut hir, &cli, &mut stats, ctx);

        if cli.print_hir_code {
            println!("--------------------------------");
            println!("Processed HIR pseudo-code:");
            let mut hir_printer = hir::CodePrinter::new(&name_table, &literal_table, &punct_table);
            hir_printer.visit(&mut hir, hir::VisitFlags::all());
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
        println!("Operator table");
        operators.read().log(&punct_table);
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

fn do_ast_for_all_passes<F>(cli: &Cli, stats: &mut CompilerStats, pass_name: &str, asts: &mut Vec<(ast::Ast, ast_passes::Context)>, mut f: F) where
    F: FnMut(&ast::Ast, &mut ast_passes::Context)
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

pub struct HirProcessCtx<'a> {
    names:          &'a NameTable,
    puncts:         &'a PuncutationTable,

    sym_table:      &'a mut SymbolTable,
    precedence_dag: &'a PrecedenceDAG,
    op_table:       &'a mut OperatorTable,
    uses:           &'a UseTable,

    lib_path:       LibraryPath,
    
    errors:         Vec<hir::HirError>,
}

fn process_hir(hir: &mut hir::Hir, cli: &Cli, stats: &mut CompilerStats, mut ctx: HirProcessCtx) -> bool {
    //do_hir_pass(hir, cli, stats, hir::passes::);
    
    // base passes
    do_hir_pass(hir, cli, stats, hir::passes::SymbolGeneration::new(ctx.sym_table, ctx.names));
    
    // Precedence and operators
    do_hir_pass(hir, cli, stats, hir::passes::PrecedenceProcessing::new(ctx.names, ctx.precedence_dag, ctx.sym_table, ctx.op_table, ctx.uses));
    do_hir_pass(hir, cli, stats, hir::passes::OperatorCollection::new(ctx.names, ctx.op_table, ctx.lib_path.clone()));
    do_hir_pass(hir, cli, stats, hir::passes::InfixReorder::new(ctx.puncts, ctx.op_table, ctx.precedence_dag, &mut ctx.errors));



    for err in &ctx.errors {
        println!("{err}");
    }
    !ctx.errors.is_empty()
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