use std::collections::HashMap;
use std::path::Path;
use std::process::Output;
mod version;
mod config;
mod branch;
mod rebuilder;
mod loginer;
mod patcher;
mod history;


fn main() {
    println!("Hello, from batch version updater on rust!");
    println!("Starting version updater...");
    let config = config::read_config();

    let repos = config.repos.get_repos_list();
    println!("Repos to update: {:#?}", repos);

    let mut result_string = String::new();
    result_string.push_str("\n");

    println!("Logging in to AWS...");
    loginer::login(config.git.branch.clone());
    println!("Logged in to AWS");

    let mut results_hash: HashMap<&String, String> = HashMap::new();

    for repo in repos.iter() {
        
        println!("Getting repo type for repo: {}", repo);
        let repo_type = config.repos.get_repo_type(repo);
        println!("Got repo type for repo: {}.Type={:?}", repo, repo_type);
        
        let repo_path = Path::new(&config.root).join(&repo).to_str().expect("Cant't build path").to_string();
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
            repo: repo_path.clone()
        };
    
        let (current_version, next_version) = selecter.get_version();
        println!("Versions: current={}, next={}", current_version, next_version);

        let history_provider = history::HistoryProvider{path:repo_path.clone()};

        println!("Collecting repo history: {}", repo_path);
        let history = history_provider.provide();
        println!("Collected repo history: {}. Results: {:?}", repo_path, history);

        result_string.push_str(&format!("{}\n release/{}\n {}\n", repo, next_version, history));

        println!("Updating version in repo: {}", repo_path);
        let patcher = patcher::Patcher{next_version, current_version, path:repo_path.clone(), repo_type:config.repos.get_repo_type(repo), branch:config.git.branch.clone(), release_branch:config.git.release_branch.clone()};

        let result = patcher.update_version_in_repo();
        let std_out = String::from_utf8(result.stdout.clone()).expect("Cant't parse stdout");
        let replaced_stdout = std_out.replace("\n", "\n\t");

        results_hash.insert(repo, replaced_stdout);
    }

    println!("Repos history logs: {}", result_string);
    println!("Repos PRs: {:#?}", results_hash);
    println!("Version updater finished!");
}





