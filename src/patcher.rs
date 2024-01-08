use crate::config::RepoType;
use std::fs::{read_to_string, write};
use std::path::Path;
use std::process::{Command, exit, Output};

pub struct Patcher {
    pub next_version: String,
    pub current_version: String,
    pub path: String,
    pub repo_type: RepoType,
    pub branch: String,
    pub release_branch: String,
}

impl Patcher {
    pub fn update_version_in_repo(&self){
        println!("Updating version in repo: {}", self.path);
        match self.repo_type {
            RepoType::Node => {
                self.patch_node();
            },
            RepoType::Python => {
                self.patch_python();
            }
        }
        println!("Updated version in repo: {}. New version is: {}", self.path, self.next_version);
    }

    fn patch_node(&self) {
        println!("Patching node repo: {}", self.path);
        let last: char = self.current_version.chars().last().unwrap();
        
        if last.is_numeric(){
            println!("Patching node repo by npm: {}", self.path);
            self.up_npm_version();
            println!("Patched node repo by npm: {}", self.path);
        } else {
            println!("Patching node repo by replacement: {}", self.path);
            self.up_version_by_replacement(vec!["package.json".to_string(), "package-lock.json".to_string()]);
            println!("Patched node repo by replacement: {}", self.path);
        }


        self.add_changes();
        self.commit_changes();
        self.add_tags();
        self.push_to_origin();
        self.push_to_tags();
        self.create_pr();


        println!("Patched node repo: {}", self.path);
    }

    fn patch_python(&self) {
        println!("Patching python repo: {}", self.path);

        println!("Patching python repo by replacement: {}", self.path);
        self.up_version_by_replacement(vec!["version.json".to_string()]);
        println!("Patched python repo by replacement: {}", self.path);

        self.add_changes();
        self.commit_changes();
        self.add_tags();
        self.push_to_origin();
        self.push_to_tags();
        self.create_pr();

        println!("Patched python repo: {}", self.path);
    }

    fn up_npm_version(&self) {
        println!("Increasing version by npm: {}", self.path);
        let output = Command::new("npm")
            .arg("version")
            .arg(&self.next_version)
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute npm command");
        if !output.status.success() {
            eprintln!("Failed increase version by npm in repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
    }

    fn up_version_by_replacement(&self, files: Vec<String>){
        for file in files.iter() {
            let path = Path::new(&self.path).join(file).to_str().expect("Can't find the target file").to_string();
            let mut package_json_content = read_to_string(&path).expect("Failed to read files");
            
            package_json_content = package_json_content.replace(format!("\"version\": \"{}\"", &self.current_version).as_str(), format!("\"version\": \"{}\"", &self.next_version).as_str());
            write(&path, package_json_content).expect("Failed to write files");
        }
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
            eprintln!("Failed to hard reset branch for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Added changes to git: {}", self.path);
    }

    fn commit_changes(&self) {
        println!("Committing changes to git: {}", self.path);
        let output = Command::new("git")
            .arg("commit")
            .arg("-am")
            .arg(format!("'@{} release'", self.next_version))
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to hard reset branch for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Committing changes to git: {}", self.path);
    }

    fn add_tags(&self) {
        println!("Adding tags to git: {}", self.path);
        let output = Command::new("git")
            .arg("commit")
            .arg("-a")
            .arg(format!("'@{} release'", self.next_version))
            .arg("-m")
            .arg(format!("'@{} release version'", self.next_version))
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to hard reset branch for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Adding tags to git: {}", self.path);
    }

    fn push_to_origin(&self) {
        println!("Pushing changes to git origin: {}", self.path);
        let output = Command::new("git")
            .arg("push")
            .arg("-f")
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to hard reset branch for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
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
            eprintln!("Failed to hard reset branch for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Pushed tags to git origin: {}", self.path);
    }

    fn create_pr(&self) -> Output {
        println!("Creating PR in AWS: {}", self.path);
        let output = Command::new("drunx")
            .arg("aws")
            .arg("pr")
            .arg(&self.branch)
            .arg(&self.release_branch)
            .arg(format!("'CDA Artifact {}'", self.next_version))
            .current_dir(&self.branch)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to hard reset branch for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            exit(1);
        }
        println!("Created PR in AWS: {}", self.path);

        return output;
    }
}