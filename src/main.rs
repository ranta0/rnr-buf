mod cli;
mod config;
mod errors;
mod exec;
mod filelist;
mod filesystem;

use std::process::exit;

use anyhow::bail;
use clap::Parser;

use crate::cli::Opts;
use crate::errors::error_string;
use crate::exec::{list_files, open_editor};
use crate::filelist::FileList;

use self::config::Config;
use self::exec::batch_operations;

fn main() {
    let result = run();

    match result {
        Err(err) => {
            println!("{}{:#}", error_string(), err);
            exit(1);
        }
        _ => {
            exit(0);
        }
    }
}

fn run() -> Result<(), anyhow::Error> {
    let opts = Opts::parse();

    if opts.paths.is_empty() {
        bail!("No valid paths given.");
    }

    let config = Config::from_args(&opts);

    let original = list_files(opts.paths, &config)?;

    let raw = open_editor(&original.raw, &config)?;

    let modified = FileList::new_from_raw(raw)?;

    batch_operations(&original, &modified, &config)?;

    Ok(())
}
