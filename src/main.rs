use std::collections::HashMap;
use clap::Parser;
use std::path::Path;
mod config;
mod workers;
mod cli;
use cli::CLi;
use workers::version::VersionSelecter;
use workers::branch::BranchSwitcher;
use workers::rebuilder::RepoRebuilder;
use workers::loginer::login;
use workers::patcher::Patcher;
use workers::history::HistoryProvider;

fn main() {
    println!("Hello, from batch version updater on rust!");
    println!("Starting version updater...");

    println!("Reading cli args...");
    let cli_args = CLi::parse();
    println!("CLI args: {:#?}", cli_args);

    let config = config::read_config(&cli_args.path);
    let repos = config.repos.get_repos_list();
    println!("Repos to update: {:#?}", repos);

    let mut result_string = String::new();
    result_string.push_str("\n");

    println!("Logging in to AWS...");
    login(config.git.branch.clone(), &config.aws.role_script_path, &config.aws.role);
    println!("Logged in to AWS");

    let mut results_hash: HashMap<&String, String> = HashMap::new();

    for repo in repos.iter() {
        println!("Getting repo type for repo: {}", repo);
        let repo_type = config.repos.get_repo_type(repo);
        println!("Got repo type for repo: {}.Type={:?}", repo, repo_type);

        let repo_path = Path::new(&config.root)
            .join(&repo)
            .to_str()
            .expect("Cant't build path")
            .to_string();
        let switcher = BranchSwitcher { target_branch: config.git.branch.to_string() };

        println!("Checking out to target branch for repo: {}", repo_path);
        switcher.checkout_target_branch(&repo_path);
        println!("Checked out to target branch for repo: {}", repo_path);

        if config.repo_rebuild_required {
            println!("Rebuilding repo: {}", repo_path);
            let rebuilder = RepoRebuilder {
                repo: repo_path.clone(),
                repo_type: repo_type,
            };
            rebuilder.rebuild_repo();
            println!("Rebuilt repo: {}", repo_path);
        } else {
            println!("Dry run mode. Skipping repo rebuild: {}", repo_path);
        }

        let selecter = VersionSelecter {
            expected_version: config.git.version.clone(),
            repo: repo_path.clone(),
        };

        let (current_version, next_version) = selecter.get_version();
        println!("Versions: current={}, next={}", current_version, next_version);

        let history_provider = HistoryProvider { path: repo_path.clone() };

        println!("Collecting repo history: {}", repo_path);
        let history = history_provider.provide();
        println!("Collected repo history: {}. Results: {:?}", repo_path, history);

        result_string.push_str(&format!("{}\nrelease/{}\n{}\n", repo, next_version, history));

        if !config.version_update_required {
            println!("Dry run mode. Skipping version update in repo: {}", repo_path);
            continue;
        }

        println!("Updating version in repo: {}", repo_path);
        let patcher = Patcher {
            next_version,
            current_version,
            path: repo_path.clone(),
            repo_type: config.repos.get_repo_type(repo),
            branch: config.git.branch.clone(),
            release_branch: config.git.release_branch.clone(),
            repo_name: repo.clone(),
            role: config.aws.role.clone(),
            sso_script_path: config.aws.role_script_path.clone(),
        };

        let result = patcher.update_version_in_repo();

        results_hash.insert(repo, result);
        println!("Updated version in repo: {}", repo_path);
    }

    println!("Repos history logs:\n{}", result_string);
    println!("Repos PRs: {:#?}", results_hash);
    println!("Version updater finished!");
}
