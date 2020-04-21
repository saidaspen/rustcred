mod github;
use github::{get_participants, lines_of, User};

const REPO: &str = "saidaspen/rustcred";
const BRANCH: &str = "master";

fn main() {
    // Get participants
    let participants: Vec<User> = get_participants().expect("Unable to get partricipants");
    println!("All participants: {:?}", participants);

    // Read the users who has opted out
    let opted_out: Vec<String> = lines_of(REPO, BRANCH, "opted_out").unwrap_or_else(|_| vec![]);
    println!("Users who has opted out: {:?}", opted_out);

    // Filter out participants who have opted out
    let participants: Vec<User> = participants
        .iter()
        .filter(|p| !opted_out.contains(&p.login))
        .map(|x| x.clone())
        .collect();
    println!("All participants (after filtering): {:?}", participants);

    // Get all tracked repos
    let tracked_repos: Vec<String> =
        lines_of(REPO, BRANCH, "tracked_repos").expect("expected to find tracked_repos file");
    println!("Tracked repos: {:?}", tracked_repos);

    // for each participant {

    //  Get all closed PRs for a specific user
    //  let prs = github::prs_for_user("saidaspen");
    //
    //  Filter out those which are in relevant repos
    //  Group them per repo and count
    //  Map to Score per Repo
    // }
    //
    // Generate HTML page for overview
    // For each user {
    //  Generate HTML for user
    // }
}
