use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "Xenon bootstrap compiler")]
pub struct Cli {
    pub input_files: Vec<String>,

    #[arg(long)]
    pub group:               Option<String>,
    #[arg(long)]
    pub package:             String,

    #[arg(long)]
    pub library:             Option<String>,

    #[arg(long)]
    pub print_lex_output:    bool,
    #[arg(long)]
    pub output_lex_csv:      bool,
    #[arg(long)]
    pub lex_only:            bool,

    #[arg(long)]
    pub print_parse_output:  bool,
    #[arg(long)]
    pub parse_only:          bool,

    #[arg(long)]
    pub print_hir_nodes:     bool,
    #[arg(long)]
    pub print_hir_code:      bool,
    #[arg(long)]
    pub print_hir_use_table: bool,

    #[arg(long)]
    pub print_sym_table:     bool,
    #[arg(long)]
    pub print_trait_dag:     bool,
    #[arg(long)]
    pub print_precedence:    bool,
    #[arg(long)]
    pub print_op_table:      bool,
    #[arg(long)]
    pub print_type_registry: bool,

    #[arg(long)]
    pub timings:             bool,

    #[arg(long)]
    pub pass_timings:        bool,
}