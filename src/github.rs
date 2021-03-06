use serde::Deserialize;
use std::error;

const APP_NAME: &str = "RustCred";
const USER_PER_PAGE: u32 = 100;

/// Note that for some of these structs, the corresponding JSON from the GitHub API has several more fields, not all of them are interesting to us.
/// Have a look at the GitHub API Documentation for details: https://developer.github.com/v3/users/

/// User representation of the GitHub API.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct User {
    pub login: String,
    url: String,
}

/// Representing a Contribution for a specific repository in the GitHub API.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct Contribution {
    pub login: String,
    #[serde(rename(deserialize = "contributions"))]
    pub num: u32,
}

/// Get the github main URL
/// Returns a different url for testing
fn github_url() -> String {
    #[cfg(not(test))]
    return String::from("https://api.github.com");

    #[cfg(test)]
    mockito::server_url()
}

/// GitHubConn represents one RustCred "connection" to the GitHub API.
pub struct GitHubConn {
    pub github_user: String,
    pub token: String,
    pub repo: String,
}

impl GitHubConn {
    pub fn new(token: String, github_user: String, repo: String) -> GitHubConn {
        GitHubConn {
            token,
            github_user,
            repo,
        }
    }

    /// Query the GitHub API using the given URL, will use the provided developer token.
    /// Helper method for reducing code duplication
    fn query_gh(&self, url: &str) -> Result<String, Box<dyn error::Error>> {
        Ok(reqwest::blocking::Client::new()
            .get(url)
            .header("User-Agent", APP_NAME)
            .basic_auth(&self.github_user, Some(&self.token.to_owned()))
            .header("Accept", "application/vnd.github.cloak-preview")
            .send()?
            .text()?)
    }

    /// Gets all contributors for a certains repo.
    /// It supports the paging mechanism of the GitHub API and will retrieve 100 hits per page.
    pub fn get_contributors(&self, repo: &str) -> Result<Vec<Contribution>, Box<dyn error::Error>> {
        let mut page: u32 = 0;
        let mut contribs = vec![];
        while contribs.len() % USER_PER_PAGE as usize == 0 {
            page += 1;
            let additional_contribs = &self.get_contribs_page(&repo, page)?;
            if additional_contribs.is_empty() {
                break;
            }
            contribs = [&contribs[..], &additional_contribs].concat();
        }
        Ok(contribs)
    }

    /// Get a single page of contributors
    fn get_contribs_page(
        &self,
        repo: &str,
        page: u32,
    ) -> Result<Vec<Contribution>, Box<dyn error::Error>> {
        let url = &format!(
            "{}/repos/{}/contributors?per_page={}&page={}",
            github_url(),
            &repo,
            USER_PER_PAGE,
            page
        );
        Ok(serde_json::from_str(&self.query_gh(url)?)?)
    }

    /// Get's all the GitHub users who have starred the RustCred repository
    /// GitHub Users who have starred the repository is considered to be participants in RustCred.
    /// It is implemented by blocking, so it will block the outgoing web-request to GitHub and will
    /// thus take a while to execute.
    pub fn get_participants(&self) -> Result<Vec<User>, Box<dyn error::Error>> {
        let mut page: u32 = 0;
        let mut users = vec![];
        while users.len() % USER_PER_PAGE as usize == 0 {
            page += 1;
            let additional_users = &self.get_participants_page(page)?;
            if additional_users.is_empty() {
                break;
            }
            users = [&users[..], &additional_users].concat();
        }
        Ok(users)
    }

    /// Get a single page of participants
    fn get_participants_page(&self, page: u32) -> Result<Vec<User>, Box<dyn error::Error>> {
        let url = &format!(
            "{}/repos/{}/stargazers?per_page={}&page={}",
            github_url(),
            &self.repo,
            USER_PER_PAGE,
            page
        );
        let result = &self.query_gh(url)?;
        Ok(serde_json::from_str(result)?)
    }
}

#[cfg(test)]
mod tests {
    use crate::github::{GitHubConn, User};
    use mockito::mock;
    use std::error;

    fn mock_with(url: &str, body: &str) -> mockito::Mock {
        mock("GET", url)
            .with_status(200)
            .with_header("content-type", "application/json; charset=utf-8")
            .with_body(body)
            .create()
    }

    #[test]
    fn get_contributors() -> Result<(), Box<dyn error::Error>> {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        let page: Vec<&str> = vec!["{\"login\": \"testuser\", \"contributions\": 2}"; 99];
        let page = format!("[{}]\n", &page.join(", "));
        let _m = mock_with(
            "/repos/saidaspen/rustcred/contributors?per_page=100&page=1",
            &page,
        );
        let contributors = conn.get_contributors("saidaspen/rustcred")?;
        assert_eq!(contributors.len(), 99);
        Ok(())
    }

    #[test]
    fn get_participants_paging_second_empty() -> Result<(), Box<dyn error::Error>> {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        let first_page = format!(
            "[{}]\n",
            vec!["{\"login\": \"user\", \"url\": \"https://someuserurl\"}"; 100].join(", ")
        );
        let _m = mock_with(
            "/repos/saidaspen/rustcred/stargazers?per_page=100&page=1",
            &first_page,
        );
        let _m = mock_with(
            "/repos/saidaspen/rustcred/stargazers?per_page=100&page=2",
            "[]",
        );
        assert_eq!(conn.get_participants()?.len(), 100);
        Ok(())
    }

    #[test]
    fn get_participants_paging() -> Result<(), Box<dyn error::Error>> {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        let mut page: Vec<&str> =
            vec!["{\"login\": \"user\", \"url\": \"https://someuserurl\"}"; 99];
        let other_user = "{\"login\": \"other_user\", \"url\": \"https://someuserurl\"}";
        page.push(other_user);
        let first_page = format!("[{}]\n", &page.join(", "));
        let second_page = format!("[{}]\n", &page.join(", "));
        let third_page = format!("[{}]\n", vec![other_user, other_user].join(", "));
        let _m = mock_with(
            "/repos/saidaspen/rustcred/stargazers?per_page=100&page=1",
            &first_page,
        );
        let _m = mock_with(
            "/repos/saidaspen/rustcred/stargazers?per_page=100&page=2",
            &second_page,
        );
        let _m = mock_with(
            "/repos/saidaspen/rustcred/stargazers?per_page=100&page=3",
            &third_page,
        );
        let participants = conn.get_participants()?;
        assert_eq!(participants.len(), 202);
        assert_eq!(
            participants[0],
            User {
                login: "user".to_string(),
                url: "https://someuserurl".to_string(),
            }
        );
        assert_eq!(
            participants[99],
            User {
                login: "other_user".to_string(),
                url: "https://someuserurl".to_string(),
            }
        );
        assert_eq!(
            participants[201],
            User {
                login: "other_user".to_string(),
                url: "https://someuserurl".to_string(),
            }
        );
        Ok(())
    }

    #[test]
    fn gets_participants_no_users() -> Result<(), Box<dyn error::Error>> {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        let _m = mock(
            "GET",
            "/repos/saidaspen/rustcred/stargazers?per_page=100&page=1",
        )
        .with_status(200)
        .with_header("content-type", "application/json; charset=utf-8")
        .with_body("[]")
        .create();
        let participants = conn.get_participants()?;
        assert_eq!(participants.len(), 0);
        Ok(())
    }
}
