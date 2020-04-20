use std::error;

use serde::Deserialize;

/// User representation of the GitHub API.
/// Note that the GitHub API has several more fields, not all of them are interesting to us.
/// Have a look at the GitHub API Documentation for details: https://developer.github.com/v3/users/
#[derive(Deserialize, Debug)]
pub struct User {
    login: String,
    id: u32,
    url: String,
}

/// Get's all the GitHub users who have starred the RustCred repository
/// GitHub Users who have starred the repository is considered to be participants in RustCred.
/// It is implemented by blocking, so it will block the outgoing web-request to GitHub and will
/// thus take a while to execute.
pub fn get_participants() -> Result<Vec<User>, Box<dyn error::Error>> {
    Ok(reqwest::blocking::Client::new()
        .get("https://api.github.com/repos/saidaspen/rustcred/stargazers")
        // User-Agent is mandated by the GitHub API, if not supplied, request will be rejected.
        .header("User-Agent", "RustCred App")
        .send()?
        .json::<Vec<User>>()?)
}

/// Gets the number of merged pull-requests a certain author has for a certain GitHub Repository
pub fn get_num_merged_prs(repo: &str, author: &str) -> u32 {
    unimplemented!();
}
