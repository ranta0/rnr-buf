use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};

use anyhow::bail;
use dialoguer::Confirm;
use std::io::{Read, Write};
use tempfile::NamedTempFile;

use walkdir::WalkDir;

use crate::config::Config;
use crate::filelist::FileList;
use crate::filesystem::{
    all_dirs_exist, create_all_dirs, file_autonamer, get_last_component, has_hidden,
};

#[derive(Debug)]
pub struct Activity {
    pub mkdirs: Vec<PathBuf>,
    pub source: PathBuf,
    pub given_destination_path: PathBuf,
    pub destination: PathBuf,
}

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
) -> Result<Vec<Activity>, anyhow::Error> {
    if original.list.len() != modified.list.len() {
        bail!("Files are not matching creation and deletion are not allowed");
    }

    let mut outcome: Vec<Activity> = Vec::new();

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

        let mut activity = Activity {
            mkdirs: Vec::new(),
            source: path.source.clone(),
            given_destination_path: index_element.source.clone(),
            destination: destination.clone(),
        };

        let all_dirs_exist = all_dirs_exist(&destination);
        let mut dir_missing_created = false;
        if !all_dirs_exist {
            if config.mkdir {
                dir_missing_created = true;
                activity.mkdirs = create_all_dirs(&destination)?;
            }

            if !config.quiet {
                bail!("{:?} dirs do not exist", destination);
            }
        }

        if !destination.exists() && (all_dirs_exist || dir_missing_created) {
            outcome.push(activity);
        } else if !config.quiet {
            bail!("A file `{:?}` exists", destination);
        }
    }

    Ok(outcome)
}

pub fn perfom_operations(outcome: &Vec<Activity>, config: &Config) -> Result<(), anyhow::Error> {
    for activity in outcome {
        if !config.quiet {
            println!("{:?} -> {:?}", activity.source, activity.destination);
        }

        let mut confirmation = false;
        if !config.yes || !config.quiet {
            confirmation = Confirm::new()
                .with_prompt("Are you sure?")
                .interact()
                .unwrap();
        }

        if confirmation {
            fs::rename(&activity.source, &activity.destination)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::PathBuf;

    use super::Activity;
    use crate::config::Config;

    #[derive(Debug)]
    pub struct TestingActivity<'a> {
        // pub mkdirs: Vec<&'a str>,
        pub source: &'a str,
        pub destination: &'a str,
    }

    fn assert_compare_activities(
        activity: &Activity,
        testing_activity: &TestingActivity,
        temp_path: String,
    ) {
        use std::path::MAIN_SEPARATOR;

        match (activity.source.to_str(), activity.destination.to_str()) {
            (Some(source), Some(destination)) => {
                assert_eq!(
                    source,
                    format!("{}{}{}", temp_path, MAIN_SEPARATOR, testing_activity.source)
                );
                assert_eq!(
                    destination,
                    format!(
                        "{}{}{}",
                        temp_path, MAIN_SEPARATOR, testing_activity.destination
                    )
                );
            }
            _ => {
                panic!("error converting pathbuf to string.")
            }
        }
    }

    macro_rules! func_test_operations {
        ($($test_name:ident, $before_list:expr, $after_list:expr, $activities:expr, $config:expr)*) => {
            $(
                #[test]
                fn $test_name() {
                    use std::path::MAIN_SEPARATOR;

                    use crate::filelist::FileList;
                    use crate::exec::batch_operations;

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
                    //     - file_3.txt
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

                    let mut before_list_raw_string = String::new();
                    for string in &mut $before_list {
                        before_list_raw_string.push_str(format!("{}{}{}\n", temp_path, MAIN_SEPARATOR, string).as_str())
                    }

                    let mut after_list_raw_string = String::new();
                    for string in &mut $after_list {
                        after_list_raw_string.push_str(format!("{}{}{}\n", temp_path, MAIN_SEPARATOR, string).as_str())
                    }

                    match (FileList::new_from_raw(before_list_raw_string), FileList::new_from_raw(after_list_raw_string)) {
                        (Ok(before_list), Ok(after_list)) => {
                            match batch_operations(&before_list, &after_list, &$config) {
                                Ok(outcome) => {
                                    for (result, expected) in outcome.iter().zip($activities.iter()) {
                                        assert_compare_activities(
                                            result,
                                            expected,
                                            temp_path.to_owned()
                                        );
                                    }
                                }
                                Err(err) => {
                                    panic!("{}", err);
                                }
                            }
                        }
                        _ => {
                            panic!("Failed to create file list.");
                        }
                    }
                }
            )*
        };
    }

    func_test_operations!(
        test_batch_operations_move,
        ["file_1.txt", "file_3.txt"],
        ["file_1.txt", "file_4.txt"],
        [TestingActivity {
            source: "file_3.txt",
            destination: "file_4.txt",
        }],
        Config {
            automatic_rename: false,
            absolute: false,
            editor: None,
            ignore_hidden: false,
            recursive: true,
            mkdir: false,
            yes: false,
            quiet: false,
        }

        // other/other_file_1 exists
        test_batch_operations_rename,
        ["file_1.txt", "file_3.txt"],
        ["file_1.txt", "other/other_file_1.txt"],
        [TestingActivity {
            source: "file_3.txt",
            destination: "other/other_file_2.txt",
        }],
        Config {
            automatic_rename: true,
            absolute: false,
            editor: None,
            ignore_hidden: false,
            recursive: true,
            mkdir: false,
            yes: false,
            quiet: false,
        }

        test_batch_operations_move_many,
        ["file_1.txt", "file_2.txt", "other/other_file_1.txt"],
        ["file_10.txt", "file_20.txt", "other/other_file_10.txt"],
        [TestingActivity {
            source: "file_1.txt",
            destination: "file_10.txt",
        },
        TestingActivity {
            source: "file_2.txt",
            destination: "file_20.txt",
        },
        TestingActivity {
            source: "other/other_file_1.txt",
            destination: "other/other_file_10.txt",
        }],
        Config {
            automatic_rename: false,
            absolute: false,
            editor: None,
            ignore_hidden: false,
            recursive: true,
            mkdir: false,
            yes: false,
            quiet: false,
        }
    );

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
