use conf::Config;
use github::GitHubEvent;

use std::fs::File;
use std::io;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use std::sync::Arc;

use warp::{Filter, http::Response};

//
// Routes
//

fn handle_page_request(conf: Arc<Config>, path: String) -> Response<impl Into<hyper::Body>> {
    // TODO add full path checking
    let full_path = conf.site_root.join(path);
    let ext = full_path.extension().unwrap();
    let start = Instant::now();

    // TODO instead of checking against paths here, do a full walk ahead of time and check against
    // that list. Should be more secure.

    // if the file exists return it directly
    if full_path.exists() && full_path.is_file() {
        let mut buf = Vec::new(); // none of my files are currently large enough to worry about this
        File::open(&full_path).unwrap().read_to_end(&mut buf).unwrap();

        return Response::builder()
            .header("application-type", "*/*") // TODO
            .body(buf);
    }

    if ext == "html" {
        let full_path = full_path.with_extension("md");
        if full_path.exists() && full_path.is_file() {
            let rendered = get_md_as_html(&full_path).unwrap();
            // eww, there has to be a better way to handle this
            let title = full_path.file_stem().map_or("thread.run", |s| s.to_str().unwrap());

            let generated = pretend_template(title, &rendered, start.elapsed());
            return Response::builder()
                .header("application-type", "text/html")
                .body(generated);
        }
    }

    Response::builder()
        .header("application-type", "text")
        .status(http::StatusCode::NOT_FOUND)
        .body("Could not find file")
}


fn git_webhook(conf: Config, event: GitHubEvent) {
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

#[tokio::main]
async fn main() {
    // put config into an "any" route to be combined with other routes via "and"
    let config = Config::from_file(std::env::args());
    let config = warp::any().map(move || Arc::new(config));

    // GET /
    let root_redirect = 
        warp::path::end().map(|| warp::redirect(warp::http::Uri::from_static("/index.html")));

    // GET /path...
    let pages = warp::any().and(config.clone()).and(warp::path::param::<String>()).map(handle_page_request);
    
    // POST /webhooks/github
    // TODO add restriction for application/json
    let github_route = warp::path::path("/webhooks/github").and(config.clone()).and(warp::body::content_length_limit(1024 * 16)).and(warp::body::json()).map(git_webhook);

    // restrict routes to their expected methods and combine them
    let routes = warp::get().and(root_redirect.or(pages)).or(warp::post().and(github_route));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}
