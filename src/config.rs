use serde_derive::Deserialize;

use std::fs;

use crate::{
    custom_error::{CustomError, CustomResult},
    logger::LogLevel,
};

#[derive(Debug, Deserialize)]
pub struct WorkersConfig {
    node_workers: Box<[String]>,
    python_workers: Box<[String]>,
}

#[derive(Debug, Deserialize)]
pub struct GitConfig {
    pub version: Option<String>,
    pub branch: String,
    pub release_branch: String,
}

#[derive(Debug, Deserialize)]
pub struct AwsConfig {
    pub role_script_path: String,
    pub role: String,
    pub sso_start_url: String,
}

#[derive(Debug, Deserialize)]
pub struct LoggerConfig {
    pub log_level: LogLevel,
}
// Top level struct to hold the TOML data.
#[derive(Debug, Deserialize)]
pub struct Data {
    pub git: GitConfig,
    pub aws: AwsConfig,
    pub root: String,
    pub version_update_required: bool,
    pub repo_rebuild_required: bool,
    pub process_only_updated_repo: bool,
    pub disable_checks: bool,
    pub repos: WorkersConfig,
    pub logger: LoggerConfig,
}

#[derive(Debug, Clone, Copy)]
pub enum RepoType {
    Node,
    Python,
}

impl WorkersConfig {
    pub fn get_repos_list(&self) -> CustomResult<Vec<String>> {
        let mut repos: Vec<String> =
            Vec::with_capacity(self.node_workers.len() + self.python_workers.len());
        for repo in self.node_workers.iter() {
            repos.push(repo.to_string());
        }
        for repo in self.python_workers.iter() {
            repos.push(repo.to_string());
        }

        Ok(repos)
    }

    pub fn get_repo_type(&self, repo: &str) -> CustomResult<RepoType> {
        if self.node_workers.contains(&repo.to_string()) {
            return Ok(RepoType::Node);
        }
        if self.python_workers.contains(&repo.to_string()) {
            return Ok(RepoType::Python);
        }

        Err(CustomError::ConfigParsingError(
            "Unknown repo type".to_string(),
        ))
    }
}

pub fn read_config(path: &str) -> CustomResult<Data> {
    println!("Reading config file: {}", path);
    let contents =
        fs::read_to_string(path).unwrap_or_else(|_| panic!("Could not read file `{path}`"));

    let data: Data =
        toml::from_str(&contents).unwrap_or_else(|_| panic!("Unable to load data from `{path}`"));
    println!("Read config file: {}. {:#?}", path, data);

    Ok(data)
}
