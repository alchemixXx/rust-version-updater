use serde_json;
use std::fs;
use std::path::Path;

const VERSION_FILES: [&str; 2] = ["package.json", "version.json"];

pub struct VersionSelecter<'repo> {
    pub expected_version: &'repo String,
    pub repo: &'repo String,
}

impl<'repo> VersionSelecter<'repo> {
    pub fn get_version(&self) -> (String, String) {
        println!("Getting current version...");
        let current_version = self.read_version_file();
        println!("Got current version: {}", current_version);

        if self.expected_version.is_empty() {
            println!("Expected version is empty. Getting current version from file...");

            let next_version = self.get_next_version_from_current(current_version.clone());
            println!("Got next version: {}", next_version);

            return (current_version, next_version);
        }

        return (current_version, self.expected_version.clone());
    }

    fn get_next_version_from_current(&self, mut current_version: String) -> String {
        let last: char = current_version.chars().last().unwrap();
        println!("last character in the version: '{}'", last);

        if last.is_ascii_alphabetic() && last != 'z' {
            let next_letter = ((last as u8) + 1) as char;
            current_version.pop();

            return format!("{}{}", current_version, next_letter);
        } else if last == 'z' {
            current_version.pop();
            return format!("{}aa", current_version);
        } else {
            return format!("{}a", current_version);
        }
    }

    fn read_version_file(&self) -> String {
        for version_file in VERSION_FILES.iter() {
            let path = self.get_version_path(version_file);
            match fs::read_to_string(path) {
                Ok(content) => {
                    let json: serde_json::Value = serde_json
                        ::from_str(&content)
                        .expect("JSON was not well-formatted");
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
                }
                Err(_) => {
                    eprintln!("Could not read file `{}`", version_file);
                }
            };
        }
        eprintln!("Could not get version from any of the files `{:?}`", VERSION_FILES);
        panic!("Could not get version from any of the files");
    }

    fn get_version_path(&self, file_name: &str) -> String {
        let path = Path::new(&self.repo).join(file_name);
        let path = path
            .to_str()
            .expect(format!("Can't build path: {}, {}", self.repo, file_name).as_str())
            .to_string();
        println!("Path: {}", path);
        path
    }
}

#[test]
fn test_get_next_version_from_current_0_should_be_0a() {
    let version_selecter = VersionSelecter {
        expected_version: &String::from(""),
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0"));

    assert_eq!(next_version, "1.0.0a");
}

#[test]
fn test_get_next_version_from_current_0a_should_be_0b() {
    let version_selecter = VersionSelecter {
        expected_version: &String::from(""),
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0a"));

    assert_eq!(next_version, "1.0.0b");
}

#[test]
fn test_get_next_version_from_current_0z_should_be_0aa() {
    let version_selecter = VersionSelecter {
        expected_version: &String::from(""),
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0z"));

    assert_eq!(next_version, "1.0.0aa");
}

#[test]
fn test_get_next_version_from_current_0az_should_be_0aaa() {
    let version_selecter = VersionSelecter {
        expected_version: &String::from(""),
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0az"));

    assert_eq!(next_version, "1.0.0aaa");
}
