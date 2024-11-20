use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Xenon bootstrap compiler")]
pub struct Cli {
    pub input_files: Vec<String>,

    #[arg(long)]
    pub print_lex_output: bool,
    #[arg(long)]
    pub output_lex_csv: bool,
    #[arg(long)]
    pub lex_only: bool,

    #[arg(long)]
    pub print_parse_output: bool,
    #[arg(long)]
    pub parse_only: bool,

    #[arg(long)]
    pub timings: bool,

    #[arg(long)]
    pub pass_timings: bool,
}