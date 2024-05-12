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
use crate::config::Config;
use crate::errors::error_string;
use crate::exec::{batch_operations, list_files, open_editor, perfom_operations};
use crate::filelist::FileList;

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

    let outcome = batch_operations(&original, &modified, &config)?;

    perfom_operations(&outcome, &config)?;

    Ok(())
}
