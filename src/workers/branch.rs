use std::process::Command;

use crate::{
    custom_error::{CustomError, CustomResult},
    logger::LoggerTrait,
};

pub struct BranchSwitcher<'branch> {
    pub target_branch: &'branch String,
}

impl<'config> LoggerTrait for BranchSwitcher<'config> {}
impl<'branch> BranchSwitcher<'branch> {
    pub fn checkout_target_branch(&self, repo_path: &str) -> CustomResult<()> {
        self.stash_repo(repo_path)?;
        self.fetch_repo(repo_path)?;
        self.switch_branch(repo_path)?;
        self.hard_reset_branch(repo_path)?;

        Ok(())
    }

    fn hard_reset_branch(&self, repo_path: &str) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Hard resetting branch for repo: {}", repo_path).as_str());
        let output = Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(format!("origin/{}", self.target_branch))
            .current_dir(repo_path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to hard reset branch for repo: {}", repo_path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to hard reset branch for repo".to_string(),
            ));
        }

        Ok(())
    }

    fn stash_repo(&self, repo_path: &str) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Stashing repo: {}", repo_path).as_str());
        let output = Command::new("git")
            .arg("stash")
            .current_dir(repo_path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to stash repo: {}", repo_path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to stash repo".to_string(),
            ));
        }

        Ok(())
    }

    fn fetch_repo(&self, repo_path: &str) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Fetching repo: {}", repo_path).as_str());
        let output = Command::new("git")
            .arg("fetch")
            .current_dir(repo_path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to fetch repo: {}", repo_path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to fetch repo".to_string(),
            ));
        }

        Ok(())
    }

    fn switch_branch(&self, repo_path: &str) -> CustomResult<()> {
        let logger = self.get_logger();
        logger.info(format!("Switching branch for repo: {}", repo_path).as_str());
        let output = Command::new("git")
            .arg("checkout")
            .arg(self.target_branch)
            .current_dir(repo_path)
            .output()
            .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
        if !output.status.success() {
            logger.error(format!("Failed to switch branch for repo: {}", repo_path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

            return Err(CustomError::CommandExecution(
                "Failed to switch branch for repo".to_string(),
            ));
        }

        Ok(())
    }
}
