use std::process::Command;

use crate::logger::LoggerTrait;
pub struct HistoryProvider<'repo> {
    pub path: &'repo String,
}

impl<'config> LoggerTrait for HistoryProvider<'config> {}
impl<'repo> HistoryProvider<'repo> {
    pub fn provide(&self) -> String {
        let logger = self.get_logger();
        logger.info(format!("Calculating difference for repo: {}", self.path).as_str());
        let history_string = self.get_git_history_as_string();
        logger.info(format!("Calculated difference for repo: {}", self.path).as_str());

        logger.info(format!("Generating history for repo: {}", self.path).as_str());
        let result = self.generate_git_history_string(history_string);
        logger.info(format!("Generated history for repo: {}", self.path).as_str());

        result
    }

    fn generate_git_history_string(&self, history: String) -> String {
        let mut target = String::new();
        let parts = history.split('\n');

        for part in parts {
            let trimmed = part.trim().replace("* ", "");
            if trimmed.starts_with("release/") || trimmed.starts_with('@') {
                return target;
            }

            target.push_str(&trimmed);
            target.push('\n');
        }

        target
    }

    fn get_git_history_as_string(&self) -> String {
        let logger = self.get_logger();
        logger.info(format!("Providing history for repo: {}", self.path).as_str());
        let output = Command::new("git")
            .arg("log")
            .arg("--pretty=format:%s")
            .arg("--graph")
            .arg("--abbrev-commit")
            .arg("--decorate")
            .arg("--date=relative")
            .arg("-100")
            .current_dir(self.path)
            .output()
            .expect("Failed to execute git command");
        if !output.status.success() {
            logger.error(format!("Failed to provide history for repo: {}", self.path).as_str());
            logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());
            panic!("Failed to provide history for repo");
        }
        logger.info(format!("Provided history for repo: {}", self.path).as_str());

        return String::from_utf8_lossy(&output.stdout).to_string();
    }
}
