
use serde_derive::Deserialize;

use std::fs;
use toml;


const CONFIG_FILE: &str = "updater_config.toml";

#[derive(Debug, Deserialize)]
pub struct WorkersConfig {
    node_workers: Box<[String]>,
    python_workers: Box<[String]>,
}

#[derive(Debug, Deserialize)]
pub struct GitConfig {
    pub version: String,
    pub branch: String,
    pub release_branch: String,
}
// Top level struct to hold the TOML data.
#[derive(Debug, Deserialize)]
pub struct Data {
    pub git:GitConfig,
    pub root: String,
    pub repos: WorkersConfig,
}

#[derive(Debug)]
pub enum RepoType {
    Node,
    Python,
    
}

impl WorkersConfig {
    pub fn get_repos_list(&self) -> Vec<String> {
        let mut repos: Vec<String> = Vec::new();
        for repo in self.node_workers.iter() {
            repos.push(repo.to_string());
        }
        for repo in self.python_workers.iter() {
            repos.push(repo.to_string());
        }
        repos
    }

    pub fn get_repo_type(&self, repo: &str) -> RepoType{
        if self.node_workers.contains(&repo.to_string()) {
            return RepoType::Node;
        }
        if self.python_workers.contains(&repo.to_string()) {
            return RepoType::Python;
        }
        
        panic!("Unknown repo type");
    }
}

pub fn read_config() -> Data {
    println!("Reading config file: {}", CONFIG_FILE);
    let contents = fs::read_to_string(CONFIG_FILE).expect(format!("Could not read file `{}`", CONFIG_FILE).as_str()); 

    let data: Data = toml::from_str(&contents).expect(format!("Unable to load data from `{}`", CONFIG_FILE).as_str());
    println!("Read config file: {}", CONFIG_FILE);
    println!("{:#?}", data);
    
    return data;
}