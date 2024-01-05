use std::process::{Command, exit};

use crate::config::RepoType;

pub struct RepoRebuilder{
    pub repo: String,
    pub repo_type: RepoType,
}

impl RepoRebuilder {
    pub fn rebuild_repo(&self) {
        println!("Rebuilding repo: {}", self.repo);
        match self.repo_type {
            RepoType::Node => {
                self.rebuild_node_repo();
            },
            RepoType::Python => {
                println!("Nothing to rebuild in repo='{}'", self.repo);
            }
        }
    }

    fn rebuild_node_repo(&self){
        self.delete_folders();
        self.install_npm_packages();
        self.build_node_repo();
    }

    fn delete_folders(&self){
        println!("Deleting node_modules and dis folders");
        let output = Command::new("rm")
            .arg("-rf")
            .current_dir(&self.repo)
            .output()
            .expect("Failed to execute command");
        if !output.status.success() {
            eprintln!("Failed to delete folders in repo: {}", self.repo);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Deleted node_modules and dis folders");
    }

    fn install_npm_packages(&self){
        println!("Installing packages");
        let output = Command::new("npm")
            .arg("install")
            .current_dir(&self.repo)
            .output()
            .expect("Failed to execute command");
        if !output.status.success() {
            eprintln!("Failed to install packages in repo: {}", self.repo);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Installed packages");
    }

    fn build_node_repo(&self){
        println!("Building node repo");
        let output = Command::new("npm")
            .arg("run")
            .arg("build")
            .current_dir(&self.repo)
            .output()
            .expect("Failed to execute command");
        if !output.status.success() {
            eprintln!("Failed to build node repo: {}", self.repo);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Built node repo");
    }
}