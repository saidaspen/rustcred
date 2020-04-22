use std::error;

use serde::Deserialize;
const STARGAZERS: &str = "/repos/saidaspen/rustcred/stargazers";
const APP_NAME: &str = "RustCred";

/// User representation of the GitHub API.
/// Note that the GitHub API has several more fields, not all of them are interesting to us.
/// Have a look at the GitHub API Documentation for details: https://developer.github.com/v3/users/
#[derive(Deserialize, Debug, Clone, PartialEq)]
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

fn github_url() -> String {
    #[cfg(not(test))]
    return String::from("https://api.github.com");

    #[cfg(test)]
    mockito::server_url()
}

fn raw_content_github_url() -> String {
    #[cfg(not(test))]
    return String::from("raw.githubusercontent.com");

    #[cfg(test)]
    mockito::server_url()
}

/// Get's all the GitHub users who have starred the RustCred repository
/// GitHub Users who have starred the repository is considered to be participants in RustCred.
/// It is implemented by blocking, so it will block the outgoing web-request to GitHub and will
/// thus take a while to execute.
pub fn get_participants() -> Result<Vec<User>, Box<dyn error::Error>> {
    Ok(reqwest::blocking::Client::new()
        .get(&format!("{}{}", github_url(), STARGAZERS))
        .header("User-Agent", APP_NAME)
        .send()?
        .json::<Vec<User>>()?)
}

/// Gets the number of merged pull-requests a certain author has for a certain GitHub Repository
/// TODO: Handle paging
pub fn prs_for_user(author: &str) -> Result<Vec<PrSearchItem>, Box<dyn error::Error>> {
    Ok(reqwest::blocking::Client::new()
        .get(&format!(
            "{}/search/issues?q=author:{}+state:closed+is:pr",
            github_url(),
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
    let url = format!("{}/{}/{}/{}", raw_content_github_url(), repo, branch, file);
    println!("{:?}", url);

    Ok(reqwest::blocking::Client::new()
        .get(&url)
        .header("User-Agent", APP_NAME)
        .header("Accept", "application/vnd.github.cloak-preview")
        .send()?
        .text()?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| String::from(s.trim()))
        .collect::<Vec<String>>())
}

#[cfg(test)]
mod tests {
    use crate::github::{get_participants, lines_of, User, STARGAZERS};
    use mockito::mock;

    #[test]
    fn gets_lines_of_file() {
        // Here we also have a newline in the middle to test it does not add those.
        let body = r#"
        line1 

        line2 
        "#;
        let _m = mock("GET", "/saidaspen/rustcred/master/somefile")
            .with_status(200)
            .with_header("content-type", "text/plain; charset=utf-8")
            .with_body(&body)
            .create();
        let opted_out_users = match lines_of("saidaspen/rustcred", "master", "somefile") {
            Ok(l) => l,
            Err(e) => panic!("unable to get opted out users: {:?}", e),
        };
        assert_eq!(opted_out_users.len(), 2);
        assert_eq!(opted_out_users[0], "line1");
        assert_eq!(opted_out_users[1], "line2");
    }

    #[test]
    fn gets_participants() {
        let body = r#"
        [
            {
                "login": "firstuser",
                "id": 1,
                "url": "https://api.github.com/users/firstuser"
            },
            {
                "login": "user_w_unused_field",
                "id": 2,
                "url": "https://api.github.com/users/user_w_unused_field",
                "some_unused_fields": "value"
            }
        ]"#;
        let _m = mock("GET", STARGAZERS)
            .with_status(200)
            .with_header("content-type", "application/json; charset=utf-8")
            .with_body(&body)
            .create();
        let participants = match get_participants() {
            Ok(p) => p,
            Err(e) => panic!("unable to get participants {:?}", e),
        };
        assert_eq!(participants.len(), 2);
        assert_eq!(
            participants[0],
            User {
                login: "firstuser".to_string(),
                id: 1,
                url: "https://api.github.com/users/firstuser".to_string()
            }
        );
        assert_eq!(
            participants[1],
            User {
                login: "user_w_unused_field".to_string(),
                id: 2,
                url: "https://api.github.com/users/user_w_unused_field".to_string()
            }
        );
    }
}
