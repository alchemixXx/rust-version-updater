use std::process::Command;
#[derive(Debug)]
enum TargetConfig {
    JFrog(String),
    CodeBuild(String),
}

pub fn login(branch: &str, script_path: &str, role: &str) {
    println!("Logging in");
    let target_config = get_target_config(branch);
    login_to_aws(target_config);
    switch_role(script_path, role);
    println!("Logged in");
}

fn login_to_aws(target_config: TargetConfig) {
    match target_config {
        TargetConfig::JFrog(profile) => {
            use_target_config(&profile);
            generate_npm_token();
        }

        TargetConfig::CodeBuild(profile) => {
            println!("Logging in to CodeBuild");
            use_target_config(&profile);
            generate_aws_tokens();
            generate_npm_token();
        }
    }
}

fn use_target_config(target_config: &String) {
    println!("Using target config: {:?}", target_config);
    let output = Command::new("npmrc")
        .arg(target_config)
        .output()
        .expect("Failed to execute git command");
    if !output.status.success() {
        eprintln!("Failed to use target config");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to use target config");
    }
    println!("Used target config");
}

fn get_target_config(branch: &str) -> TargetConfig {
    if branch == "next" {
        return TargetConfig::JFrog("jfrog".to_string());
    }

    if branch == "dev-59" {
        return TargetConfig::CodeBuild("codebuild".to_string());
    }

    eprintln!("Unknown branch: {}", branch);
    panic!("Unknown branch");
}

pub fn generate_aws_tokens() {
    let output = Command::new("aws-sso-util")
        .arg("login")
        .output()
        .expect("Failed to execute aws login command");
    if !output.status.success() {
        eprintln!("Failed to login to AWS");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to login to AWS");
    }
}

fn generate_npm_token() {
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
        .expect("Failed to execute git command");
    if !output.status.success() {
        eprintln!("Failed to generate token to codeartifact");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to generate token to codeartifact");
    }
}

pub fn switch_role(script_path: &str, role: &str) {
    println!("Switching role");
    let command_string = get_switch_role_command(script_path, role);
    let output = Command::new("zsh")
        .arg("-c")
        .arg(&command_string)
        .output()
        .expect("Failed to switch the aws role");
    if !output.status.success() {
        eprintln!("Failed to switch the aws role");
        eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
        panic!("Failed to switch the aws role");
    }
    println!("Switched role");
}

pub fn get_switch_role_command(script_path: &str, role: &str) -> String {
    return format!("source {0} -profile {1}", script_path, role);
}
