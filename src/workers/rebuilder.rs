use std::process::Command;

use crate::{ config::RepoType, logger::LoggerTrait };

pub struct RepoRebuilder<'repo> {
    pub repo: &'repo String,
    pub repo_type: RepoType,
}

impl<'config> LoggerTrait for RepoRebuilder<'config> {}
impl<'repo> RepoRebuilder<'repo> {
    pub fn rebuild_repo(&self) {
        let logger = self.get_logger();
        logger.debug(format!("Rebuilding repo: {}", self.repo).as_str());
        match self.repo_type {
            RepoType::Node => {
                self.rebuild_node_repo();
            }
            RepoType::Python => {
                logger.debug(format!("Nothing to rebuild in repo='{}'", self.repo).as_str());
            }
        }
    }

    fn rebuild_node_repo(&self) {
        self.delete_folders();
        self.install_npm_packages();
        self.build_node_repo();
    }

    fn delete_folders(&self) {
        let logger = self.get_logger();
        logger.debug("Deleting node_modules and dist folders");
        let output = Command::new("rm")
            .arg("-rf")
            .current_dir(self.repo)
            .output()
            .expect("Failed to execute command");
        if !output.status.success() {
            logger.error(format!("Failed to delete folders in repo: {}", self.repo).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());
            panic!("Failed to delete folders in repo");
        }
        logger.debug("Deleted node_modules and dis folders");
    }

    fn install_npm_packages(&self) {
        let logger = self.get_logger();
        logger.debug("Installing packages");
        let output = Command::new("npm")
            .arg("install")
            .current_dir(self.repo)
            .output()
            .expect("Failed to execute command");
        if !output.status.success() {
            logger.error(format!("Failed to install packages in repo: {}", self.repo).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());
            panic!("Failed to install packages in repo");
        }
        logger.debug("Installed packages");
    }

    fn build_node_repo(&self) {
        let logger = self.get_logger();
        logger.debug("Building node repo");
        let output = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(self.repo)
            .output()
            .expect("Failed to execute command");
        if !output.status.success() {
            logger.error(format!("Failed to build node repo: {}", self.repo).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());
            panic!("Failed to build node repo");
        }
        logger.debug("Built node repo");
    }
}
