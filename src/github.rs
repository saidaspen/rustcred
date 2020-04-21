use std::error;

use serde::Deserialize;

const APP_NAME: &str = "RustCred";

/// User representation of the GitHub API.
/// Note that the GitHub API has several more fields, not all of them are interesting to us.
/// Have a look at the GitHub API Documentation for details: https://developer.github.com/v3/users/
#[derive(Deserialize, Debug, Clone)]
pub struct User {
    pub login: String,
    id: u32,
    url: String,
}

#[derive(Deserialize, Debug)]
pub struct PrSearchResp {
    total_count: u32,
    items: Vec<PrSearchItem>,
}

#[derive(Deserialize, Debug)]
pub struct PrSearchItem {
    url: String,
    repository_url: String,
    id: u32,
    user: User,
    state: String,
    score: f64,
}

/// Get's all the GitHub users who have starred the RustCred repository
/// GitHub Users who have starred the repository is considered to be participants in RustCred.
/// It is implemented by blocking, so it will block the outgoing web-request to GitHub and will
/// thus take a while to execute.
pub fn get_participants() -> Result<Vec<User>, Box<dyn error::Error>> {
    Ok(reqwest::blocking::Client::new()
        .get("https://api.github.com/repos/saidaspen/rustcred/stargazers")
        .header("User-Agent", APP_NAME)
        .send()?
        .json::<Vec<User>>()?)
}

/// Gets the number of merged pull-requests a certain author has for a certain GitHub Repository
/// TODO: Handle paging
pub fn prs_for_user(author: &str) -> Result<Vec<PrSearchItem>, Box<dyn error::Error>> {
    Ok(reqwest::blocking::Client::new()
        .get(&format!(
            "https://api.github.com/search/issues?q=author:{}+state:closed+is:pr",
            author
        ))
        .header("User-Agent", APP_NAME)
        .header("Accept", "application/vnd.github.cloak-preview")
        .send()?
        .json::<PrSearchResp>()?
        .items)
}

pub fn lines_of(
    repo: &str,
    branch: &str,
    file: &str,
) -> Result<Vec<String>, Box<dyn error::Error>> {
    Ok(reqwest::blocking::Client::new()
        .get(&format!(
            "https://raw.githubusercontent.com/{}/{}/{}",
            repo, branch, file
        ))
        .header("User-Agent", APP_NAME)
        .header("Accept", "application/vnd.github.cloak-preview")
        .send()?
        .text()?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| String::from(s))
        .collect::<Vec<String>>())
}
