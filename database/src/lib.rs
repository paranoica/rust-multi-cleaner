use crate::structures::CleanerData;

pub mod cleaner_database;
pub mod registry_database;

mod registry_utils;
pub mod structures;
pub mod utils;

pub fn get_version() -> &'static str {
    option_env!("APP_VERSION").unwrap_or("1.9.8")
}

pub fn get_icon() -> &'static [u8; 3216] {
    let bytes: &'static [u8; 3216] = include_bytes!("../../assets/icon.png");
    bytes
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::cleaner_database::{get_database_from_file, get_default_database};
    use crate::structures::CleanerData;
    use crate::utils::get_file_size_string;

    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_get_version() {
        let version = get_version();

        assert!(!version.is_empty(), "Version must not be empty!");
        assert!(version.contains('.'), "Version must contain dots (semantic versioning)!");
    }

    #[test]
    fn test_get_icon() {
        let icon = get_icon();

        assert_eq!(icon.len(), 3216, "Icon must be exactly 3216 bytes!");
        assert_eq!(&icon[0..4], &[0x89, 0x50, 0x4E, 0x47], "Icon must be a PNG file!");
    }

    #[test]
    fn test_get_default_database() {
        let database = get_default_database();

        assert!(!database.is_empty(), "Default database should not be empty");
        for entry in database.iter() {
            assert!(!entry.path.is_empty(), "Path must not be empty!");
            assert!(!entry.category.is_empty(), "Category must not be empty!");
            assert!(!entry.program.is_empty(), "Program must not be empty!");
        }
    }

    #[test]
    fn test_database_decompression() {
        let database = get_default_database();

        assert!(database.len() > 100, "Database must contain many entries (decompression worked)!");
    }

    #[test]
    fn test_database_placeholder_expansion() {
        let database = get_default_database();
        for entry in database.iter() {
            assert!(!entry.path.contains("{username}"), "Username placeholder should be replaced in path: {}!", entry.path);
        }
    }

    #[test]
    fn test_database_from_file_invalid() {
        let result = get_database_from_file("nonexistent_file.json");
        
        assert!(result.is_err(), "Interpretator must return error for non-existent file!");
    }

    #[test]
    fn test_database_from_file_invalid_json() {
        let mut temp_file = NamedTempFile::new().unwrap();

        temp_file.write_all(b"{ invalid json }").unwrap();
        let result = get_database_from_file(temp_file.path().to_str().unwrap());

        assert!(result.is_err(), "Interpretator must return error for invalid JSON!");
    }

    #[test]
    fn test_database_from_file_valid() {
        let mut temp_file = NamedTempFile::new().unwrap();
        let json_data = r#"[
            {
                "path": "C:\\Test\\{username}\\file.txt",
                "category": "TestCategory",
                "program": "TestProgram",
                "class": "TestClass",
                "remove_files": true
            }
        ]"#;

        temp_file.write_all(json_data.as_bytes()).unwrap();
        let result = get_database_from_file(temp_file.path().to_str().unwrap());

        assert!(result.is_ok(), "Interpretator must load valid JSON successfully!");
        let database = result.unwrap();
        assert_eq!(database.len(), 1, "Interpretator must have one entry!");

        assert_eq!(database[0].program, "TestProgram");
        assert_eq!(database[0].category, "TestCategory");
    }

    #[test]
    fn test_file_size_string_formatting() {
        assert_eq!(get_file_size_string(0), "0 B");
        assert_eq!(get_file_size_string(512), "512 B");
        assert_eq!(get_file_size_string(1024), "1.0 KB");
        assert_eq!(get_file_size_string(1536), "1.5 KB");
        assert_eq!(get_file_size_string(1_048_576), "1.0 MB");
        assert_eq!(get_file_size_string(1_073_741_824), "1.0 GB");
        assert_eq!(get_file_size_string(1_099_511_627_776), "1.0 TB");
    }

    #[test]
    fn test_cleaner_data_structure() {
        let data = CleanerData {
            path: String::from("test/path"),
            category: String::from("Cache"),
            program: String::from("TestApp"),
            class: String::from("Application"),
            files_to_remove: vec![String::from("*.tmp")],
            directories_to_remove: vec![String::from("cache")],
            remove_all_in_dir: false,
            remove_directory_after_clean: true,
            remove_directories: true,
            remove_files: true,
        };

        assert_eq!(data.path, "test/path");
        assert_eq!(data.category, "Cache");
        assert_eq!(data.program, "TestApp");

        assert!(data.remove_files);
        assert!(data.remove_directories);
    }

    #[test]
    fn test_database_categories_exist() {
        let database = get_default_database();
        let categories: std::collections::HashSet<String> = database.iter().map(|entry| entry.category.clone()).collect();

        assert!(categories.len() > 0, "Interpretator must have at least one category!");
    }

    #[test]
    fn test_database_programs_exist() {
        let database = get_default_database();
        let programs: std::collections::HashSet<String> = database.iter().map(|entry| entry.program.clone()).collect();

        assert!(programs.len() > 10, "Interpretator must have no more than 10 different programs in database!");
    }

    #[test]
    fn test_database_performance() {
        use std::time::Instant;

        let start = Instant::now();
        let database = get_default_database();
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100, "Database loading time must be fast (< 100ms), took {:?}!",duration);
        assert!(!database.is_empty());
    }

    #[test]
    fn test_gzip_compression_ratio() {
        let database = get_default_database();

        assert!(database.len() > 500 ,"Database must have no more than 500 entries (compression ratio is good)!");
    }
}