use std::{env, fs::File, io::{stdout, Read}, path::Path, time};

use lexer::{Lexer, NameTable, PuncutationTable};
use literals::LiteralTable;

mod error_warning;
mod literals;
mod lexer;

fn main() {
    println!("cwd: {}", env::current_dir().unwrap().to_str().unwrap());

    let total_start = time::Instant::now();

    let path_str = "../tests/test.xn";
    let path = Path::new(path_str);
    let mut file = File::open(path).unwrap();

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
            err.set_path(path_str.to_string());
            println!("{err}");
            return;
        },
    };
    let tokens = lexer.tokens;

    let lex_end = time::Instant::now();
    let lex_dur = lex_end - lex_start;
    println!("Lexer took {:.1} ms", lex_dur.as_secs_f32() * 1000.0);

    tokens.log(&literal_table, &name_table, &punct_table);

    let total_dur = time::Instant::now() - total_start;
    println!("Compiler took {:.2}s", total_dur.as_secs_f32());

    let mut stdout = stdout().lock();
    _ = tokens.log_csv(&mut stdout, &literal_table, &name_table, &punct_table);
} 