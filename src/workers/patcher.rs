use crate::config::RepoType;
use crate::custom_error::{CustomError, CustomResult};
use crate::logger::LoggerTrait;
use crate::workers::loginer::get_switch_role_command;
use serde_derive::{Deserialize, Serialize};
use std::fs::{read_to_string, write};
use std::path::Path;
use std::process::{Command, Output};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Commit {
    pull_request: PullRequest,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PullRequest {
    pull_request_id: String,
}

pub struct Patcher<'repo> {
    pub next_version: String,
    pub current_version: String,
    pub repo_name: &'repo String,
    pub path: &'repo String,
    pub repo_type: RepoType,
    pub branch: &'repo String,
    pub release_branch: &'repo String,
    pub role: &'repo String,
    pub sso_script_path: &'repo String,
    pub disable_checks: bool,
}

impl<'config> LoggerTrait for Patcher<'config> {}

impl<'repo> Patcher<'repo> {
    pub fn update_version_in_repo(&self) -> CustomResult<String> {
        let logger = self.get_logger();
        logger.info(format!("Updating version in repo: {}", self.path).as_str());

        match self.repo_type {
            RepoType::Node => self.up_node_version()?,
            RepoType::Python => self.up_python_version()?,
        }

        self.add_changes()?;
        self.commit_changes()?;
        self.add_tags()?;
        self.push_to_origin()?;
        self.push_to_tags()?;

        let pr_link = self.create_pr()?;

        logger.info(
            format!(
                "Updated version in repo: {0}. New version is: {1}, PR: {2:?}",
                self.path, self.next_version, pr_link
            )
            .as_str(),
        );
        Ok(pr_link)
    }

    fn up_node_version(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Updating version in package.json: {}", self.path).as_str());
        self.up_version_by_replacement("package.json", 1)?;
        logger.info(format!("Updated version in package.json: {}", self.path).as_str());

        logger.info(format!("Updating version in package-lock.json: {}", self.path).as_str());
        self.up_version_by_replacement("package-lock.json", 2)?;
        logger.info(format!("Updating version in package-lock.json: {}", self.path).as_str());

        logger.info(format!("Patched node repo by replacement: {}", self.path).as_str());

        Ok(())
    }

    fn up_python_version(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Updating version in package.json: {}", self.path).as_str());
        self.up_version_by_replacement("package.json", 1)?;
        logger.info(format!("Updated version in package.json: {}", self.path).as_str());

        logger.info(format!("Updating version in version.json: {}", self.path).as_str());
        self.up_version_by_replacement("version.json", 1)?;
        logger.info(format!("Updating version in version.json: {}", self.path).as_str());

        logger.info(format!("Patched node repo by replacement: {}", self.path).as_str());

        Ok(())
    }

    fn up_version_by_replacement(&self, file: &str, replacement_number: usize) -> CustomResult<()> {
        let path = match Path::new(&self.path).join(file).to_str() {
            Some(val) => String::from(val),
            None => return Err(CustomError::VersionBuild("Can't build path".to_string())),
        };
        let mut package_json_content = read_to_string(&path).expect("Failed to read files");

        package_json_content = package_json_content.replacen(
            format!("\"version\": \"{}\"", &self.current_version).as_str(),
            format!("\"version\": \"{}\"", &self.next_version).as_str(),
            replacement_number,
        );
        write(&path, package_json_content)
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;

        Ok(())
    }

    fn add_changes(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Adding changes to git: {}", self.path).as_str());
        let output = Command::new("git")
            .arg("add")
            .arg("--all")
            .current_dir(self.path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed add changes for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed add changes for repo".to_string(),
            ));
        }
        logger.info(format!("Added changes to git: {}", self.path).as_str());

        Ok(())
    }

    fn commit_changes(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Committing changes to git: {}", self.path).as_str());
        let mut command = Command::new("git");
        command.arg("commit");

        if self.disable_checks {
            command.arg("--no-verify").arg("-m");
        } else {
            command.arg("-am");
        }

        let output = command
            .arg(format!("@{} release", self.next_version))
            .current_dir(self.path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to commit for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to commit for repo".to_string(),
            ));
        }
        logger.info(format!("Committed changes to git: {}", self.path).as_str());

        Ok(())
    }

    fn add_tags(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Adding tags to git: {}", self.path).as_str());
        let output = Command::new("git")
            .arg("tag")
            .arg("-a")
            .arg(format!("release/{}", self.next_version))
            .arg("-m")
            .arg(format!("release/{} version", self.next_version))
            .current_dir(self.path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed add tags for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed add tags for repo".to_string(),
            ));
        }
        logger.info(format!("Adding tags to git: {}", self.path).as_str());

        Ok(())
    }

    fn push_to_origin(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Pushing changes to git origin: {}", self.path).as_str());
        let mut command = Command::new("git");
        command.arg("push").arg("origin");

        if self.disable_checks {
            command.arg("--no-verify");
        };

        let output = command
            .arg(self.branch.as_str())
            .current_dir(self.path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to push changes for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to push changes for repo".to_string(),
            ));
        }
        logger.info(format!("Pushed changes to git origin: {}", self.path).as_str());

        Ok(())
    }

    fn push_to_tags(&self) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Pushing tags to git origin: {}", self.path).as_str());
        let mut command = Command::new("git");
        command.arg("push").arg("--tags");

        if self.disable_checks {
            command.arg("--no-verify");
        }

        let output = command
            .current_dir(self.path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to push tags for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            for line in String::from_utf8_lossy(&output.stderr).lines() {
                if line.contains("[new tag]") {
                    logger.info("Tag was pushed");
                    return Ok(());
                }
            }

            return Err(CustomError::CommandExecution(
                "Failed to push tags for repo".to_string(),
            ));
        }
        logger.info(format!("Pushed tags to git origin: {}", self.path).as_str());

        Ok(())
    }

    fn create_pr(&self) -> CustomResult<String> {
        let logger = self.get_logger();
        logger.info(format!("Creating PR in AWS: {}", self.path).as_str());

        let output = self.execute_pr_create_with_login_command()?;

        let str_json = String::from_utf8(output.stdout).expect("Failed to parse stdout");
        let commit: Commit = serde_json::from_str(&str_json).expect("Failed to parse json");

        let pr_link = format!(
            "https://console.aws.amazon.com/codesuite/codecommit/repositories/{}/pull-requests/{}/details?region=us-east-1",
            self.repo_name,
            commit.pull_request.pull_request_id
        );
        logger.warn(format!("Created PR in AWS: {}, PR: {}", self.path, pr_link).as_str());

        Ok(pr_link)
    }

    fn get_pr_create_command_string(&self) -> CustomResult<String> {
        let cda_artifact = format!("'CDA Artifact {}'", self.next_version);
        let command = format!(
            "aws codecommit create-pull-request --title {0} --targets repositoryName={1},sourceReference={2},destinationReference={3}",
            cda_artifact,
            self.repo_name,
            self.branch,
            self.release_branch
        );

        Ok(command)
    }

    fn execute_pr_create_with_login_command(&self) -> CustomResult<Output> {
        let logger = self.get_logger();
        let switch_role_command_string = get_switch_role_command(self.sso_script_path, self.role);

        logger.info(format!("Switch role command: {}", switch_role_command_string).as_str());
        let aws_pr_create_command_string = self.get_pr_create_command_string()?;
        let command_string = format!(
            r#"
                {0}
                {1}
            "#,
            switch_role_command_string, aws_pr_create_command_string
        );

        let output = Command::new("zsh")
            .arg("-c")
            .arg(&command_string)
            .current_dir(self.path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;

        if !output.status.success() {
            logger.error(format!("Failed to create PR for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to create PR for repo".to_string(),
            ));
        }

        Ok(output)
    }
}
