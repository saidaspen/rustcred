mod github;

fn main() {
    // Get participants
    let participants: Vec<github::User> = match github::get_participants() {
        Ok(users) => users,
        _ => panic!("Unable to get participants."),
    };
    println!("All participants: {:?}", participants);

    // Read the users who has opted out
    let opted_out: Vec<String> =
        github::lines_of("saidaspen/rustcred", "master", "opted_out").unwrap_or_else(|x| vec![]);
    println!("Users who has opted out: {:?}", opted_out);

    // Filter out participants who have opted out
    let participants: Vec<github::User> = participants
        .iter()
        .filter(|p| !opted_out.contains(&p.login))
        .map(|x| x.clone())
        .collect();
    println!("All participants (after filtering): {:?}", participants);

    // Get all tracked repos
    let tracked_repos: Vec<String> =
        github::lines_of("saidaspen/rustcred", "master", "tracked_repos")
            .expect("expected to find tracked_repos file");
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
