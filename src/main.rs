use clap::Parser;
use std::collections::HashMap;
use std::path::Path;
mod cli;
mod config;
mod logger;
mod workers;
use cli::CLi;
use workers::branch::BranchSwitcher;
use workers::history::HistoryProvider;
use workers::loginer::login;
use workers::patcher::Patcher;
use workers::rebuilder::RepoRebuilder;
use workers::version::VersionSelecter;

fn main() {
    println!("Reading cli args...");
    let cli_args = CLi::parse();
    println!("CLI args: {:#?}", cli_args);

    let config = config::read_config(&cli_args.path);

    crate::logger::Logger::init(config.logger.log_level);
    let logger = crate::logger::Logger::new();
    logger.info("Version updater started!");
    let repos = config.repos.get_repos_list();
    logger.info(format!("Repos to update: {:#?}", repos).as_str());

    let mut result_string = String::new();
    result_string.push('\n');

    logger.debug("Logging in to AWS...");
    login(
        &config.git.branch,
        &config.aws.role_script_path,
        &config.aws.role,
    );
    logger.debug("Logged in to AWS");

    let mut results_hash: HashMap<&String, String> = HashMap::new();

    for repo in repos.iter() {
        logger.debug(format!("Getting repo type for repo: {}", repo).as_str());
        let repo_type = config.repos.get_repo_type(repo);
        logger.debug(format!("Got repo type for repo: {}.Type={:?}", repo, repo_type).as_str());

        let repo_path = Path::new(&config.root)
            .join(repo)
            .to_str()
            .expect("Cant't build path")
            .to_string();
        let switcher = BranchSwitcher {
            target_branch: &config.git.branch,
        };

        logger.debug(format!("Checking out to target branch for repo: {}", repo_path).as_str());
        switcher.checkout_target_branch(&repo_path);
        logger.debug(format!("Checked out to target branch for repo: {}", repo_path).as_str());

        let history_provider = HistoryProvider { path: &repo_path };
        logger.debug(format!("Collecting repo history: {}", repo_path).as_str());
        let history = history_provider.provide();
        logger.debug(
            format!(
                "Collected repo history: {}. Results: {:?}",
                repo_path, history
            )
            .as_str(),
        );

        if config.process_only_updated_repo && history.is_empty() {
            logger.warn(format!("Skipping repo: {}. No history found\n\n", repo_path).as_str());
            continue;
        }

        if config.repo_rebuild_required {
            logger.debug(format!("Rebuilding repo: {}", repo_path).as_str());
            let rebuilder = RepoRebuilder {
                repo: &repo_path,
                repo_type,
            };
            rebuilder.rebuild_repo();
            logger.debug(format!("Rebuilt repo: {}", repo_path).as_str());
        } else {
            logger.warn(format!("Dry run mode. Skipping repo rebuild: {}", repo_path).as_str());
        }

        let selecter = VersionSelecter {
            expected_version: &config.git.version,
            repo: &repo_path,
        };

        let (current_version, next_version) = selecter.get_version();
        logger.debug(
            format!(
                "Versions: current={}, next={}",
                current_version, next_version
            )
            .as_str(),
        );

        result_string.push_str(&format!(
            "{}\nrelease/{}\n{}\n",
            repo, next_version, history
        ));
        logger.warn(format!("\n\n{}\nrelease/{}\n{}", repo, next_version, history).as_str());

        if !config.version_update_required {
            logger.debug(
                format!(
                    "Dry run mode. Skipping version update in repo: {}",
                    repo_path
                )
                .as_str(),
            );
            continue;
        }

        logger.debug(format!("Updating version in repo: {}", repo_path).as_str());
        let patcher = Patcher {
            next_version,
            current_version,
            path: &repo_path,
            repo_type: config.repos.get_repo_type(repo),
            branch: &config.git.branch,
            release_branch: &config.git.release_branch,
            repo_name: repo,
            role: &config.aws.role,
            sso_script_path: &config.aws.role_script_path,
        };

        let result = patcher.update_version_in_repo();
        logger.warn(format!("{}\n\n", result).as_str());

        results_hash.insert(repo, result);
        logger.debug(format!("Updated version in repo: {}", repo_path).as_str());
    }

    logger.warn(
        format!(
            "Repos history logs:\n{}\nRepos PRs: {:#?}",
            result_string, results_hash
        )
        .as_str(),
    );
    logger.info("Version updater finished!");
}
