use clap::Parser;

#[derive(Parser)]
#[command(
    name = "rnr-buf",
    version,
    about = "Rename multiple files",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/ranta0/rnr-buf/issues",
    max_term_width = 98,
    args_override_self = true
)]
pub struct Opts {
    // File names
    pub paths: Vec<String>,

    /// It will automatically look for $EDITOR by default
    #[arg(short, long)]
    pub editor: Option<String>,

    #[arg(short = 'R', long)]
    pub recursive: bool,

    #[arg(short = 'a', long)]
    pub absolute: bool,

    #[arg(long)]
    pub automatic_rename: bool,

    #[arg(short = 'H', long)]
    pub ignore_hidden: bool,

    #[arg(long)]
    pub mkdir: bool,
}
