use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use anyhow::bail;
use std::io::{Read, Write};
use tempfile::NamedTempFile;

use walkdir::WalkDir;

use crate::config::Config;
use crate::filehierarchy::FileHierarchy;
use crate::filesystem::{
    all_dirs_exist, create_all_dirs, file_autonamer, get_last_component, has_hidden,
};

pub fn list_files(paths: Vec<String>, config: &Config) -> Result<FileHierarchy, anyhow::Error> {
    let mut hierarchy = FileHierarchy::new();

    for path in paths {
        let process = PathBuf::from(&path);

        if config.recursive {
            for entry in WalkDir::new(&process) {
                let entry_path = if config.absolute {
                    entry?.into_path().canonicalize()?
                } else {
                    entry?.into_path()
                };

                if has_hidden(&entry_path) && config.ignore_hidden {
                    continue;
                }

                // check if dir
                if entry_path.is_file() {
                    hierarchy.insert(entry_path, 0);
                }
            }
        } else {
            hierarchy.insert(process, 0);
        }
    }

    hierarchy = hierarchy.enumerate();

    Ok(hierarchy)
}

pub fn open_editor(outcome: &str, config: &Config) -> Result<String, anyhow::Error> {
    let mut editor = match env::var("EDITOR") {
        Ok(val) => val,
        Err(_) => "vi".to_owned(),
    };

    if config.editor.is_some() {
        editor = config.editor.clone().unwrap();
    }

    let mut temp_file = NamedTempFile::new()?;
    write!(temp_file, "{}", outcome)?;
    temp_file.flush()?;

    let temp_file_path = temp_file.path();

    let command = if cfg!(target_os = "windows") {
        format!("cmd /C {}", editor)
    } else {
        editor.to_owned()
    };

    Command::new(command)
        .arg(temp_file_path)
        .spawn()
        .expect("Failed to spawn command")
        .wait_with_output()
        .expect("Failed to wait for command");

    // Re-open it.
    let mut file_edited = temp_file.reopen()?;

    let mut buf = String::new();
    file_edited.read_to_string(&mut buf)?;

    Ok(buf)
}

pub fn batch_operations(
    original: &FileHierarchy,
    modified: &FileHierarchy,
    config: &Config,
) -> Result<(), anyhow::Error> {
    if original.list.len() != modified.list.len() {
        bail!("Files are not matching creation and deletion are not allowed");
    }

    for path in original.list.iter() {
        let found = modified.get_by_file(&path.source);
        if found.is_some() {
            continue;
        }

        // always found because the 2 buffers are at the very least the same size
        let index_element = modified.get_by_index(path.position).unwrap();
        let mut destination = index_element.source.clone();

        // file has been renamed or moved
        let modified_component = get_last_component(&index_element.source);
        let original_component = get_last_component(&path.source);

        // components match, it has been moved else renamed
        if modified_component == original_component {
            // move and/or create
            // check if a file system with the same name exists
            if index_element.source.exists() && !config.automatic_rename {
                bail!("{:?} exists in the system", index_element.source);
            }

            if index_element.source.exists() && config.automatic_rename {
                destination = file_autonamer(&index_element.source);
            }
        }

        // TODO: show and ask for confirmation
        println!("{:?} -> {:?}", path.source, destination);
        if !all_dirs_exist(&destination) {
            if !config.mkdir {
                bail!("{:?} dirs do not exist", destination);
            }

            create_all_dirs(&destination)?;
        }

        fs::rename(&path.source, &destination)?;
    }

    Ok(())
}
