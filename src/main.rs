#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

use std::path::{Path, PathBuf};

use rocket::response::NamedFile;

#[get("/<path..>")]
fn pages(path: PathBuf) -> Option<NamedFile> {
    NamedFile::open(Path::new(SITE_ROOT).join(path)).ok()
}

fn rocket() -> rocket::Rocket {
    rocket::ignite().mount("/", routes![pages])
}

fn main() {
    rocket().launch();
}

const SITE_ROOT: &'static str = "../thread.run/";
