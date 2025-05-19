use std::env;
use std::fs;
use std::path::Path;
use chrono::{DateTime, Local};
use walkdir::WalkDir;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file_or_directory> [-r|--recursive]", args[0]);
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let recursive = args.len() > 2 && (args[2] == "-r" || args[2] == "--recursive");

    if path.is_file() {
        rename_file(path);
    } else if path.is_dir() {
        if recursive {
            rename_files_in_directory_recursive(path);
        } else {
            rename_files_in_directory(path);
        }
    } else {
        eprintln!("Error: The provided path is neither a file nor a directory.");
        std::process::exit(1);
    }
}

fn rename_file(path: &Path) {
    if let Err(e) = rename_single_file(path) {
        eprintln!("Error renaming file {:?}: {}", path, e);
    }
}

fn rename_single_file(path: &Path) -> std::io::Result<()> {
    let metadata = fs::metadata(path)?;
    let created: DateTime<Local> = metadata.created()?.into();
    let date_str = created.format("%Y-%m-%d").to_string();

    let file_name = path.file_name().unwrap().to_str().unwrap();
    let new_name = format!("{} - {}", date_str, file_name);
    let new_path = path.with_file_name(new_name);

    fs::rename(path, &new_path)?;
    println!("Renamed {:?} to {:?}", path, new_path);
    Ok(())
}

fn rename_files_in_directory(dir: &Path) {
    for entry in fs::read_dir(dir).expect("Failed to read directory") {
        if let Ok(entry) = entry {
            let path = entry.path();
            if path.is_file() {
                rename_file(&path);
            }
        }
    }
}

fn rename_files_in_directory_recursive(dir: &Path) {
    for entry in WalkDir::new(dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() {
            rename_file(path);
        }
    }
}