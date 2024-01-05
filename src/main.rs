use std::path::Path;
use std::process::exit;
mod version;
mod config;
mod branch;
mod rebuilder;
mod loginer;


fn main() {
    println!("Hello, from batch version updater on rust!");
    println!("Starting version updater...");
    let mut config = config::read_config();

    let repos = config.repos.get_repos_list();
    println!("Repos to update: {:#?}", repos);

    println!("Logging in to AWS...");
    loginer::login(config.git.branch.clone());
    println!("Logged in to AWS...");

    for repo in repos.iter() {
        
        println!("Getting repo type for repo: {}", repo);
        let repo_type = config.repos.get_repo_type(repo);
        println!("Got repo type for repo: {}.Type={:?}", repo, repo_type);
        
        let repo_path = get_repo_path(&config.root, repo);
        let switcher = branch::BranchSwitcher{target_branch:config.git.branch.to_string()};
        
        println!("Checking out to target branch for repo: {}", repo_path);
        switcher.checkout_target_branch(&repo_path);
        println!("Checked out to target branch for repo: {}", repo_path);

        println!("Rebuilding repo: {}", repo_path);
        let rebuilder = rebuilder::RepoRebuilder{repo:repo_path.clone(), repo_type:repo_type};
        rebuilder.rebuild_repo();
        println!("Rebuilt repo: {}", repo_path);



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



