mod github;


fn main() {
    let participants = match github::get_participants() {
        Ok(users) => users,
        _ => panic!("Unable to get participants"),
    };
    println!("{:?}", participants);
}


