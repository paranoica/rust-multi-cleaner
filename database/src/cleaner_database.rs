use crate::CleanerData;
use crate::registry_utils::get_steam_directory_from_registry;

use disk_name::get_letters;
use flate2::read::GzDecoder;

use serde_json;
use std::error::Error;

use std::fs;
use std::io::Read;
use std::sync::OnceLock;

static DATABASE: OnceLock<Vec<CleanerData>> = OnceLock::new();

pub fn get_default_database() -> &'static Vec<CleanerData> {
    DATABASE.get_or_init(|| {
        #[cfg(unix)]
        let compressed_data = include_bytes!(concat!(env!("OUT_DIR"), "/linux_database.min.json.gz"));

        #[cfg(windows)]
        let compressed_data = include_bytes!(concat!(env!("OUT_DIR"), "/windows_database.min.json.gz"));

        let mut decoder = GzDecoder::new(&compressed_data[..]);
        let mut json_data = String::new();

        decoder.read_to_string(&mut json_data).expect("Failed to decompress database");
        let database: Vec<CleanerData> = serde_json::from_str::<Vec<CleanerData>>(&json_data).expect("Failed to parse database");

        let username = whoami::username();
        let drive_host = if cfg!(windows) { get_letters() } else { vec![] };

        let steam_directory = if cfg!(windows) {
            get_steam_directory_from_registry()
        } else {
            String::new()
        };

        let mut expanded_database = Vec::new();
        for entry in database {
            let mut new_entry = entry.clone();

            new_entry.path = new_entry.path.replace("{username}", &username);
            new_entry.path = new_entry.path.replace("{steam}", &steam_directory);

            if cfg!(windows) && new_entry.path.contains("{drive}") {
                for drive in &drive_host {
                    let mut drive_entry = new_entry.clone();
                    
                    drive_entry.path = drive_entry.path.replace("{drive}", drive);
                    expanded_database.push(drive_entry);
                }
            } else {
                expanded_database.push(new_entry);
            }
        }

        expanded_database
    })
}

pub fn get_database_from_file(file_path: &str) -> Result<Vec<CleanerData>, Box<dyn Error>> {
    let data = fs::read_to_string(file_path)?;
    let database: Vec<CleanerData> = serde_json::from_str(&data)?;

    let username = whoami::username();
    let drive_host = if cfg!(windows) { get_letters() } else { vec![] };

    let steam_directory = if cfg!(windows) {
        get_steam_directory_from_registry()
    } else {
        String::new()
    };

    let mut expanded_database = Vec::new();
    for entry in database {
        let mut new_entry = entry.clone();

        new_entry.path = new_entry.path.replace("{username}", &username);
        new_entry.path = new_entry.path.replace("{steam}", &steam_directory);

        if cfg!(windows) && new_entry.path.contains("{drive}") {
            for drive in &drive_host {
                let mut drive_entry = new_entry.clone();

                drive_entry.path = drive_entry.path.replace("{drive}", drive);
                expanded_database.push(drive_entry);
            }
        } else {
            expanded_database.push(new_entry);
        }
    }

    Ok(expanded_database)
}