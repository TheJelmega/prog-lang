use std::{
    env,
    time,
    fs::{self, File},
    io::Read,
    path::{Path, PathBuf},
};

use clap::Parser as _;
use ast::{Parser, Visitor};
use cli::Cli;
use common::NameTable;
use literals::LiteralTable;

mod error_warning;
mod literals;

mod common;

mod cli;

mod lexer;
mod ast;

fn main() {
    let cli = Cli::parse();

    let cwd = env::current_dir().unwrap().to_str().unwrap().to_string();
    println!("cwd: {cwd}");

    let total_start = time::Instant::now();

    for input_file in &cli.input_files {
        println!("File path: {cwd}/{input_file}");


        let mut file = File::open(input_file).unwrap();

        let mut buf = Vec::new();
        _ = file.read_to_end(&mut buf).unwrap();
    
        let file_content = unsafe { String::from_utf8_unchecked(buf) };
    
        let mut literal_table = LiteralTable::new();
        let mut name_table = NameTable::new();
        let mut punct_table = PuncutationTable::new();
    
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
            println!("Lexing {input_file} took {:.2} ms, generating {} tokens", lex_dur.as_secs_f32() * 1000.0, tokens.tokens.len()            );
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

        let parse_strart = time::Instant::now();

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
        ast.file = input_file.into();

        if cli.timings {
            let parse_dur = time::Instant::now() - parse_strart;
            println!("Parsing {input_file} took {:.2} ms, generating {} nodes", parse_dur.as_secs_f32() * 1000.0, ast.nodes.len() );
        }

        if cli.print_parse_output {
            ast.log(&name_table, &literal_table, &punct_table);
        }

        if cli.parse_only {
            continue;
        }
    }

    if cli.timings {
        let total_dur = time::Instant::now() - total_start;
        println!("Compiler took {:.2}s", total_dur.as_secs_f32());
    }
} 