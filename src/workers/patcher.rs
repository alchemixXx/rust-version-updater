use crate::config::RepoType;
use std::fs::{read_to_string, write};
use std::path::Path;
use std::process::{Command, Output};

pub struct Patcher {
    pub next_version: String,
    pub current_version: String,
    pub path: String,
    pub repo_type: RepoType,
    pub branch: String,
    pub release_branch: String,
}

impl Patcher {
    pub fn update_version_in_repo(&self) -> Output{
        println!("Updating version in repo: {}", self.path);
        let result = match self.repo_type {
            RepoType::Node => self.patch_node(),
            RepoType::Python => self.patch_python()
        };
        println!("Updated version in repo: {}. New version is: {}", self.path, self.next_version);

        return result;
    }

    fn patch_node(&self) -> Output {
        println!("Patching node repo: {}", self.path);

        self.up_node_version();
        self.add_changes();
        self.commit_changes();
        self.add_tags();
        self.push_to_origin();
        self.push_to_tags();
        let pr_link: Output = self.create_pr();


        println!("Patched node repo: {}. PR: {:?}", self.path, pr_link);

        return pr_link
    }

    fn patch_python(&self) -> Output {
        println!("Patching python repo: {}", self.path);

        self.up_python_version();
        self.add_changes();
        self.commit_changes();
        self.add_tags();
        self.push_to_origin();
        self.push_to_tags();
        let pr_link: Output = self.create_pr();

        println!("Patched python repo: {}. PR: {:?}", self.path, pr_link);

        return pr_link;
    }

    fn up_node_version(&self){
        println!("Updating version in package.json: {}", self.path);
        self.up_version_by_replacement("package.json", 1);
        println!("Updated version in package.json: {}", self.path);

        println!("Updating version in package-lock.json: {}", self.path);
        self.up_version_by_replacement("package-lock.json", 2);
        println!("Updating version in package-lock.json: {}", self.path);

        println!("Patched node repo by replacement: {}", self.path);
    }

    fn up_python_version(&self){
        println!("Updating version in package.json: {}", self.path);
        self.up_version_by_replacement("package.json", 1);
        println!("Updated version in package.json: {}", self.path);

        println!("Updating version in version.json: {}", self.path);
        self.up_version_by_replacement("version.json", 1);
        println!("Updating version in version.json: {}", self.path);

        println!("Patched node repo by replacement: {}", self.path);
    }


    fn up_version_by_replacement(&self, file: &str, replacement_number: usize){
        let path = Path::new(&self.path).join(file).to_str().expect("Can't find the target file").to_string();
        let mut package_json_content = read_to_string(&path).expect("Failed to read files");
        
        package_json_content = package_json_content.replacen(format!("\"version\": \"{}\"", &self.current_version).as_str(), format!("\"version\": \"{}\"", &self.next_version).as_str(), replacement_number);
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

    fn create_pr(&self) -> Output {
        println!("Creating PR in AWS: {}", self.path);
        let output = Command::new("drunx")
            .arg("aws")
            .arg("pr")
            .arg(&self.branch)
            .arg(&self.release_branch)
            .arg(format!("'CDA Artifact {}'", self.next_version))
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to create PR for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to create PR for repo");
        }
        println!("Created PR in AWS: {}", self.path);

        return output;
    }
}