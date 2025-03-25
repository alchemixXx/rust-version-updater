use serde_json;
use std::fs;
use std::path::Path;

use crate::custom_error::{CustomError, CustomResult};
use crate::logger::LoggerTrait;

const VERSION_FILES: [&str; 2] = ["package.json", "version.json"];

pub struct VersionSelecter<'repo> {
    pub expected_version: &'repo Option<String>,
    pub repo: &'repo String,
    pub release: &'repo String,
}

impl LoggerTrait for VersionSelecter<'_> {}
impl VersionSelecter<'_> {
    pub fn get_version(&self) -> CustomResult<(String, String)> {
        let logger = self.get_logger();
        logger.debug("Getting current version...");
        let current_version = self.read_version_file()?;
        logger.debug(format!("Got current version: {}", current_version).as_str());

        match self.expected_version {
            Some(expected_version) => Ok((current_version, expected_version.clone())),
            None => {
                logger.debug("Expected version is empty. Getting current version from file...");

                if self.release == "5.8_RELEASE"
                    || self.release == "5.9_RELEASE"
                    || self.release == "5.91_RELEASE"
                {
                    let next_version =
                        self.get_next_char_version_from_current(current_version.clone())?;
                    logger.debug(format!("Got next version: {}", next_version).as_str());

                    Ok((current_version, next_version))
                } else {
                    let next_version =
                        self.get_next_digit_version_from_current(current_version.clone())?;
                    logger.debug(format!("Got next version: {}", next_version).as_str());

                    Ok((current_version, next_version))
                }
            }
        }
    }

    fn get_next_digit_version_from_current(
        &self,
        mut current_version: String,
    ) -> CustomResult<String> {
        let logger = self.get_logger();
        let last = current_version.split(".").last().unwrap();
        logger.debug(format!("last character in the version: '{}'", last).as_str());
        let last_number_result = last.parse::<u32>();

        match last_number_result {
            Ok(last_number) => {
                let next_number = last_number + 1;

                for _ in 0..last.len() {
                    current_version.pop();
                }
                Ok(format!("{current_version}{next_number}"))
            }
            Err(_) => {
                let mut next_letter: Option<char> = None;
                for ch in current_version.clone().chars().rev() {
                    if ch.is_ascii_digit() {
                        next_letter = Some(((ch as u8) + 1) as char);
                        current_version.pop();
                        break;
                    }
                    current_version.pop();
                }

                match next_letter {
                    Some(next_letter) => Ok(format!("{current_version}{next_letter}")),
                    None => Err(CustomError::VersionBuild(
                        "Could not get next version".to_string(),
                    )),
                }
            }
        }
    }

    fn get_next_char_version_from_current(
        &self,
        mut current_version: String,
    ) -> CustomResult<String> {
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
        Err(CustomError::VersionBuild(
            "Could not get version from any of the files".to_string(),
        ))
    }

    fn get_version_path(&self, file_name: &str) -> CustomResult<String> {
        let logger = self.get_logger();
        let path = Path::new(&self.repo).join(file_name);
        let path = path.to_str();

        match path {
            Some(path) => {
                logger.debug(format!("Path: {}", path).as_str());
                Ok(path.to_string())
            }
            None => Err(CustomError::VersionBuild(format!(
                "Can't build path: {}, {}",
                self.repo, file_name,
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_next_letter_version_from_current_0_should_be_0a() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("5.8_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_char_version_from_current(String::from("1.0.0"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.0a"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_letter_version_from_current_0a_should_be_0b() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("5.91_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_char_version_from_current(String::from("1.0.0a"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.0b"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_letter_version_from_current_0z_should_be_0aa() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("5.9_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_char_version_from_current(String::from("1.0.0z"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.0aa"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_letter_version_from_current_0az_should_be_0aaa() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("5.8_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_char_version_from_current(String::from("1.0.0az"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.0aaa"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_0_should_be_1() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("5.10_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("1.0.0"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.1"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_2a_should_be_3() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("10.1_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("1.0.2a"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.3"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_1z_should_be_2() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("11.0_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("1.0.1z"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.2"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_9_should_be_10() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("11.0_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("10.1.9"));

        match next_version {
            Ok(value) => assert_eq!(value, "10.1.10"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_19_should_be_20() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("11.0_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("10.1.19"));

        match next_version {
            Ok(value) => assert_eq!(value, "10.1.20"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_29_should_be_30() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("11.0_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("10.1.29"));

        match next_version {
            Ok(value) => assert_eq!(value, "10.1.30"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }

    #[test]
    fn test_get_next_digit_version_from_current_0az_should_be_1() {
        let version_selecter = VersionSelecter {
            expected_version: &None,
            repo: &String::from("/path/to/repo"),
            release: &String::from("5.10_RELEASE"),
        };

        let next_version =
            version_selecter.get_next_digit_version_from_current(String::from("1.0.0az"));

        match next_version {
            Ok(value) => assert_eq!(value, "1.0.1"),
            Err(err) => assert_eq!(err.to_string(), "Could not get next version"),
        }
    }
}
