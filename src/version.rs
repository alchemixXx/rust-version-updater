use std::process::exit;
use serde_json;
use std::fs;
use std::path::Path;

const VERSION_FILES:[&str; 2] = ["package.json", "version.json"];

pub struct VersionSelecter {
    pub expected_version: String,
    pub repo: String
}

impl VersionSelecter {
    pub fn get_version(&self) -> String {
        if self.expected_version.is_empty() {
            let current_version =  self.read_version_file();
            let next_version = self.get_next_version_from_current(current_version);

            return next_version;
        }
        return self.expected_version.clone()
    }

    fn get_next_version_from_current(&self, mut current_version: String) -> String {
        let last: char = current_version.chars().last().unwrap();
        println!("last character in the version: '{}'", last);

    if last.is_ascii_alphabetic() && last != 'z' {
        let next_letter = (last as u8 + 1) as char;
        current_version.pop();

        return format!("{}{}", current_version, next_letter);
    } else if last == 'z' {
        current_version.pop();
        return  format!("{}aa", current_version);
    } else {
        return  format!("{}a", current_version);
    }
    }

    fn read_version_file(&self) -> String {
        for version_file in VERSION_FILES.iter() {
            let path=self.get_version_path(version_file);
            match fs::read_to_string(path) {
                Ok(content) => {
                    let json: serde_json::Value = serde_json::from_str(&content).expect("JSON was not well-formatted");
                    let version_value = &json["version"];

                    let mut version = version_value.to_string();

                    if version.starts_with("\"") && version.ends_with("\"") {
                        version = version.replace("\"", "");
                    } 
                    
                    
                    if version == "null" || version.is_empty() || version == "0.0.0" {
                        eprintln!("Version is empty in file `{}`", version_file);
                        continue;
                    }
                    println!("Got current version: {}", version);
                    return version;
                
                },
                Err(_) => {
                    eprintln!("Could not read file `{}`", version_file);
                }
            };


        }
        eprintln!("Could not get version from any of the files `{:?}`", VERSION_FILES);
        exit(1);
    }

    fn get_version_path(&self, file_name: &str) -> String {
        let path = Path::new(&self.repo).join(file_name);
        let path = match path.to_str() {
            Some(v) => v.to_string(),
            None => {
                println!("Can't build path: {}, {}", self.repo, file_name);
                exit(1);
            }
        };
        println!("Path: {}", path);
        path
    }
}

