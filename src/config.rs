
use serde_derive::Deserialize;

use std::fs;
use std::process::exit;
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

impl Data {
    pub fn update_version(&mut self, version: &str) {
        self.git.version = version.to_string();
    }
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
    let contents = match fs::read_to_string(CONFIG_FILE) {
        Ok(c) => c,
        Err(_) => {
            eprintln!("Could not read file `{}`", CONFIG_FILE);
            exit(1);
        }
    };

    let data: Data = match toml::from_str(&contents) {
        Ok(d) => d,
        Err(msg) => {
            eprintln!("{}", msg);
            eprintln!("Unable to load data from `{}`", CONFIG_FILE);
            exit(1);
        }
    };

    println!("Read config file: {}", CONFIG_FILE);
    println!("{:#?}", data);
    
    return data;
}