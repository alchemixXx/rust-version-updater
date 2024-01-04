use std::path::Path;
use std::process::exit;
mod version;
mod config;



fn main() {
    println!("Hello, from batch version updater on rust!");
    println!("Starting version updater...");
    let mut config = config::read_config();

    let repos = config.repos.get_repos_list();
    println!("Repos to update: {:#?}", repos);

    for repo in repos.iter() {
        let repo_path = get_repo_path(&config.root, repo);
        let selecter = version::VersionSelecter {
            expected_version: config.git.version.clone(),
            repo: repo_path
        };
    
        let version = selecter.get_version();
        println!("Next Version: {}", version);

        config.update_version(&version);
    }


}


fn get_repo_path(root: &str, repo: &str) -> String {
    let path = Path::new(root).join(repo);
    let path = match path.to_str() {
        Some(v) => v.to_string(),
        None => {
            println!("Can't build path: {}, {}", root, repo);
            exit(1);
        }
    };
    println!("Repo Path: {}", path);
    path
}



