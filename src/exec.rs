use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use anyhow::bail;
use dialoguer::Confirm;
use std::io::{Read, Write};
use tempfile::NamedTempFile;

use walkdir::WalkDir;

use crate::config::Config;
use crate::errors::error_string;
use crate::filelist::FileList;
use crate::filesystem::{
    all_dirs_exist, create_all_dirs, file_autonamer, get_last_component, has_hidden,
};

pub fn list_files(paths: Vec<String>, config: &Config) -> Result<FileList, anyhow::Error> {
    let mut list = FileList::new();

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
                    list.insert(entry_path, 0);
                }
            }
        } else {
            list.insert(process, 0);
        }
    }

    list.enumerate();

    Ok(list)
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
    original: &FileList,
    modified: &FileList,
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

        // always found because the 2 buffers are the same size
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

        if destination.exists() && config.automatic_rename {
            destination = file_autonamer(&destination);
        }

        if !config.quiet {
            println!("{:?} -> {:?}", path.source, destination);
        }

        let mut confirmation = false;
        if !config.yes || !config.quiet {
            confirmation = Confirm::new()
                .with_prompt("Are you sure?")
                .interact()
                .unwrap();
        }

        if confirmation {
            if !all_dirs_exist(&destination) {
                if !config.mkdir {
                    bail!("{:?} dirs do not exist", destination);
                }

                create_all_dirs(&destination)?;
            }

            if !destination.exists() {
                fs::rename(&path.source, &destination)?;
            } else if config.quiet {
                bail!("A file `{:?}` exists", destination);
            } else {
                println!("{}A file `{:?}` exists", error_string(), destination);
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;

    use crate::config::Config;

    #[test]
    fn test_list_files_recursive() {
        use crate::exec::list_files;
        let tempdir = tempfile::tempdir().expect("Error creating temp directory");
        let temp_path = tempdir.path().to_str().unwrap();

        let mock_dir = format!("{}/other", temp_path);
        let mock_files: Vec<String> = vec![
            format!("{}/file_1.txt", temp_path),
            format!("{}/file_2.txt", temp_path),
            format!("{}/file_3.txt", temp_path),
            format!("{}/file_1.txt", mock_dir),
            format!("{}/other_file_1.txt", mock_dir),
        ];

        // - tmp
        //     |
        //     - file_1.txt
        //     |
        //     - file_2.txt
        //     |
        //     - file_2.txt
        //     |
        //     - other
        //         |
        //         - file_1.txt
        //         |
        //         - other_file_1.txt

        fs::create_dir(&mock_dir).expect("Error creating mock directory...");
        for file in &mock_files {
            fs::File::create(file).expect("Error creating mock file...");
        }

        let mock_config = Config::new();

        if let Ok(result) = list_files(vec![temp_path.to_owned()], &mock_config) {
            let first = result.get_by_index(0).unwrap_or_else(|| {
                panic!("Failed to get the first item from the list.");
            });
            let last = result.get_by_index(4).unwrap_or_else(|| {
                panic!("Failed to get the last item from the list.");
            });

            assert_eq!(
                PathBuf::from(format!("{}{}", temp_path, "/file_1.txt")),
                first.source
            );
            assert_eq!(
                PathBuf::from(format!("{}{}", temp_path, "/other/other_file_1.txt")),
                last.source
            );
        } else {
            panic!("Failed to create FileList from raw data.");
        }
    }

    #[test]
    #[should_panic]
    fn test_editor() {
        use super::open_editor;
        let list_files = "tmp/file_1.txt\n".to_owned();

        // Open a fake editor
        let mut mock_config = Config::new();
        mock_config.editor = Some("fake_cmd_editor".to_owned());

        if open_editor(&list_files, &mock_config).is_err() {
            panic!("Failed to open editor.");
        }
    }
}
