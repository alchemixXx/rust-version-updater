use serde_derive::Deserialize;
use std::fs;
use std::process::exit;
use toml;


const CONFIG_FILE: &str = "updater_config.toml";

#[derive(Debug, Deserialize)]
struct WorkersConfig {
    node_workers: Box<[String]>,
    python_workers: Box<[String]>,
}

#[derive(Debug, Deserialize)]
struct GitConfig {
    version: String,
    branch: String,
}
// Top level struct to hold the TOML data.
#[derive(Debug, Deserialize)]
struct Data {
    git:GitConfig,
    root: String,
    repos: WorkersConfig,
}

impl WorkersConfig {
    fn get_repos_list(&self) -> Vec<String> {
        let mut repos = Vec::new();
        for repo in self.node_workers.iter() {
            repos.push(repo.to_string());
        }
        for repo in self.python_workers.iter() {
            repos.push(repo.to_string());
        }
        repos
    }
}


fn main() {
    println!("Hello, from batch version updater on rust!");
    println!("Starting version updater...");
    let config = read_config();

    let repos = config.repos.get_repos_list();
    println!("Repos to update: {:#?}", repos);
}


fn read_config() -> Data {
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
