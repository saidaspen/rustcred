use std::error;

use serde::Deserialize;

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
        // User-Agent is mandated by the GitHub API, if not supplied, request will be rejected.
        .header("User-Agent", "RustCred App")
        .send()?
        .json::<Vec<User>>()?)
}

/// Gets the number of merged pull-requests a certain author has for a certain GitHub Repository
/// TODO: Handle paging
pub fn prs_for_user(author: &str) -> Result<Vec<PrSearchItem>, Box<dyn error::Error>> {
    let query = format!(
        "https://api.github.com/search/issues?q=author:{}+state:closed+is:pr",
        author
    );
    let resp = reqwest::blocking::Client::new()
        .get(&query)
        // User-Agent is mandated by the GitHub API, if not supplied, request will be rejected.
        .header("User-Agent", "RustCred App")
        .header("Accept", "application/vnd.github.cloak-preview")
        .send()?
        .json::<PrSearchResp>()?;
    Ok(resp.items)
}

pub fn lines_of(
    repo: &str,
    branch: &str,
    file: &str,
) -> Result<Vec<String>, Box<dyn error::Error>> {
    let query = format!(
        "https://raw.githubusercontent.com/{}/{}/{}",
        repo, branch, file
    );
    let resp: String = reqwest::blocking::Client::new()
        .get(&query)
        // User-Agent is mandated by the GitHub API, if not supplied, request will be rejected.
        .header("User-Agent", "RustCred App")
        .header("Accept", "application/vnd.github.cloak-preview")
        .send()?
        .text()?;
    Ok(resp
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| String::from(s))
        .collect::<Vec<String>>())
}
