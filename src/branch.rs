use std::process::Command;

pub struct BranchSwitcher {
    pub target_branch: String,
}

impl BranchSwitcher {
    pub fn checkout_target_branch(&self, repo_path: &str) {
        self.stash_repo(repo_path);
        self.fetch_repo(repo_path);
        self.switch_branch(repo_path);
        self.hard_reset_branch(repo_path);

    }

    fn hard_reset_branch(&self, repo_path: &str) {
        println!("Hard resetting branch for repo: {}", repo_path);
        let output = Command::new("git")
            .arg("reset")
            .arg("--hard")
            .arg(format!("origin/{}", self.target_branch))
            .current_dir(repo_path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to hard reset branch for repo: {}", repo_path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to hard reset branch for repo");
        }
    }

    fn stash_repo(&self, repo_path: &str) {
        println!("Stashing repo: {}", repo_path);
        let output = Command::new("git")
            .arg("stash")
            .current_dir(repo_path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to stash repo: {}", repo_path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to stash repo");
        }
    }


    fn fetch_repo(&self, repo_path: &str) {
        println!("Fetching repo: {}", repo_path);
        let output = Command::new("git")
            .arg("fetch")
            .current_dir(repo_path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to fetch repo: {}", repo_path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to fetch repo");
        }
    }

    fn switch_branch(&self, repo_path: &str) {
        println!("Switching branch for repo: {}", repo_path);
        let output = Command::new("git")
            .arg("checkout")
            .arg(&self.target_branch)
            .current_dir(repo_path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to switch branch for repo: {}", repo_path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to switch branch for repo");
        }
    }
}

