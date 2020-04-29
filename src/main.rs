mod github;
use github::{Contribution, GitHubConn, User};
use indicatif::ProgressBar;
use std::collections::HashMap;
use std::collections::HashSet;
use std::env;
use std::error;
use std::fs;

extern crate clap;
extern crate tera;

extern crate chrono;
use chrono::{DateTime, Utc};

use clap::{App, Arg};
use tera::Context;
use tera::Tera;

const REPO: &str = "saidaspen/rustcred";
const VERSION: &'static str = env!("CARGO_PKG_VERSION");

/// These are the limits to get a certain mark.
/// 10 Contributions is a Gold mark
/// 5 Contributions is a Silver mark
/// 1 Contribution is a banlloon
const GOLD_LIMIT: u32 = 10;
const SILVER_LIMIT: u32 = 5;
const BALLOONS_LIMIT: u32 = 1;

fn main() {
    let matches = App::new("RustCred")
        .version(VERSION)
        .author("Said Aspen <info@rustcred.dev>")
        .about("Scores for your Rust Open Source contributions")
        .arg(
            Arg::with_name("output directory")
                .short("o")
                .long("output")
                .takes_value(true)
                .required(true)
                .help("output directory"),
        )
        .arg(
            Arg::with_name("templates directory")
                .short("t")
                .long("templates")
                .takes_value(true)
                .required(true)
                .help("Directory with the input html tempalates"),
        )
        .arg(
            Arg::with_name("github token")
                .short("g")
                .long("token")
                .takes_value(true)
                .required(true)
                .help("Github Personal Token needed"),
        )
        .arg(
            Arg::with_name("github username")
                .short("u")
                .long("user")
                .takes_value(true)
                .required(true)
                .help("Github username needed"),
        )
        .get_matches();
    let templates_dir = matches.value_of("templates directory").expect("a"); //Empty expects because these seems to be enforced and handled by Clap
    let output_dir = matches.value_of("output directory").expect("b");
    let token = matches.value_of("github token").expect("c");
    let user = matches.value_of("github username").expect("d");

    let gh = GitHubConn::new(token.to_string(), user.to_string(), REPO.to_string());

    // Get list of participants (everyone who has starred the GitHub Repo)
    println!("[1/11] Getting participant...");
    let participants: Vec<User> = gh.get_participants().expect("Unable to get partricipants.");

    // Read the users who has opted out (Everyone in the opted_out file in the GitHub repo)
    println!("[2/11] Getting opted out users...");
    let opted_out: Vec<String> = lines_of("opted_out").unwrap_or_else(|_| vec![]);

    // Filter out participants who have opted out
    // These are people who wanted to star the repo, but who does not want to show up in the scores
    // list.
    println!("[3/11] Filtering users...");
    let participants: HashSet<String> = participants
        .iter()
        .filter(|p| !opted_out.contains(&p.login))
        .cloned()
        .map(|p| p.login)
        .collect();

    // Get all tracked repos
    // Each repo is specified on its own line in the tracked_repos file in the GitHub repo.
    println!("[4/11] Getting tracked repos...");
    let tracked_repos: Vec<String> =
        lines_of("tracked_repos").expect("Unable to read tracked_repos file");

    // Scores is mapped from github username to RepoContribution
    let mut scores: HashMap<String, Vec<Contribution>> = HashMap::new();

    // Keeps track of the number of RustCred participants who has contributed to a specific repo.
    // Maps from repo name to number of contributions.
    let mut total_repo_contribs: HashMap<String, u32> = HashMap::new();

    let pb = ProgressBar::new(tracked_repos.len() as u64);

    println!("[5/11] Getting contributors for repos...");
    for repo in &tracked_repos {
        total_repo_contribs.insert(repo.clone(), 0);
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
                .and_modify(|contribs| *contribs += 1);
        }
        pb.inc(1);
    }
    pb.finish_and_clear();

    let mut total_repo_contribs: Vec<(String, u32)> = total_repo_contribs
        .iter()
        .map(|(k, v)| (k.to_owned(), *v))
        .collect();
    total_repo_contribs.sort_by(|a, b| b.1.cmp(&a.1));
    // Change scores such that it now is a vector of Score sorted by the RustCred
    println!("[6/11] Mapping scores...");
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
    println!("[7/11] Sorting scores...");
    scores.sort_by(|a, b| b.rust_cred.cmp(&a.rust_cred));

    let mut tera = match Tera::new(format!("{}/*.html", templates_dir).as_ref()) {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    tera.autoescape_on(vec![]);

    println!("[8/11] Rendering trackedrepos.html...");
    let tracked_html = render_tracked_repos(&tera, &total_repo_contribs);
    println!("[9/11] Rendering about.html...");
    let about_html = render_about(&tera);
    println!("[10/11] Rendering index.html...");
    let scores_html = render_scores(&tera, &scores);

    println!("[11/11] Writing files to {}...", output_dir);
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

fn render_tracked_repos(tera: &Tera, total_repo_contribs: &Vec<(String, u32)>) -> String {
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

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
struct Score {
    user: String,
    gold: u32,
    silver: u32,
    balloons: u32,
    rust_cred: u32,
}

fn lines_of(f_name: &str) -> Result<Vec<String>, Box<dyn error::Error>> {
    Ok(fs::read_to_string(f_name)?
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(|s| String::from(s.trim()))
        .filter(|s| !s.starts_with('#'))
        .collect::<Vec<String>>())
}
