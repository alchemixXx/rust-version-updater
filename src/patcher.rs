pub struct Patcher {
    version: String,
    path: String
}

impl Patcher {
    pub fn update_version_in_repo(&self){
        println!("Updating version in repo: {}", self.repo);
        println!("Updated version in repo: {}. New version is: ", self.repo, self.version);
    }
}