use crate::cli::Opts;

// Configuration options
pub struct Config {
    /// Whether to use the absolute path or not
    pub absolute: bool,

    /// By default it picks the system env EDITOR, otherwise the one given
    pub editor: Option<String>,

    /// Whether the command is recursive or not, default is false
    pub recursive: bool,

    /// Whether to not fail in case of finding a target with the same filename,
    /// it automatically adjusts it.
    pub automatic_rename: bool,

    /// Whether to ignore hidden files and directories (or not).
    pub ignore_hidden: bool,

    /// Whether to automatically create dirs or not.
    pub mkdir: bool,

    /// Confirm all changes.
    pub yes: bool,

    /// Whether to have terminal output or not. It will fail on error.
    pub quiet: bool,
    // /// whether to follow symlinks or not.
    // pub follow_links: bool,
    //
    // /// the maximum search depth, or `none` if no maximum search depth should be set.
    // ///
    // /// a depth of `1` includes all files under the current directory, a depth of `2` also includes
    // /// all files under subdirectories of the current directory, etc.
    // pub max_depth: option<usize>,
    //
    // /// the minimum depth for reported entries, or `none`.
    // pub min_depth: option<usize>,
    //
    // /// if true, the program doesn't print anything and will instead return an exit code of 0
    // /// if there's at least one match. otherwise, the exit code will be 1.
    // pub quiet: bool,
    //
    // /// the type of file to search for. if set to `none`, all file types are displayed. if
    // /// set to `some(..)`, only the types that are specified are shown.
    // pub file_types: option<filetype>,
    //
    // /// the extension to search for. only entries matching the extension will be included.
    // ///
    // /// the value (if present) will be a lowercase string without leading dots.
    // // pub extensions: option<regexset>,
    //
    // /// maximum number of search results to pass to each `command`. if zero, the number is
    // /// unlimited.
    // pub batch_size: usize,
    //
    // /// a list of custom ignore files.
    // pub ignore_files: vec<pathbuf>,
}

#[allow(dead_code)]
impl Config {
    pub fn new() -> Self {
        Self {
            automatic_rename: false,
            absolute: false,
            editor: None,
            ignore_hidden: false,
            recursive: true,
            mkdir: true,
            yes: false,
            quiet: false,
        }
    }

    pub fn from_args(opts: &Opts) -> Self {
        Self {
            absolute: opts.absolute,
            editor: opts.editor.clone(),
            recursive: opts.recursive,
            automatic_rename: opts.automatic_rename,
            ignore_hidden: opts.ignore_hidden,
            mkdir: opts.mkdir,
            yes: opts.yes,
            quiet: opts.quiet,
        }
    }
}
