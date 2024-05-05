mod cli;
mod exec;
mod filelist;
mod filesystem;
mod config;

use std::process::exit;

use anyhow::bail;
use clap::Parser;

use crate::cli::Opts;
use crate::exec::{list_files, open_editor};
use crate::filelist::FileList;

use self::config::Config;
use self::exec::batch_operations;

fn main() {
    let result = run();

    match result {
        Err(err) => {
            println!("[rnr-buf error]: {:#}", err);
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

    let config = Config::new(&opts);

    let original = list_files(opts.paths, &config)?;

    let raw = open_editor(&original.raw, &config)?;

    let modified = FileList::new_from_raw(raw)?;

    batch_operations(&original, &modified, &config)?;

    Ok(())
}
