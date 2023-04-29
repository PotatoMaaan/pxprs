use clap::Parser;
use serde::Deserialize;
use serde_json;
use std::{fmt::Display, time::Instant};

/// Get the popular images of a specific search term on pixiv
#[derive(Parser, Debug)]
struct Args {
    /// The term to search for
    term: String,
    /// Show only recently popular
    #[arg(short, long)]
    recent: bool,
    /// Show only permanently popular
    #[arg(short, long)]
    permanent: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Post {
    id: String,
    title: String,
    user_name: String,
    user_id: String,
}

#[derive(Deserialize, Debug)]
struct PostList {
    recent: Vec<Post>,
    permanent: Vec<Post>,
}

impl Display for Post {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        println!("Post: {}", get_pixiv_post_url(self));
        println!("   -> Title: {}", self.title);
        println!(
            "   -> Posted by: {} ({})",
            self.user_name,
            get_pixiv_user_url(self)
        );
        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let start = Instant::now();
    let url = format!(
        "https://www.pixiv.net/ajax/search/artworks/{term}?word={term}&lang=en",
        term = args.term
    );
    println!("Sending request to Pixiv API...");

    let body = reqwest::blocking::get(&url)?.text()?;

    let json: serde_json::Value = serde_json::from_str(&body)?;
    if json["error"] == serde_json::Value::Bool(true) {
        println!("The pixiv API returned an Error:");
        println!("{:#}", json);
        return Ok(());
    }

    let typed_json: PostList = serde_json::from_value(json["body"]["popular"].clone())
        .expect("Pixiv API response did not contain [\"body\"][\"popular\"], this means that pixiv has changed the way the API works.");

    if typed_json.permanent.len() == 0 || typed_json.recent.len() == 0 {
        println!(
            "\nPixiv API returned an empty list, {} is probably not a valid Pixiv Tag!",
            args.term
        );
        return Ok(());
    }

    if args.permanent {
        println!("\n\nShowing permanently popular posts: ");
        print_posts(typed_json.permanent);
    } else if args.recent {
        println!("\n\nShowing recently popular posts: ");
        print_posts(typed_json.recent);
    } else {
        println!("\n\nShowing permanently popular posts: ");
        print_posts(typed_json.permanent);
        println!("\n\nShowing recently popular posts: ");
        print_posts(typed_json.recent);
    }

    println!("Took: {:.0?}", start.elapsed());
    Ok(())
}

fn print_posts(posts: Vec<Post>) {
    for post in posts {
        println!("{}\n", post);
    }
}

fn get_pixiv_post_url(post: &Post) -> String {
    return format!("https://www.pixiv.net/en/artworks/{}", post.id);
}

fn get_pixiv_user_url(post: &Post) -> String {
    return format!("https://www.pixiv.net/en/users/{}", post.user_id);
}
