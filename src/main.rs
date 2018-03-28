#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate comrak;
extern crate git2;
extern crate json;

mod git;
mod conf;
use conf::Config;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::env;

use rocket::response::content::Content;
use rocket::response::Redirect;
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
fn pages(path: PathBuf, conf: State<Config>) -> io::Result<Content<String>> {
    let full_path = conf.site_root.join(path);
    let ext = full_path.extension();

    if let Some(ext) = ext {
        if ext == "html" {
            let full_path = full_path.with_extension("md");
            let rendered = get_md_as_html(&full_path)?;
            // eww, there has to be a better way to handle this
            let title = full_path.file_stem().map_or("thread.run", |s| s.to_str().unwrap());
            let generated = pretend_template(title, &rendered);
            return Ok(Content(ContentType::HTML, generated));
        }
    }

    let mut contents = String::new();
    File::open(&full_path)?.read_to_string(&mut contents)?;

    let content_type = match ext.map_or(None, |s| s.to_str()) {
        // TODO handle images
        Some("css") => ContentType::CSS,
        // TODO load unknown files to a binary container so the default
        // can be non-text
        _ => ContentType::Plain,
    };

    Ok(Content(content_type, contents))
}

#[post("/webhooks/github", format = "application/json")]
fn git_webhook(conf: State<Config>) {
    let head = git::get_head_sha(&conf.site_root).expect("couldn't get HEAD sha");
    println!("{}", head);
    // TODO check hash against content
    git::pull(&conf.site_root, &conf.ssh).unwrap();
}

use comrak::{markdown_to_html, ComrakOptions};
fn get_md_as_html(path: &Path) -> io::Result<String> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;

    Ok(markdown_to_html(&contents, &ComrakOptions::default()))
}

fn pretend_template(title: &str, content: &str) -> String {
    // I would rather this be in a constant but format! requires a string literal?
    // TODO fix this
    format!(r#"
<html>
<head>
    <title>{title}</title>
    <meta charset="UTF-8">
    <link rel="stylesheet" type="text/css" href="/css/base.css">
</head>
<body>
    {body}
</body>
"#, title = title, body = content)
}


//
// Main Section
//

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .attach(AdHoc::on_attach(|rocket| {
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
