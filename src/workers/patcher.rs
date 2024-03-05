use serde_derive::{ Serialize, Deserialize };
use crate::config::RepoType;
use crate::workers::loginer::get_switch_role_command;
use std::fs::{ read_to_string, write };
use std::path::Path;
use std::process::{ Command, Output };

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
}

impl<'repo> Patcher<'repo> {
    pub fn update_version_in_repo(&self) -> String {
        println!("Updating version in repo: {}", self.path);

        match self.repo_type {
            RepoType::Node => self.up_node_version(),
            RepoType::Python => self.up_python_version(),
        }

        self.add_changes();
        self.commit_changes();
        self.add_tags();
        self.push_to_origin();
        self.push_to_tags();

        let pr_link = self.create_pr();

        println!(
            "Updated version in repo: {0}. New version is: {1}, PR: {2:?}",
            self.path,
            self.next_version,
            pr_link
        );
        return pr_link;
    }

    fn up_node_version(&self) {
        println!("Updating version in package.json: {}", self.path);
        self.up_version_by_replacement("package.json", 1);
        println!("Updated version in package.json: {}", self.path);

        println!("Updating version in package-lock.json: {}", self.path);
        self.up_version_by_replacement("package-lock.json", 2);
        println!("Updating version in package-lock.json: {}", self.path);

        println!("Patched node repo by replacement: {}", self.path);
    }

    fn up_python_version(&self) {
        println!("Updating version in package.json: {}", self.path);
        self.up_version_by_replacement("package.json", 1);
        println!("Updated version in package.json: {}", self.path);

        println!("Updating version in version.json: {}", self.path);
        self.up_version_by_replacement("version.json", 1);
        println!("Updating version in version.json: {}", self.path);

        println!("Patched node repo by replacement: {}", self.path);
    }

    fn up_version_by_replacement(&self, file: &str, replacement_number: usize) {
        let path = Path::new(&self.path)
            .join(file)
            .to_str()
            .expect("Can't find the target file")
            .to_string();
        let mut package_json_content = read_to_string(&path).expect("Failed to read files");

        package_json_content = package_json_content.replacen(
            format!("\"version\": \"{}\"", &self.current_version).as_str(),
            format!("\"version\": \"{}\"", &self.next_version).as_str(),
            replacement_number
        );
        write(&path, package_json_content).expect("Failed to write files");
    }

    fn add_changes(&self) {
        println!("Adding changes to git: {}", self.path);
        let output = Command::new("git")
            .arg("add")
            .arg("--all")
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed add changes for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed add changes for repo");
        }
        println!("Added changes to git: {}", self.path);
    }

    fn commit_changes(&self) {
        println!("Committing changes to git: {}", self.path);
        let output = Command::new("git")
            .arg("commit")
            .arg("-am")
            .arg(format!("@{} release", self.next_version))
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to commit for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to commit for repo");
        }
        println!("Committed changes to git: {}", self.path);
    }

    fn add_tags(&self) {
        println!("Adding tags to git: {}", self.path);
        let output = Command::new("git")
            .arg("tag")
            .arg("-a")
            .arg(format!("release/{}", self.next_version))
            .arg("-m")
            .arg(format!("release/{} version", self.next_version))
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed add tags for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed add tags for repo");
        }
        println!("Adding tags to git: {}", self.path);
    }

    fn push_to_origin(&self) {
        println!("Pushing changes to git origin: {}", self.path);
        let output = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg(self.branch.as_str())
            .arg("-f")
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to push changes for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to push changes for repo");
        }
        println!("Pushed changes to git origin: {}", self.path);
    }

    fn push_to_tags(&self) {
        println!("Pushing tags to git origin: {}", self.path);
        let output = Command::new("git")
            .arg("push")
            .arg("--tags")
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to push tags for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));

            for line in String::from_utf8_lossy(&output.stderr).lines() {
                if line.contains("[new tag]") {
                    println!("Tag was pushed");
                    return;
                }
            }
            panic!("Failed to push tags for repo");
        }
        println!("Pushed tags to git origin: {}", self.path);
    }

    fn create_pr(&self) -> String {
        println!("Creating PR in AWS: {}", self.path);

        let output = self.execute_pr_create_with_login_command();

        let str_json = String::from_utf8(output.stdout).expect("Failed to parse stdout");
        let commit: Commit = serde_json::from_str(&str_json).expect("Failed to parse json");

        let pr_link = format!(
            "https://console.aws.amazon.com/codesuite/codecommit/repositories/{}/pull-requests/{}/details?region=us-east-1",
            self.repo_name,
            commit.pull_request.pull_request_id
        );
        println!("Created PR in AWS: {}, PR: {}", self.path, pr_link);

        return pr_link;
    }

    fn get_pr_create_command_string(&self) -> String {
        let command = format!(
            "aws codecommit create-pull-request --title {0} --targets repositoryName={1},sourceReference={2},destinationReference={3}",
            format!("'CDA Artifact {}'", self.next_version),
            self.repo_name,
            self.branch,
            self.release_branch
        );

        return command;
    }

    fn execute_pr_create_with_login_command(&self) -> Output {
        let switch_role_command_string = get_switch_role_command(&self.sso_script_path, &self.role);

        println!("Switch role command: {}", switch_role_command_string);
        let aws_pr_create_command_string = self.get_pr_create_command_string();
        let command_string = format!(
            r#"
                {0}
                {1}
            "#,
            switch_role_command_string,
            aws_pr_create_command_string
        );

        let output = Command::new("zsh")
            .arg("-c")
            .arg(&command_string)
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute PR creation command");

        if !output.status.success() {
            eprintln!("Failed to create PR for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to create PR for repo");
        }

        return output;
    }
}
