use crate::{
    custom_error::{CustomError, CustomResult},
    logger::Logger,
};
use std::process::Command;
#[derive(Debug)]
enum TargetConfig {
    JFrog(String),
    CodeBuild(String),
}

pub fn login(branch: &str, script_path: &str, role: &str) -> CustomResult<()> {
    let logger = Logger::new();
    logger.debug("Logging in");
    let target_config = get_target_config(branch)?;
    login_to_aws(target_config)?;
    switch_role(script_path, role)?;
    logger.debug("Logged in");

    Ok(())
}

fn login_to_aws(target_config: TargetConfig) -> CustomResult<()> {
    let logger = Logger::new();
    match target_config {
        TargetConfig::JFrog(profile) => {
            use_target_config(&profile)?;
            generate_npm_token()?;
        }

        TargetConfig::CodeBuild(profile) => {
            logger.debug("Logging in to CodeBuild");
            use_target_config(&profile)?;
            generate_aws_tokens()?;
            generate_npm_token()?;
        }
    }

    Ok(())
}

fn use_target_config(target_config: &String) -> CustomResult<()> {
    let logger = Logger::new();
    logger.debug(format!("Using target config: {:?}", target_config).as_str());
    let output = Command::new("npmrc")
        .arg(target_config)
        .output()
        .map_err(|err| CustomError::NpmConfigError(err.to_string()))?;
    if !output.status.success() {
        logger.error("Failed to use target config");
        logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

        return Err(CustomError::NpmConfigError(
            "Failed to use target config".to_string(),
        ));
    }
    logger.info("Used target config");

    Ok(())
}

fn get_target_config(branch: &str) -> CustomResult<TargetConfig> {
    if branch == "next" {
        Ok(TargetConfig::JFrog("jfrog".to_string()))
    } else {
        Ok(TargetConfig::CodeBuild("codebuild".to_string()))
    }
}

pub fn generate_aws_tokens() -> CustomResult<()> {
    let logger = Logger::new();
    let output = Command::new("aws")
        .arg("sso")
        .arg("login")
        .arg("--sso-session")
        .arg("sso")
        .output()
        .map_err(|err| CustomError::NpmConfigError(err.to_string()))?;
    if !output.status.success() {
        logger.error("Failed to login to AWS");
        logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

        return Err(CustomError::CommandExecution(
            "Failed to login to AWS".to_string(),
        ));
    }

    Ok(())
}

fn generate_npm_token() -> CustomResult<()> {
    let logger = Logger::new();
    let output = Command::new("aws")
        .arg("codeartifact")
        .arg("login")
        .arg("--tool")
        .arg("npm")
        .arg("--domain")
        .arg("conform")
        .arg("--domain-owner")
        .arg("022587608743")
        .arg("--profile")
        .arg("conform5-code-artifacts-read-role")
        .arg("--region")
        .arg("us-east-1")
        .arg("--repository")
        .arg("conform5-npm-common")
        .arg("--profile")
        .arg("conform5-edetek-dev-01.conform5-batch-dev")
        .output()
        .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
    if !output.status.success() {
        logger.error("Failed to generate token to codeartifact");
        logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

        return Err(CustomError::CommandExecution(
            "Failed to generate token to codeartifact".to_string(),
        ));
    }

    Ok(())
}

pub fn switch_role(script_path: &str, role: &str) -> CustomResult<()> {
    let logger = Logger::new();
    logger.debug("Switching role");
    let command_string = get_switch_role_command(script_path, role);
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&command_string)
        .output()
        .map_err(|err| CustomError::CommandExecution(err.to_string()))?;
    if !output.status.success() {
        logger.error("Failed to switch the aws role");
        logger.error(format!("Error: {}", String::from_utf8_lossy(&output.stderr)).as_str());

        return Err(CustomError::CommandExecution(
            "Failed to switch the aws role".to_string(),
        ));
    }
    logger.debug("Switched role");
    Ok(())
}

pub fn get_switch_role_command(script_path: &str, role: &str) -> String {
    format!("source {0} -profile {1}", script_path, role)
}
