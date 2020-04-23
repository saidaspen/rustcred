mod github;
use github::{Contribution, GitHubConn, User};
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;

const REPO: &str = "saidaspen/rustcred";
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

    // Get all tracked repos
    let tracked_repos: Vec<String> = gh
        .lines_of(BRANCH, "tracked_repos")
        .expect("expected to find tracked_repos file");

    let participants_set: HashSet<String> =
        participants.iter().map(|p| p.login.to_string()).collect();

    let mut scores: HashMap<String, HashMap<String, u32>> = HashMap::new();
    for repo in tracked_repos {
        let contributions: Vec<Contribution> = gh
            .get_contributors(&repo)
            .expect("unable to get contributors for repo")
            .iter()
            .filter(|c| participants_set.contains(&c.login))
            .cloned()
            .collect();
        for contrib in contributions {
            let login = contrib.login.to_string();
            scores
                .entry(login.clone())
                .and_modify(|nested| {
                    nested.insert(repo.clone(), contrib.contributions);
                })
                .or_insert_with(|| {
                    let mut new_map: HashMap<String, u32> = HashMap::new();
                    new_map.insert(repo.clone(), contrib.contributions);
                    new_map
                });
        }
    }

    for (user, score) in scores.iter() {
        println!("{}", user);
        println!("--------------------------");
        for (repo, contribs) in score {
            println!("{}\t{}", repo, contribs);
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
