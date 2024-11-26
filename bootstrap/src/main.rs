use std::{
    env, fs::{self, File}, io::Read, path::{Path, PathBuf}, sync::{Arc, RwLock}, time
};

use ast_passes::Context;
use clap::Parser as _;
use ast::{Parser, Visitor};
use cli::Cli;
use common::{NameTable, PrecedenceDAG, SymbolTable};
use lexer::{Lexer, PuncutationTable};
use literals::LiteralTable;

mod error_warning;
mod literals;

mod common;

mod cli;

mod lexer;
mod ast;
mod ast_passes;

fn main() {
    let cli = Cli::parse();

    let library = match &cli.library {
        Some(library) => library.clone(),
        None => cli.package.clone(),
    };

    let cwd = env::current_dir().unwrap().to_str().unwrap().to_string();
    println!("cwd: {cwd}");

    let total_start = time::Instant::now();

    let mut files_to_process = Vec::new();
    for input_file in &cli.input_files {
        files_to_process.push((input_file.clone(), Vec::new()));
    }
    
    let symbol_table = SymbolTable::new();
    let symbol_table = Arc::new(RwLock::new(symbol_table));

    let precedences = PrecedenceDAG::new();
    let precedences = Arc::new(RwLock::new(precedences));

    let mut asts = Vec::new();

    let mut literal_table = LiteralTable::new();
    let mut name_table = NameTable::new();
    let mut punct_table = PuncutationTable::new();

    while let Some((input_file, base_scope)) = files_to_process.pop() {
        println!("================================================================");
        println!("File path: {cwd}/{input_file}");


        let mut file = File::open(&input_file).unwrap();

        let mut buf = Vec::new();
        _ = file.read_to_end(&mut buf).unwrap();
    
        let file_content = unsafe { String::from_utf8_unchecked(buf) };
    
        let lex_start = time::Instant::now();
    
        let mut lexer = Lexer::new(&file_content, &mut literal_table, &mut name_table, &mut punct_table);
        match lexer.lex() {
            Ok(()) => {},
            Err(mut err) => {
                err.set_path(input_file.clone());
                println!("{err}");
                return;
            },
        };
        let tokens = lexer.tokens;
        
        if cli.timings {
            let lex_dur = time::Instant::now() - lex_start;
            println!("Lexing {input_file} took {:.2} ms, generating {} tokens", lex_dur.as_secs_f32() * 1000.0, tokens.tokens.len());
        }
    
        if cli.print_lex_output {
            tokens.log(&literal_table, &name_table, &punct_table);
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

            let mut csv_file = File::create(lex_csv_out_path).unwrap();
            _ = tokens.log_csv(&mut csv_file, &literal_table, &name_table, &punct_table);
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
                println!("{}({}): {err}", input_file, tok_meta.as_error_loc_string());
                return;
            },
        }

        let mut ast = parser.ast;
        ast.file = input_file.clone().into();

        if cli.timings {
            let parse_dur = time::Instant::now() - parse_start;
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
            symbol_table.clone(),
            base_scope.clone(),
            &ast,
            precedences.clone()
        );

        do_ast_pass(&cli, &input_file, "Context Setup", || {
            let mut pass = ast_passes::ContextSetup::new(&mut ast_ctx);
            pass.visit(&ast);
        });

        do_ast_pass(&cli, &input_file, "Module Scoping", || {
            let mut pass = ast_passes::ModuleScopePass::new(&mut ast_ctx, base_scope.clone(), &name_table);
            pass.visit(&ast);
        });
        
        do_ast_pass(&cli, &input_file, "Module Attribute Resolve", || {
            let mut pass = ast_passes::ModuleAttributeResolver::new(&mut ast_ctx, &name_table, &literal_table);
            pass.visit(&ast);
        });

        let mut sub_paths = Vec::new();
        do_ast_pass(&cli, &input_file, "Module Symbol Generation + Path Collection", || {
            let mut input_path = PathBuf::from(input_file.clone());
            input_path.pop();
            let mut pass = ast_passes::ModulePathResolution::new(&mut ast_ctx, &name_table, input_path);
            pass.visit(&ast);

            sub_paths = pass.collected_paths;
        });

        for err in &*ast_ctx.errors.lock().unwrap() {
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

    do_ast_for_all_passes(&cli, "Precedence Collection", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceCollection::new(ast_ctx, &name_table);
        pass.visit(ast);
    });

    let mut imported_precedences = Vec::new();
    do_ast_for_all_passes(&cli, "Precedence Import", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceImportCollection::new(ast_ctx, &name_table, cli.package.clone());
        pass.visit(ast);

        if !pass.imports.is_empty() {
            imported_precedences = pass.imports;
        }
    });

    // TODO: External precedences importing happens here

    do_ast_for_all_passes(&cli, "Precedence Connection", &mut asts, |ast, ast_ctx| {
        let mut pass = ast_passes::PrecedenceConnection::new(ast_ctx, &name_table);
        pass.visit(ast);
    });

    
    println!("================================================================");
    println!("Symbol table:");
    symbol_table.read().unwrap().log();

    println!("Precedence DAG Unordered:");
    precedences.read().unwrap().log_unordered();

    if cli.timings {
        let total_dur = time::Instant::now() - total_start;
        println!("Compiler took {:.2}s", total_dur.as_secs_f32());
    }
}

fn do_ast_pass<F>(cli: &Cli, input_file: &str, pass_name: &str, f: F) where 
    F: FnOnce()
{
    let start = time::Instant::now();
    f();
    if cli.pass_timings {
        let parse_dur = time::Instant::now() - start;
        println!("Processing AST Pass '{pass_name:32}' for '{input_file}' took {:.2} ms", parse_dur.as_secs_f32() * 1000.0);
    }
}

fn do_ast_for_all_passes<F>(cli: &Cli, pass_name: &str, asts: &mut Vec<(ast::Ast, ast_passes::Context)>, mut f: F) where
    F: FnMut(&ast::Ast, &mut ast_passes::Context)
{
    for (ast, ctx) in asts {
        let start = time::Instant::now();

        f(ast, ctx);
        
        if cli.pass_timings {
            let parse_dur = time::Instant::now() - start;
            let input_file = ast.file.to_str().unwrap();
            println!("Processing AST Pass '{pass_name:32}' for '{input_file}' took {:.2} ms", parse_dur.as_secs_f32() * 1000.0);
        }
    }
}