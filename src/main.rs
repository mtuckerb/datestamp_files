use std::env;
use std::fs;
use walkdir::WalkDir;
use serde_json::json;
use dotenv::dotenv;
use clap::Parser;
use std::path::Path;
use reqwest::blocking::{ ClientBuilder};
use serde_json::Value; // Added from update
use regex::Regex; // Added from update
use chrono::{DateTime, Utc, Local, TimeZone, NaiveDate}; // Updated from update

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Path to the file or directory to process
    #[arg(value_name = "PATH")]
    path: String,

    /// Process a single file
    #[arg(short, long, conflicts_with = "directory")]
    file: bool,

    /// Process a directory
    #[arg(short, long, conflicts_with = "file")]
    directory: bool,

    /// Recursively process directories
    #[arg(short, long, requires = "directory")]
    recursive: bool,

    /// Use Obsidian mode
    #[arg(short, long)]
    obsidian: bool,
}

fn main() {
    dotenv().ok(); // Load the environment variables from .env file

    let obsidian_api_url = env::var("OBSIDIAN_API_URL")
        .expect("OBSIDIAN_API_URL must be set in .env file");
    let obsidian_api_key = env::var("OBSIDIAN_API_KEY")
        .expect("OBSIDIAN_API_KEY must be set in .env file");

    let cli = Cli::parse();

    let result = if cli.file {
        if cli.obsidian {
            rename_file_obsidian(&cli.path, &obsidian_api_url, &obsidian_api_key)
        } else {
            rename_file(Path::new(&cli.path))
        }
    } else if cli.directory {
        if cli.recursive {
            if cli.obsidian {
                rename_files_in_directory_recursive_obsidian(&cli.path, &obsidian_api_url, &obsidian_api_key)
            } else {
                rename_files_in_directory_recursive(Path::new(&cli.path))
            }
        } else {
            if cli.obsidian {
                rename_files_in_directory_obsidian(&cli.path, &obsidian_api_url, &obsidian_api_key)
            } else {
                rename_files_in_directory(Path::new(&cli.path))
            }
        }
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "You must specify either -f (file) or -d (directory)."))
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn rename_file(path: &Path) -> std::io::Result<()> {
    rename_single_file(path)
}
fn rename_files_in_directory(dir: &Path) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            rename_file(&path)?;
        }
    }
    Ok(())
}

fn rename_files_in_directory_recursive(dir: &Path) -> std::io::Result<()> {
    for entry in WalkDir::new(dir).into_iter().filter_map(Result::ok) {
        let path = entry.path();
        if path.is_file() {
            rename_file(path)?;
        }
    }
    Ok(())
}

fn rename_single_file(path: &Path) -> std::io::Result<()> {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    
    // Check if the file already starts with a date in the correct format
    if Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap().is_match(file_name) {
        println!("Skipping {:?} as it already starts with a date in the correct format", path);
        return Ok(());
    }

    // Try to parse date from filename
    if let Some(date) = parse_date_from_filename(file_name) {
        let date_str = date.format("%Y-%m-%d").to_string();
        let new_name = format!("{} - {}", date_str, file_name);
        let new_path = path.with_file_name(new_name);
        fs::rename(path, &new_path)?;
        println!("Renamed {:?} to {:?}", path, new_path);
    } else {
        // If no date found in filename, use file creation date
        let metadata = fs::metadata(path)?;
        let created: DateTime<Local> = metadata.created()?.into();
        let date_str = created.format("%Y-%m-%d").to_string();
        let new_name = format!("{} - {}", date_str, file_name);
        let new_path = path.with_file_name(new_name);
        fs::rename(path, &new_path)?;
        println!("Renamed {:?} to {:?}", path, new_path);
    }
    Ok(())
}


fn rename_file_obsidian(path: &str, api_url: &str, api_key: &str) -> std::io::Result<()> {
    rename_single_file_obsidian(path, api_url, api_key)
}

fn rename_single_file_obsidian(path: &str, api_url: &str, api_key: &str) -> std::io::Result<()> {
    let client = create_insecure_client();
    if path.is_empty() {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Path cannot be empty"));
    }

    let file_name = Path::new(path).file_name().unwrap().to_str().unwrap();

    // Check if the file already starts with a date in the correct format
    if Regex::new(r"^\d{4}-\d{2}-\d{2}").unwrap().is_match(file_name) {
        println!("Skipping {} as it already starts with a date in the correct format", path);
        return Ok(());
    }

    let response = client.get(format!("{}/vault/{}", api_url, path))
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Accept", "application/vnd.olrapi.note+json")
        .send()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

    if response.status().is_success() {
        let file_info: serde_json::Value = response.json()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        
        let date_str = if let Some(date) = parse_date_from_filename(file_name) {
            date.format("%Y-%m-%d").to_string()
        } else {
            let created = file_info["stat"]["ctime"]
                .as_i64()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing or invalid creation time"))?;
            
            let date_time: DateTime<Utc> = Utc.timestamp_millis_opt(created).single()
                .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid timestamp"))?;
            
            date_time.format("%Y-%m-%d").to_string()
        };

        let new_name = format!("{} - {}", date_str, file_name);
        let new_path = Path::new(path).with_file_name(&new_name);
        
        println!("Renaming {:?} to {:?}", path, new_path);
        
        let rename_response = client.put(format!("{}/rename", api_url))
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&json!({
                "oldPath": path,
                "newPath": new_path.to_str().unwrap()
            }))
            .send()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        if rename_response.status().is_success() {
            println!("Renamed {} to {}", path, new_path.to_str().unwrap());
            Ok(())
        } else {
            Err(std::io::Error::new(std::io::ErrorKind::Other, format!("API error: {}", rename_response.status())))
        }
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, format!("API error: {}", response.status())))
    }
}

fn rename_files_in_directory_obsidian(dir: &str, api_url: &str, api_key: &str) -> std::io::Result<()> {
    let files = query_obsidian_vault(dir, api_url, api_key)?;
    for file in files {
        let _ = rename_file_obsidian(&format!("{}{}",&dir,&file), api_url, api_key);
    }
    Ok(())
}

fn rename_files_in_directory_recursive_obsidian(dir: &str, api_url: &str, api_key: &str) -> std::io::Result<()> {
    let files = query_obsidian_vault(dir, api_url, api_key)?;
    for file in files {
        let _ = rename_file_obsidian(&format!("{}{}",&dir,&file), api_url, api_key);
    }
    Ok(())
}
fn create_insecure_client() -> reqwest::blocking::Client {
    ClientBuilder::new()
        .danger_accept_invalid_certs(true)
        .build()
        .expect("Failed to create client")
}

fn query_obsidian_vault(path: &str, api_url: &str, api_key: &str) -> std::io::Result<Vec<String>> {
    let client = create_insecure_client();
    let response = client.get(format!("{}/vault/{}", api_url, path))
        .header("Authorization", format!("Bearer {}", api_key))
        .send()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send request: {}", e)))?;

    let status = response.status();
    println!("Response status: {}", &status);
    
    let body = response.text()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to read response body: {}", e)))?;
    
    println!("Response body: {}", &body);

    if status.is_success() {
        if body.is_empty() {
            println!("Warning: Empty response body");
            Ok(vec![])
        } else {
            match serde_json::from_str::<Value>(&body) {
                Ok(json) => {
                    if let Some(files) = json["files"].as_array() {
                        let file_names: Vec<String> = files
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                        Ok(file_names)
                    } else {
                        Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "JSON does not contain a 'files' array"))
                    }
                },
                Err(e) => {
                    println!("Error parsing JSON: {}", e);
                    println!("Raw response body: {}", body);
                    Err(std::io::Error::new(std::io::ErrorKind::InvalidData, format!("Failed to parse JSON: {}. Raw body: {}", e, body)))
                }
            }
        }
    } else {
        Err(std::io::Error::new(std::io::ErrorKind::Other, format!("API error: {}. Response body: {}", &status, body)))
    }
}

fn parse_date_from_filename(filename: &str) -> Option<NaiveDate> {
    fn ymd(y: i32, m: i32, d: i32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m as u32, d as u32)
    }
    fn mdy(m: i32, d: i32, y: i32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m as u32, d as u32)
    }
    fn dmy(d: i32, m: i32, y: i32) -> Option<NaiveDate> {
        NaiveDate::from_ymd_opt(y, m as u32, d as u32)
    }

    let date_patterns: [(Regex, fn(i32, i32, i32) -> Option<NaiveDate>); 6] = [
        (Regex::new(r"^(\d{4})-(\d{2})-(\d{2})").unwrap(), ymd),
        (Regex::new(r"(\d{4})(\d{2})(\d{2})").unwrap(), ymd),
        (Regex::new(r"(\d{2})(\d{2})(\d{4})").unwrap(), mdy),
        (Regex::new(r"(\d{2})-(\d{2})-(\d{4})").unwrap(), mdy),
        (Regex::new(r"(\d{2})(\d{2})(\d{4})").unwrap(), dmy),
        (Regex::new(r"(\d{2})-(\d{2})-(\d{4})").unwrap(), dmy),
    ];

    for (regex, date_constructor) in &date_patterns {
        if let Some(captures) = regex.captures(filename) {
            if captures.len() == 4 {
                let a = captures[1].parse::<i32>().unwrap();
                let b = captures[2].parse::<i32>().unwrap();
                let c = captures[3].parse::<i32>().unwrap();
                if let Some(date) = date_constructor(a, b, c) {
                    return Some(date);
                }
            }
        }
    }
    None
}