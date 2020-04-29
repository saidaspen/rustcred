mod github;
use github::{Contribution, GitHubConn, User};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::fs;

extern crate clap;
extern crate tera;

extern crate chrono;
use chrono::{DateTime, Utc};

use clap::{App, Arg};
use tera::Context;
use tera::Tera;

const REPO: &str = "saidaspen/rustcred";
const BRANCH: &str = "master";
const TOKEN_PROP_NAME: &str = "RC_GITHUB_TOKEN";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// These are the limits to get a certain mark.
/// 10 Contributions is a Gold mark
/// 5 Contributions is a Silver mark
/// 1 Contribution is a banlloon
const GOLD_LIMIT: u32 = 10;
const SILVER_LIMIT: u32 = 5;
const BALLOONS_LIMIT: u32 = 1;

fn main() {
    // The application needs to have the environment property with a valid GitHub API Token.
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

    let matches = App::new("RustCred")
        .version(VERSION)
        .author("Said Aspen <info@rustcred.dev>")
        .about("Scores for your Rust Open Source contributions")
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .takes_value(true)
                .required(true)
                .help("output directory"),
        )
        .arg(
            Arg::with_name("templates")
                .short("t")
                .long("templates")
                .takes_value(true)
                .required(true)
                .help("Directory with the input html tempalates"),
        )
        .get_matches();
    let templates_dir = matches.value_of("templates").expect(""); //Empty expects because these seems to be enforced and handled by Clap
    let output_dir = matches.value_of("output").expect("");

    let gh = GitHubConn::new(github_token, REPO.to_string());

    // Get list of participants (everyone who has starred the GitHub Repo)
    let participants: Vec<User> = gh.get_participants().expect("Unable to get partricipants.");

    // Read the users who has opted out (Everyone in the opted_out file in the GitHub repo)
    let opted_out: Vec<String> = gh.lines_of(BRANCH, "opted_out").unwrap_or_else(|_| vec![]);

    // Filter out participants who have opted out
    // These are people who wanted to star the repo, but who does not want to show up in the scores
    // list.
    let participants: HashSet<String> = participants
        .iter()
        .filter(|p| !opted_out.contains(&p.login))
        .cloned()
        .map(|p| p.login)
        .collect();

    // Get all tracked repos
    // Each repo is specified on its own line in the tracked_repos file in the GitHub repo.
    let tracked_repos: Vec<String> = gh
        .lines_of(BRANCH, "tracked_repos")
        .expect("expected to find tracked_repos file");

    // Scores is mapped from github username to RepoContribution
    let mut scores: HashMap<String, Vec<Contribution>> = HashMap::new();

    // Keeps track of the number of RustCred participants who has contributed to a specific repo.
    // Maps from repo name to number of contributions.
    let mut total_repo_contribs: HashMap<String, u32> = HashMap::new();

    for repo in &tracked_repos {
        let contributions: Vec<Contribution> = gh
            .get_contributors(&repo)
            .expect("unable to get contributors for repo")
            .iter()
            .filter(|c| participants.contains(&c.login))
            .cloned()
            .collect();
        for contribution in contributions {
            let login = contribution.login.to_string();
            scores
                .entry(login)
                .and_modify(|usr_contribs| usr_contribs.push(contribution.clone()))
                .or_insert_with(|| vec![contribution.clone()]);
            total_repo_contribs
                .entry(repo.clone())
                .and_modify(|contribs| *contribs += 1)
                .or_insert_with(|| 1);
        }
    }

    // Change scores such that it now is a vector of Score sorted by the RustCred
    let mut scores: Vec<Score> = scores
        .iter()
        .map(|(k, v)| {
            let mut gold = 0;
            let mut silver = 0;
            let mut balloons = 0;
            for c in v {
                match c.num {
                    n if n >= GOLD_LIMIT => gold += 1,
                    n if n >= SILVER_LIMIT => silver += 1,
                    n if n >= BALLOONS_LIMIT => balloons += 1,
                    _ => (),
                };
            }
            Score {
                user: k.to_string(),
                gold,
                silver,
                balloons,
                rust_cred: gold * GOLD_LIMIT + silver * SILVER_LIMIT + balloons * BALLOONS_LIMIT,
            }
        })
        .collect();

    // Sort the scores by RustCred
    scores.sort();

    println!("{:?}", scores);
    let mut tera = match Tera::new(format!("{}/*.html", templates_dir).as_ref()) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![]);

    let tracked_html = render_tracked_repos(&tera, &total_repo_contribs);
    let about_html = render_about(&tera);
    let scores_html = render_scores(&tera, &scores);
    fs::write(format!("{}/trackedrepos.html", output_dir), tracked_html)
        .expect("unable to write file trackedrepos.html");
    fs::write(format!("{}/about.html", output_dir), about_html)
        .expect("unable to write file about.html");
    fs::write(format!("{}/index.html", output_dir), scores_html)
        .expect("Unable to write file index.html");
}

fn render_about(tera: &Tera) -> String {
    let mut context = Context::new();
    let now: DateTime<Utc> = Utc::now();
    let f_name = "about.html";
    context.insert("updated_at", &format!("{}", &now.format("%b %e, %Y")));
    context.insert("version", VERSION);
    match tera.render(f_name, &context) {
        Ok(s) => s,
        Err(e) => panic!("Unable to render file {}. Reason: {}", f_name, e),
    }
}

fn render_scores(tera: &Tera, scores: &Vec<Score>) -> String {
    let mut context = Context::new();
    let now: DateTime<Utc> = Utc::now();
    let f_name = "index.html";
    context.insert("updated_at", &format!("{}", &now.format("%b %e, %Y")));
    context.insert("scores", &scores);
    context.insert("version", VERSION);
    match tera.render(f_name, &context) {
        Ok(s) => s,
        Err(e) => panic!("Unable to render file {}. Reason: {}", f_name, e),
    }
}

fn render_tracked_repos(tera: &Tera, total_repo_contribs: &HashMap<String, u32>) -> String {
    let mut context = Context::new();
    let now: DateTime<Utc> = Utc::now();
    let f_name = "trackedrepos.html";
    context.insert("tracked_repos", &total_repo_contribs);
    context.insert("updated_at", &format!("{}", &now.format("%b %e, %Y")));
    context.insert("version", VERSION);
    match tera.render(f_name, &context) {
        Ok(s) => s,
        Err(e) => panic!("Unable to render file {}. Reason: {:?}", f_name, e),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Eq)]
struct Score {
    user: String,
    gold: u32,
    silver: u32,
    balloons: u32,
    rust_cred: u32,
}

impl Ord for Score {
    fn cmp(&self, other: &Self) -> Ordering {
        self.rust_cred.cmp(&other.rust_cred)
    }
}

impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.rust_cred == other.rust_cred
    }
}

impl PartialOrd for Score {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
