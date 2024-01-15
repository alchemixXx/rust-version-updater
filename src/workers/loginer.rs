use std::process::Command;

# [derive(Debug)]
enum TargetConfig {
    JFrog(String),
    CodeBuild(String),
}


pub fn login(branch: String) {
    println!("Logging in");
    let target_config = get_target_config(&branch);
    login_to_aws(target_config);
    println!("Logged in");
}

fn login_to_aws(target_config: TargetConfig){
    match target_config {
        TargetConfig::JFrog(profile) => {
            use_target_config(&profile);
        },
            
        TargetConfig::CodeBuild(profile) => {
            println!("Logging in to CodeBuild");
            use_target_config(&profile);
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
                .output()
                .expect("Failed to execute git command");
            if !output.status.success() {
                eprintln!("Failed to login to AWS");
                eprintln!("Error: {}", String::from_utf8_lossy(&output.stderr));
                panic!("Failed to login to AWS");
            }
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

fn get_target_config(branch: &String) -> TargetConfig {

    if branch == "next" {
        return TargetConfig::JFrog("jfrog".to_string());
    }

    if branch == "dev-59" {
        return TargetConfig::CodeBuild("codebuild".to_string());
    }

    eprintln!("Unknown branch: {}", branch);
    panic!("Unknown branch");
}
