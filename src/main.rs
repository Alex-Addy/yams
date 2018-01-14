#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate comrak;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::env;

use rocket::response::content::Content;
use rocket::response::Redirect;
use rocket::http::ContentType;
use rocket::State;

#[get("/")]
fn root() -> Redirect {
    Redirect::to("/index.html")
}

#[get("/<path..>")]
fn pages(path: PathBuf, site_root: State<PathBuf>) -> io::Result<Content<String>> {
    let full_path = site_root.join(path);
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

fn rocket(site_root: PathBuf) -> rocket::Rocket {
    rocket::ignite()
        .manage(site_root)
        .mount("/", routes![root, pages])
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage: {} <site_root>", args[0]);
        drop(args);
        std::process::exit(1);
    }
    rocket(PathBuf::from(&args[1])).launch();
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

