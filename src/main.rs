mod github;
use github::{Contribution, GitHubConn, User};
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
const VERSION: &str = "0.0.1";

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

    let matches = App::new("RustCred")
        .version("0.1.0")
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
    let templates_dir = matches.value_of("templates").expect("");
    let output_dir = matches.value_of("output").expect("");

    let gh = GitHubConn::new(github_token, REPO.to_string());

    // Get participants
    let participants: Vec<User> = gh.get_participants().expect("Unable to get partricipants.");

    // Read the users who has opted out
    let opted_out: Vec<String> = gh.lines_of(BRANCH, "opted_out").unwrap_or_else(|_| vec![]);

    // Filter out participants who have opted out
    let participants: HashSet<String> = participants
        .iter()
        .filter(|p| !opted_out.contains(&p.login))
        .cloned()
        .map(|p| p.login)
        .collect();

    println!("All participants: {:?}", &participants);

    // Get all tracked repos
    let tracked_repos: Vec<String> = gh
        .lines_of(BRANCH, "tracked_repos")
        .expect("expected to find tracked_repos file");

    let mut scores: HashMap<String, Vec<RepoContribution>> = HashMap::new();
    let mut total_repo_contribs: HashMap<String, u32> = HashMap::new();

    for repo in &tracked_repos {
        let contributions: Vec<Contribution> = gh
            .get_contributors(&repo)
            .expect("unable to get contributors for repo")
            .iter()
            .filter(|c| participants.contains(&c.login))
            .cloned()
            .collect();
        for contrib in contributions {
            let login = contrib.login.to_string();
            total_repo_contribs
                .entry(repo.clone())
                .and_modify(|contribs| {
                    *contribs += 1;
                })
                .or_insert_with(|| 1);
            scores
                .entry(login.clone())
                .and_modify(|repo_contribs| {
                    repo_contribs.push(RepoContribution::new(repo.clone(), contrib.contributions));
                })
                .or_insert_with(|| {
                    vec![RepoContribution::new(repo.clone(), contrib.contributions)]
                });
        }
    }
    let scores = scores
        .iter()
        .map(|(k, v)| {
            let mut gold = 0;
            let mut silver = 0;
            let mut balloons = 0;
            for c in v {
                match c.contributions {
                    n if n >= 10 => gold += 1,
                    n if n >= 5 => silver += 1,
                    n if n >= 1 => balloons += 1,
                    _ => {}
                };
            }
            Score {
                user: k.to_string(),
                gold,
                silver,
                balloons,
                total_contribs: gold * 10 + silver * 5 + balloons,
            }
        })
        .collect();
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
    context.insert("updated_at", &format!("{}", &now.format("%b %e, %Y")));
    context.insert("version", VERSION);
    match tera.render("about.html", &context) {
        Ok(s) => s,
        Err(e) => panic!("Unable to render file. {}", e),
    }
}

fn render_scores(tera: &Tera, scores: &Vec<Score>) -> String {
    let mut context = Context::new();
    let now: DateTime<Utc> = Utc::now();
    context.insert("updated_at", &format!("{}", &now.format("%b %e, %Y")));
    context.insert("scores", &scores);
    context.insert("version", VERSION);
    match tera.render("index.html", &context) {
        Ok(s) => s,
        Err(e) => panic!("Unable to render file. {}", e),
    }
}

fn render_tracked_repos(tera: &Tera, total_repo_contribs: &HashMap<String, u32>) -> String {
    let mut context = Context::new();
    let now: DateTime<Utc> = Utc::now();
    context.insert("tracked_repos", &total_repo_contribs);
    context.insert("updated_at", &format!("{}", &now.format("%b %e, %Y")));
    context.insert("version", VERSION);
    match tera.render("trackedrepos.html", &context) {
        Ok(s) => s,
        Err(e) => panic!("Unable to render file. {}", e),
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, PartialEq)]
struct Score {
    user: String,
    gold: usize,
    silver: usize,
    balloons: usize,
    total_contribs: usize,
}

struct RepoContribution {
    repo: String,
    contributions: u32,
}

impl RepoContribution {
    fn new(repo: String, contributions: u32) -> RepoContribution {
        RepoContribution {
            repo,
            contributions,
        }
    }
}
