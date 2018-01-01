#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;
extern crate comrak;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};

#[get("/<path..>")]
fn pages(path: PathBuf) -> io::Result<String> {
    if let Some(ext) = path.extension() {
        if ext == "html" {
            return get_md_as_html(&Path::new(SITE_ROOT).join(path.with_extension("md")))
        }
    }

    let mut contents = String::new();
    File::open(Path::new(SITE_ROOT).join(path))?.read_to_string(&mut contents)?;
    Ok(contents)
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![pages])
}

fn main() {
    rocket().launch();
}

const SITE_ROOT: &'static str = "../thread.run/";

use comrak::{markdown_to_html, ComrakOptions};
fn get_md_as_html(path: &Path) -> io::Result<String> {
    let mut contents = String::new();
    File::open(path)?.read_to_string(&mut contents)?;

    Ok(markdown_to_html(&contents, &ComrakOptions::default()))
}
