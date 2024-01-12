use std::process::Command;
pub struct HistoryProvider{
    pub path: String,
}

impl HistoryProvider {
    pub fn provide(&self) -> String{
        println!("Calculating difference for repo: {}", self.path);
        let history_string = self.get_git_history_as_string();
        println!("Calculated difference for repo: {}", self.path);


        println!("Generating history for repo: {}", self.path);
        let result = self.generate_git_history_string(history_string);
        println!("Generated history for repo: {}", self.path);

        return result;
    }

    fn generate_git_history_string(&self, history: String) -> String{
        let mut target = String::new();
        let parts = history.split("\n");

        for part in parts {
            let trimmed = part.trim().replace("* ", "");
            if trimmed.starts_with("release/") || trimmed.starts_with("@") {
                return target;
            }
            
            target.push_str(&trimmed);
            target.push_str("\n");
        }

        return  target
    }

    fn get_git_history_as_string(&self) -> String {
        println!("Providing history for repo: {}", self.path);
        let output = Command::new("git")
            .arg("log")
            .arg(format!("--pretty=format:%s"))
            .arg("--graph")
            .arg("--abbrev-commit")
            .arg("--decorate")
            .arg("--date=relative")
            .arg("-100")
            .current_dir(&self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            eprintln!("Failed to provide history for repo: {}", self.path);
            eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
            panic!("Failed to provide history for repo");
        }
        println!("Provided history for repo: {}", self.path);

        return String::from_utf8_lossy(&output.stdout).to_string();
    }
}