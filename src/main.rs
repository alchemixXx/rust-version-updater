use clap::Parser;
use std::collections::HashMap;
use std::path::Path;
mod cli;
mod config;
mod custom_error;
mod logger;
mod workers;
use cli::CLi;
use custom_error::CustomResult;
use workers::branch::BranchSwitcher;
use workers::history::HistoryProvider;
use workers::loginer::login;
use workers::patcher::Patcher;
use workers::rebuilder::RepoRebuilder;
use workers::version::VersionSelecter;

fn main() -> CustomResult<()> {
    println!("Reading cli args...");
    let cli_args = CLi::parse();
    println!("CLI args: {:#?}", cli_args);

    let config = config::read_config(&cli_args.path)?;

    crate::logger::Logger::init(config.logger.log_level);
    let logger = crate::logger::Logger::new();
    logger.info("Version updater started!");
    let repos = config.repos.get_repos_list()?;
    logger.info(format!("Repos to update: {:#?}", repos).as_str());

    let mut result_string = String::new();
    result_string.push('\n');

    logger.debug("Logging in to AWS...");
    login(
        &config.git.branch,
        &config.aws.role_script_path,
        &config.aws.role,
    )?;
    logger.debug("Logged in to AWS");

    let mut results_hash: HashMap<&String, String> = HashMap::new();
    let mut errors_hash: HashMap<&String, String> = HashMap::new();

    for repo in repos.iter() {
        logger.debug(format!("Getting repo type for repo: {}", repo).as_str());
        let repo_type = match config.repos.get_repo_type(repo) {
            Ok(repo_type) => repo_type,
            Err(e) => {
                errors_hash.insert(repo, e.to_string());
                continue;
            }
        };
        logger.debug(format!("Got repo type for repo: {}.Type={:?}", repo, repo_type).as_str());

        let repo_path = match Path::new(&config.root).join(repo).to_str() {
            Some(path) => path.to_string(),
            None => {
                let error = format!("Failed to get repo path for repo: {}", repo);
                errors_hash.insert(repo, error);
                continue;
            }
        };
        let switcher = BranchSwitcher {
            target_branch: &config.git.branch,
        };

        logger.debug(format!("Checking out to target branch for repo: {}", repo_path).as_str());
        match switcher.checkout_target_branch(&repo_path) {
            Ok(_) => {}
            Err(e) => {
                errors_hash.insert(repo, e.to_string());
                continue;
            }
        };
        logger.debug(format!("Checked out to target branch for repo: {}", repo_path).as_str());

        let history_provider = HistoryProvider { path: &repo_path };
        logger.debug(format!("Collecting repo history: {}", repo_path).as_str());
        let history = match history_provider.provide() {
            Ok(history) => history,
            Err(e) => {
                errors_hash.insert(repo, e.to_string());
                continue;
            }
        };
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
            match rebuilder.rebuild_repo() {
                Ok(_) => {}
                Err(e) => {
                    errors_hash.insert(repo, e.to_string());
                    continue;
                }
            };
            logger.debug(format!("Rebuilt repo: {}", repo_path).as_str());
        } else {
            logger.warn(format!("Dry run mode. Skipping repo rebuild: {}", repo_path).as_str());
        }

        let selecter = VersionSelecter {
            expected_version: &config.git.version,
            repo: &repo_path,
        };

        let (current_version, next_version) = match selecter.get_version() {
            Ok(versions) => versions,
            Err(e) => {
                errors_hash.insert(repo, e.to_string());
                continue;
            }
        };
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
            repo_type,
            branch: &config.git.branch,
            release_branch: &config.git.release_branch,
            repo_name: repo,
            role: &config.aws.role,
            sso_script_path: &config.aws.role_script_path,
            disable_checks: config.disable_checks,
        };

        let result = match patcher.update_version_in_repo() {
            Ok(result) => result,
            Err(e) => {
                errors_hash.insert(repo, e.to_string());
                continue;
            }
        };
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
    logger.warn(format!("Errors: {:#?}", errors_hash).as_str());

    logger.info("Version updater finished!");

    Ok(())
}
