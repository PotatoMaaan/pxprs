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
    /// Use simple output (print only urls)
    #[arg(short, long)]
    simple: bool,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Post {
    id: String,
    title: String,
    user_name: String,
    user_id: String,
}

impl Post {
    fn get_pixiv_post_url(&self) -> String {
        return format!("https://www.pixiv.net/en/artworks/{}", self.id);
    }

    fn get_pixiv_user_url(&self) -> String {
        return format!("https://www.pixiv.net/en/users/{}", self.user_id);
    }
}

#[derive(Deserialize, Debug)]
struct PostList {
    recent: Vec<Post>,
    permanent: Vec<Post>,
}

impl Display for Post {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if f.alternate() {
            println!("Post: {}", self.get_pixiv_post_url());
            println!("   -> Title: {}", self.title);
            println!(
                "   -> Posted by: {} ({})",
                self.user_name,
                self.get_pixiv_user_url()
            );
        } else {
            print!("{}", self.get_pixiv_post_url())
        }
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
    if !args.simple {
        println!("Sending request to Pixiv API...");
    }

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

    if args.simple {
        if args.permanent {
            print_posts(&typed_json.permanent, true)
        } else if args.recent {
            print_posts(&typed_json.recent, true)
        } else {
            print_posts(&typed_json.permanent, true);
            print_posts(&typed_json.recent, true)
        }
        return Ok(());
    }

    if args.permanent {
        println!("\n\nShowing permanently popular posts: ");
        print_posts(&typed_json.permanent, false);
    } else if args.recent {
        println!("\n\nShowing recently popular posts: ");
        print_posts(&typed_json.recent, false);
    } else {
        println!("\n\nShowing permanently popular posts: ");
        print_posts(&typed_json.permanent, false);
        println!("\n\nShowing recently popular posts: ");
        print_posts(&typed_json.recent, false);
    }

    println!("Took: {:.2?}", start.elapsed());
    Ok(())
}

fn print_posts(posts: &Vec<Post>, simple: bool) {
    for post in posts.iter() {
        if !simple {
            println!("{:#}\n", post);
        } else {
            println!("{}", post);
        }
    }
}
