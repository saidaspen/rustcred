# 🦀 RustCred
Open Source is not a competition. Well actually, at RustCred, it is...
It's a small Rust application that queries GitHub for data and then populates the HTML that is published to [https:://rustcred.dev](https://rustcred.dev).

The main idea is to generate a score for contributing to Rust related open source projects. It's not serious, it's just for fun.

This is meant to be fun. It is meant to inspire you to contribute to Rust Open Source. Don't take it too seriously.

## Running this yourself
If you, for some reason want to fork this repo and use it for something else, here are some instructions on how it is used:

Build the repo using:
```
cargo build --release
```

Then run the app using: 
```
rustcred --token <your github personal access token> --user <github username> --templates <directory where the html templates are located> --output <output after populating the html>
```

Please note that the app only outputs the HTML files, the CSS file is copied manually from the ```templates/``` dir.

## Rustcred.dev
The scores list is updated once per day, by me, manually. Basically, it gives you one award per repo, out of the tracked repos, that you have contributed to. You get a balloon, a silver medal or a gold medal depending on how many contributions you have to that repo.
- 1 contribution gives a 🎈
- 5 contribution gives a 🥈
- 10 contribution gives a 🥇

### How do I get my name on the scores list?
Simple! Just star this repo, and your name will pop-up on the scores list [RustCred.dev](https://rustcred.dev) tomorrow. 

### But I want to start this great repo, but I don't want my username on the scores list!
If you want to star the repo on GitHub, to show me you like the project, but you don't want to end up on the scores list, then you can open a pull-request on the RustCred repo to add your user name to the list of users who opted out. Once that is done, your name won't appear anymore on the scores list. 

### Adding repo to tracked repos
If you think a repo is missing create a pull-request on the RustCred repo to add that repo name to the tracked_repos file. Once that is merged, it will be considered for scores counting. 

### Why does this page look so much like Advent of Code?
I just love [Advent of Code](https://adventofcode.com/). It is the highlight of the year. I strongly recommend participating and trying to solve some of its nuggets of programming exercises.
Try to do them in Rust, it's a challenge, but perfectly doable!

## Disclaimer
RustCred was my first Rust project while learning Rust. I take no responsibility for what you might do with it. I am, however, open to contributions and pull-requests!
Any changes is done at my discretion; how the site works, what repos are tracked for scores, how often the web-page it is updated, etc.
This is meant to be fun, nothing else. 
