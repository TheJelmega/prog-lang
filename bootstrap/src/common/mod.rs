
mod logger;
use std::time;

pub use logger::*;

mod precedence;
pub use precedence::*;

mod names;
pub use names::*;

mod symbol_table;
pub use symbol_table::*;

mod scope;
pub use scope::*;









pub struct CompilerStats {
    pub file_count:          u64,
    pub lex_time:            time::Duration,
    pub bytes_parsed:        u64,
    pub chars_parsed:        u64,
    pub lines_parsed:        u64,
    pub tokens_generated:    u64,

    pub parse_time:          time::Duration,
    pub ast_nodes_generated: u64,

    pub ast_pass_time:       time::Duration,
}

impl CompilerStats {
    pub fn new() -> Self {
        Self {
            file_count: 0,
            lex_time: time::Duration::default(),
            bytes_parsed: 0,
            chars_parsed: 0,
            lines_parsed: 0,
            tokens_generated: 0,
            parse_time: time::Duration::default(),
            ast_nodes_generated: 0,
            ast_pass_time: time::Duration::default(),
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


    pub fn log(&self) {
        let logger = Logger::new();
        logger.log_fmt(format_args!("Files processed: \n"));

        logger.logln("- Lexer:");
        logger.log_fmt(format_args!("    Time: {:.2}ms\n", self.lex_time.as_secs_f32() * 1000.0));
        logger.log_fmt(format_args!("    Bytes processed: {} bytes ", self.bytes_parsed));

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

        logger.log_fmt(format_args!("    Chars processed: {}\n", self.chars_parsed));
        logger.log_fmt(format_args!("    Lines processed: {}\n", self.lines_parsed));
        logger.log_fmt(format_args!("    Tokens generated: {}\n", self.tokens_generated));

        logger.logln("- Parser:");
        logger.log_fmt(format_args!("    Time: {:.2}ms\n", self.parse_time.as_secs_f32() * 1000.0));
        logger.log_fmt(format_args!("    AST nodes generated: {}\n", self.ast_nodes_generated));

        logger.logln("- AST passes:");
        logger.log_fmt(format_args!("    Time: {:.2}ms\n", self.ast_pass_time.as_secs_f32() * 1000.0));
    }
}