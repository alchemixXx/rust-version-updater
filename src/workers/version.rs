use serde_json;
use std::fs;
use std::path::Path;

use crate::custom_error::{CustomError, CustomResult};
use crate::logger::LoggerTrait;

const VERSION_FILES: [&str; 2] = ["package.json", "version.json"];

pub struct VersionSelecter<'repo> {
    pub expected_version: &'repo Option<String>,
    pub repo: &'repo String,
}

impl<'config> LoggerTrait for VersionSelecter<'config> {}
impl<'repo> VersionSelecter<'repo> {
    pub fn get_version(&self) -> CustomResult<(String, String)> {
        let logger = self.get_logger();
        logger.debug("Getting current version...");
        let current_version = self.read_version_file()?;
        logger.debug(format!("Got current version: {}", current_version).as_str());

        match self.expected_version {
            Some(expected_version) => Ok((current_version, expected_version.clone())),
            None => {
                logger.debug("Expected version is empty. Getting current version from file...");

                let next_version = self.get_next_version_from_current(current_version.clone())?;
                logger.debug(format!("Got next version: {}", next_version).as_str());

                Ok((current_version, next_version))
            }
        }
    }

    fn get_next_version_from_current(&self, mut current_version: String) -> CustomResult<String> {
        let logger = self.get_logger();
        let last: char = current_version.chars().last().unwrap();
        logger.debug(format!("last character in the version: '{}'", last).as_str());

        if last.is_ascii_alphabetic() && last != 'z' {
            let next_letter = ((last as u8) + 1) as char;
            current_version.pop();

            Ok(format!("{current_version}{next_letter}"))
        } else if last == 'z' {
            current_version.pop();
            return Ok(format!("{}aa", current_version));
        } else {
            return Ok(format!("{}a", current_version));
        }
    }

    fn read_version_file(&self) -> CustomResult<String> {
        let logger = self.get_logger();
        for version_file in VERSION_FILES.iter() {
            let path = self.get_version_path(version_file)?;
            match fs::read_to_string(path) {
                Ok(content) => {
                    let json: serde_json::Value =
                        serde_json::from_str(&content).expect("JSON was not well-formatted");
                    let version_value = &json["version"];

                    let mut version = version_value.to_string();

                    if version.starts_with('\"') && version.ends_with('\"') {
                        version = version.replace('\"', "");
                    }

                    if version == "null" || version.is_empty() || version == "0.0.0" {
                        logger
                            .error(format!("Version is empty in file `{}`", version_file).as_str());
                        continue;
                    }
                    logger.debug(format!("Got current version: {}", version).as_str());
                    return Ok(version);
                }
                Err(_) => {
                    logger.error(format!("Could not read file `{}`", version_file).as_str());
                }
            };
        }
        logger.error(
            format!(
                "Could not get version from any of the files `{:?}`",
                VERSION_FILES
            )
            .as_str(),
        );
        return Err(CustomError::VersionBuild(
            "Could not get version from any of the files".to_string(),
        ));
    }

    fn get_version_path(&self, file_name: &str) -> CustomResult<String> {
        let logger = self.get_logger();
        let path = Path::new(&self.repo).join(file_name);
        let path = path.to_str();

        match path {
            Some(path) => {
                logger.debug(format!("Path: {}", path).as_str());
                return Ok(path.to_string());
            }
            None => {
                return Err(CustomError::VersionBuild(format!(
                    "Can't build path: {}, {}",
                    self.repo, file_name,
                )));
            }
        }
    }
}

#[test]
fn test_get_next_version_from_current_0_should_be_0a() {
    let version_selecter = VersionSelecter {
        expected_version: &None,
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0"));

    match next_version {
        Ok(value) => assert_eq!(value, "1.0.0a"),
        Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
    }
}

#[test]
fn test_get_next_version_from_current_0a_should_be_0b() {
    let version_selecter = VersionSelecter {
        expected_version: &None,
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0a"));

    match next_version {
        Ok(value) => assert_eq!(value, "1.0.0b"),
        Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
    }
}

#[test]
fn test_get_next_version_from_current_0z_should_be_0aa() {
    let version_selecter = VersionSelecter {
        expected_version: &None,
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0z"));

    match next_version {
        Ok(value) => assert_eq!(value, "1.0.0aa"),
        Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
    }
}

#[test]
fn test_get_next_version_from_current_0az_should_be_0aaa() {
    let version_selecter = VersionSelecter {
        expected_version: &None,
        repo: &String::from("/path/to/repo"),
    };

    let next_version = version_selecter.get_next_version_from_current(String::from("1.0.0az"));

    match next_version {
        Ok(value) => assert_eq!(value, "1.0.0aaa"),
        Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
    }
}
