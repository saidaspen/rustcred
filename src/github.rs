use std::error;

use serde::Deserialize;
const APP_NAME: &str = "RustCred";
const REPO_URL: &str = "https://api.github.com/repos/";
const USER_PER_PAGE: u32 = 100;

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
    items: Vec<Pr>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Pr {
    pub url: String,
    pub repository_url: String,
    id: u32,
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
    return String::from("https://raw.githubusercontent.com");

    #[cfg(test)]
    mockito::server_url()
}

pub struct GitHubConn {
    pub token: String,
    pub repo: String,
}

impl GitHubConn {
    pub fn new(token: String, repo: String) -> GitHubConn {
        GitHubConn {
            token: token,
            repo: repo,
        }
    }

    fn query_gh(&self, url: &String) -> Result<String, Box<dyn error::Error>> {
        Ok(reqwest::blocking::Client::new()
            .get(url)
            .header("User-Agent", APP_NAME)
            .header("Authorization", &self.token)
            .header("Accept", "application/vnd.github.cloak-preview")
            .send()?
            .text()?)
    }

    /// Get's all the GitHub users who have starred the RustCred repository
    /// GitHub Users who have starred the repository is considered to be participants in RustCred.
    /// It is implemented by blocking, so it will block the outgoing web-request to GitHub and will
    /// thus take a while to execute.
    pub fn get_participants(&self) -> Result<Vec<User>, Box<dyn error::Error>> {
        let mut page: u32 = 0;
        let mut users = vec![];
        while users.len() % USER_PER_PAGE as usize == 0 {
            page = page + 1;
            let additional_users = &self.get_participants_page(page)?;
            if additional_users.len() == 0 {
                break;
            }
            users = [&users[..], &additional_users].concat();
        }
        Ok(users)
    }

    fn get_participants_page(&self, page: u32) -> Result<Vec<User>, Box<dyn error::Error>> {
        let url = &format!(
            "{}/repos/{}/stargazers?per_page={}&page={}",
            github_url(),
            &self.repo,
            USER_PER_PAGE,
            page
        );
        Ok(serde_json::from_str(&self.query_gh(url)?)?)
    }

    /// Gets the number of merged pull-requests a certain author has for a certain GitHub Repository
    pub fn merged_prs_for(&self, author: &str) -> Result<Vec<Pr>, Box<dyn error::Error>> {
        let mut page: u32 = 0;
        let mut prs = vec![];
        while prs.len() % USER_PER_PAGE as usize == 0 {
            page = page + 1;
            let additional_prs = &self.get_merged_prs_page(&author, page)?;
            if additional_prs.len() == 0 {
                break;
            }
            prs = [&prs[..], &additional_prs].concat();
        }
        Ok(prs)
    }

    fn get_merged_prs_page(
        &self,
        author: &str,
        page: u32,
    ) -> Result<Vec<Pr>, Box<dyn error::Error>> {
        let url = format!(
            "{}/search/issues?q=author:{}+state:closed+is:merged&per_page={}&page={}",
            github_url(),
            author,
            USER_PER_PAGE,
            page
        );
        let prs: PrSearchResp = serde_json::from_str(&self.query_gh(&url)?.to_string())?;
        Ok(prs.items)
    }

    pub fn lines_of(&self, branch: &str, file: &str) -> Result<Vec<String>, Box<dyn error::Error>> {
        let url = format!(
            "{}/{}/{}/{}",
            raw_content_github_url(),
            &self.repo,
            branch,
            file
        );
        Ok(self
            .query_gh(&url)?
            .lines()
            .filter(|s| !s.trim().is_empty())
            .map(|s| String::from(s.trim()))
            .collect::<Vec<String>>())
    }
}

pub fn repo_name(url: &str) -> String {
    url[REPO_URL.len()..].to_string()
}

#[cfg(test)]
mod tests {
    use crate::github::{repo_name, GitHubConn, User};
    use mockito::mock;
    use std::error;

    #[test]
    fn test_repo_name() {
        assert_eq!(
            repo_name("https://api.github.com/repos/saidaspen/rustcred"),
            "saidaspen/rustcred"
        );
    }

    fn mock_with(url: &str, body: &str) -> mockito::Mock {
        mock("GET", url)
            .with_status(200)
            .with_header("content-type", "application/json; charset=utf-8")
            .with_body(body)
            .create()
    }

    #[test]
    fn gets_pr_for_user_w_paging() -> Result<(), Box<dyn error::Error>> {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        let pr = "{\"url\": \"someurl\", \"repository_url\": \"repo_url\", \"id\": 123, \"state\": \"closed\", \"score\":1.0}";
        let prs_first = vec![pr; 100];
        let first_page = format!(
            "{{\"total_count\":{}, \"items\":[{}]}}",
            prs_first.len(),
            prs_first.join(", ")
        );
        let prs_second = vec![pr; 10];
        let second_page = format!(
            "{{\"total_count\":{}, \"items\":[{}]}}",
            prs_second.len(),
            prs_second.join(", ")
        );
        let _m = mock_with(
            "/search/issues?q=author:saidaspen+state:closed+is:merged&per_page=100&page=1",
            &first_page,
        );
        let _m = mock_with(
            "/search/issues?q=author:saidaspen+state:closed+is:merged&per_page=100&page=2",
            &second_page,
        );
        let prs = conn.merged_prs_for("saidaspen")?;
        assert_eq!(prs.len(), 110);
        Ok(())
    }

    #[test]
    fn gets_lines_of_file() {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        // Here we also have a newline in the middle to test it does not add those.
        let body = r#"
        line1 

        line2 
        "#;
        let _m = mock_with("/saidaspen/rustcred/master/somefile", &body);
        let opted_out_users = match conn.lines_of("master", "somefile") {
            Ok(l) => l,
            Err(e) => panic!("unable to get opted out users: {:?}", e),
        };
        assert_eq!(opted_out_users.len(), 2);
        assert_eq!(opted_out_users[0], "line1");
        assert_eq!(opted_out_users[1], "line2");
    }

    #[test]
    fn get_participants_paging_second_empty() -> Result<(), Box<dyn error::Error>> {
        let conn = GitHubConn::new("test".to_string(), "saidaspen/rustcred".to_string());
        let first_page = format!(
            "[{}]\n",
            vec!["{\"login\": \"user\", \"id\": 1, \"url\": \"https://someuserurl\"}"; 100]
                .join(", ")
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
            vec!["{\"login\": \"user\", \"id\": 1, \"url\": \"https://someuserurl\"}"; 99];
        let other_user = "{\"login\": \"other_user\", \"id\": 2, \"url\": \"https://someuserurl\"}";
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
                id: 1,
                url: "https://someuserurl".to_string(),
            }
        );
        assert_eq!(
            participants[99],
            User {
                login: "other_user".to_string(),
                id: 2,
                url: "https://someuserurl".to_string(),
            }
        );
        assert_eq!(
            participants[201],
            User {
                login: "other_user".to_string(),
                id: 2,
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
