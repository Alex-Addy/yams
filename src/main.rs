#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;

extern crate comrak;
extern crate git2;
extern crate json;
extern crate failure;
extern crate chrono;

mod git;
mod conf;
mod github;

use conf::Config;
use github::GitHubEvent;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::env;
use std::time::{Duration, Instant};

use rocket::response::{Content, Redirect};
use rocket::http::ContentType;
use rocket::State;
use rocket::fairing::AdHoc;

//
// Routes
//

#[get("/")]
fn root() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/<path..>")]
fn pages(path: PathBuf, conf: State<Config>) -> Option<Content<Vec<u8>>> {
    let full_path = conf.site_root.join(path);
    let ext = full_path.extension().unwrap();
    let content_type = ContentType::from_extension(ext.to_str().expect("extension is not valid utf-8"))
        .unwrap_or(ContentType::Binary);
    let start = Instant::now();

    // if the file exists return it directly
    if full_path.exists() && full_path.is_file() {
        let mut buf = Vec::new(); // none of my files are currently large enough to worry about this
        File::open(&full_path).unwrap().read_to_end(&mut buf).unwrap();

        return Some(Content(content_type, buf));
    }

    if ext == "html" {
        let full_path = full_path.with_extension("md");
        if full_path.exists() && full_path.is_file() {
            let rendered = get_md_as_html(&full_path).unwrap();
            // eww, there has to be a better way to handle this
            let title = full_path.file_stem().map_or("thread.run", |s| s.to_str().unwrap());

            let generated = pretend_template(title, &rendered, start.elapsed());
            return Some(Content(ContentType::HTML, Vec::from(generated)));
        }
    }

    None
}


#[post("/webhooks/github", format = "application/json", data = "<event>")]
fn git_webhook(conf: State<Config>, event: GitHubEvent) {
    println!("{:?}", event);
    match &*event.event_type {
        "ping" => return,
        "push" => {},
        t => {
            println!("Unknown event type '{}'", t);
            return;
        },
    }

    let head = git::get_head_sha(&conf.site_root).expect("couldn't get HEAD sha");
    if head == event.new_sha.unwrap() {
        println!("Head is already at {}", head);
        return;
    }
    if !event.full_ref.unwrap().contains("master") {
        println!("Push is for non-master branch, skipping");
        return;
    }

    git::pull(&conf.site_root, &conf.ssh).unwrap();
}

//
// Utility Functions
//

use comrak::{markdown_to_html, ComrakOptions};
fn get_md_as_html(path: &Path) -> io::Result<String> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;

    Ok(markdown_to_html(&contents, &ComrakOptions::default()))
}

fn pretend_template(title: &str, content: &str, duration: Duration) -> String {
    // I would rather this be in a constant but format! requires a string literal?
    // TODO fix this
    format!(r#"
<html>
<head>
    <title>{title}</title>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="/resources/styles.css">
</head>
<body>
    {body}
</body>
<!-- Page constructed in {duration}ms on {date}. -->
</html>
"#, title = title, body = content, duration = as_ms(&duration), date = chrono::Utc::now().to_rfc3339())
}

fn as_ms(d: &Duration) -> u64 {
    d.as_secs() * 1000 + (d.subsec_nanos() as u64 / 1_000_000)
}


//
// Main Section
//

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(AdHoc::on_attach("Load Config", |rocket| {
            let conf = conf::Config::from_rocket_conf(rocket.config());
            match conf {
                Ok(c) => {
                    println!("Extracted config: {:?}", c);
                    Ok(rocket.manage(c))
                },
                Err(e) => {
                    println!("Error extracing config: {:?}", e);
                    Err(rocket)
                },
            }
        }))
        .mount("/", routes![root, pages, git_webhook])
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 1 {
        println!("Usage: {}", args[0]);
        drop(args);
        std::process::exit(1);
    }
    rocket().launch();
}
