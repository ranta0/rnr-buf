use clap::Parser;

#[derive(Parser)]
#[command(
    name = "rnr-buf",
    version,
    about = "Rename multiple files",
    after_long_help = "Bugs can be reported on GitHub: https://github.com/ranta0/rnr-buf/issues",
    max_term_width = 80,
    args_override_self = true
)]
pub struct Opts {
    // File names
    pub paths: Vec<String>,

    /// By default it picks the system env EDITOR, otherwise the one given
    #[arg(short, long)]
    pub editor: Option<String>,

    #[arg(short = 'R', long)]
    pub recursive: bool,

    /// Whether to use the absolute path or not
    #[arg(short = 'a', long)]
    pub absolute: bool,

    /// Whether to not fail in case of finding a target with the same filename,
    /// it automatically adjusts it.
    #[arg(long)]
    pub automatic_rename: bool,

    /// Whether to ignore hidden files and directories (or not).
    #[arg(long)]
    pub ignore_hidden: bool,

    /// Whether to automatically create dirs or not.
    #[arg(long)]
    pub mkdir: bool,

    /// Confirm all changes.
    #[arg(short = 'y', long)]
    pub yes: bool,

    /// Whether to have terminal output or not. It will fail on error.
    #[arg(short = 'q', long)]
    pub quiet: bool,
}
