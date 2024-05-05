use std::fs;
use std::path::{Path, PathBuf};

//
// pub fn rename(source: &Path, target: &Path) -> Result<(), anyhow::Error> {
//     let source_path = PathBuf::from("/path/to/source/file");
//     let destination_path = PathBuf::from("/path/to/destination/file");
//
//     fs::rename(&source_path, &destination_path)?;
//     Ok(())
// }
//
// pub fn create_dir() {
//     todo!();
// }

/// Get the last part of the path
pub fn get_last_component(path: &Path) -> &str {
    match path.iter().last() {
        Some(component) => component.to_str().unwrap_or(""),
        None => "",
    }
}

/// Wether the dirs in a file exist or not
pub fn all_dirs_exist(path: &Path) -> bool {
    let mut current_path = PathBuf::new();

    for component in path.components() {
        current_path.push(component);

        if !current_path.exists() {
            return false;
        }
    }

    true
}

/// Wether path contains a hidden component
pub fn has_hidden(path: &Path) -> bool {
    for component in path.components() {
        let component_str = component.as_os_str().to_string_lossy();

        if component_str.starts_with('.') {
            return true;
        }
    }

    false
}

pub fn create_all_dirs(path: &Path) -> Result<(), anyhow::Error> {
    let mut current_path = PathBuf::new();

    for component in path.components() {
        current_path.push(component);
        if current_path.extension().is_none() && !current_path.exists() {
            fs::create_dir(&current_path)?;
        }
    }

    Ok(())
}

/// Given a pathbuf generate the next in line automatic
pub fn file_autonamer(path: &Path) -> PathBuf {
    let mut new_file_path = path.to_path_buf();

    // Check if the file already exists
    while new_file_path.exists() {
        // If it does, add a number to the filename and try again
        let file_name = path.file_name().unwrap().to_string_lossy();
        let (mut name, extension) = match file_name.rsplit_once('.') {
            Some((name, ext)) => (name, ext),
            None => (file_name.as_ref(), ""),
        };

        let (name_split, after_split) = match name.rsplit_once('_') {
            Some((name, number)) => (name, number),
            None => (name, ""),
        };

        // Find the next available number
        let mut number = 1;
        print!("{:?}", after_split);

        if !after_split.is_empty() {
            // Parse the string into an integer
            if let Ok(num) = after_split.parse::<i32>() {
                number = num;
                name = name_split;
            };
        }

        loop {
            let numbered_name = format!("{}_{}.{}", name, number, extension);
            new_file_path.set_file_name(&numbered_name);
            if !new_file_path.exists() {
                break;
            }
            number += 1;
        }
    }

    new_file_path
}