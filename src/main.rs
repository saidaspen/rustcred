mod github;
use github::{repo_name, GitHubConn, User};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;

const REPO: &str = "paupino/rust-decimal"; //"saidaspen/rustcred";
const BRANCH: &str = "master";
const TOKEN_PROP_NAME: &str = "RC_GITHUB_TOKEN";

fn main() {
    let github_token = match env::var(TOKEN_PROP_NAME) {
        Ok(s) => s,
        Err(_) => {
            println!(
                r#"Environment property {} not set.
This needs to be set to your personal access token from GitHub.
See https://help.github.com/en/github/authenticating-to-github/creating-a-personal-access-token-for-the-command-line"#,
                TOKEN_PROP_NAME
            );
            return;
        }
    };

    let gh = GitHubConn::new(github_token, REPO.to_string());

    // Get participants
    let participants: Vec<User> = gh.get_participants().expect("Unable to get partricipants.");

    // Read the users who has opted out
    let opted_out: Vec<String> = gh.lines_of(BRANCH, "opted_out").unwrap_or_else(|_| vec![]);

    // Filter out participants who have opted out
    let participants: Vec<User> = participants
        .iter()
        .filter(|p| !opted_out.contains(&p.login))
        .map(|x| x.clone())
        .collect();

    print_participants(&participants);
    println!("{}", &participants.len());

    // Get all tracked repos
    let tracked_repos: HashSet<String> = gh
        .lines_of(BRANCH, "tracked_repos")
        .expect("expected to find tracked_repos file")
        .iter()
        .map(|repo| format!("https://api.github.com/repos/{}", repo))
        .collect();

    for p in participants {
        let prs = gh
            .merged_prs_for(&p.login)
            .unwrap_or_else(|e| panic!("unable to get PRs for user {}, error: {} ", p.login, e))
            .into_iter()
            .map(|pr| pr.repository_url)
            .filter(|repo| tracked_repos.contains(repo))
            .fold(HashMap::new(), |mut map, repo| {
                *map.entry(repo_name(&repo)).or_insert(0) += 1;
                map
            });
        println!("User: {}", &p.login);
        println!("--------------------------------");
        for (k, v) in prs.iter() {
            println!("{}\t{}", k, v);
        }
    }
    //
    // Generate HTML page for overview
    // For each user {
    //  Generate HTML for user
    // }
}

fn print_participants(participants: &Vec<User>) {
    println!(
        "All participants (after filtering): {:?}",
        participants
            .iter()
            .map(|p| p.login.to_string())
            .collect::<Vec<String>>()
    );
}
