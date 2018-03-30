
use failure::{Error, err_msg};
use rocket::{Data, Outcome, Request};
use rocket::data::{FromData};
use rocket::http::Status;
use json;
use std::io::Read;

#[derive(Debug)]
pub struct GitHubEvent {
    pub event_type: String,
    pub delivery: String,

    // 'push' event fields
    pub full_ref: Option<String>, // 'ref'
    pub new_sha: Option<String>, // 'after' or 'head'

    // 'ping' event fields
    pub zen: Option<String>, // 'zen'
    pub hook_id: Option<usize>, // 'hook_id'
}

impl GitHubEvent {
    fn new(event_type: String, delivery: String) -> GitHubEvent {
        GitHubEvent {
            event_type: event_type,
            delivery: delivery,

            full_ref: None,
            new_sha: None,
            zen: None,
            hook_id: None,
        }
    }

    fn push(event_type: String, delivery: String, full_ref: String, new_sha: String) -> GitHubEvent {
        GitHubEvent {
            event_type: event_type,
            delivery: delivery,

            full_ref: Some(full_ref),
            new_sha: Some(new_sha),
            zen: None,
            hook_id: None,
        }
    }

    fn ping(event_type: String, delivery: String, zen: String, hook_id: usize) -> GitHubEvent {
        GitHubEvent {
            event_type: event_type,
            delivery: delivery,

            full_ref: None,
            new_sha: None,
            zen: Some(zen),
            hook_id: Some(hook_id),
        }
    }
}

impl FromData for GitHubEvent {
    type Error = Error;

    fn from_data(request: &Request, data: Data) -> Outcome<Self, (Status, Self::Error), Data> {
        let headers = request.headers();
        let ev_type = headers.get_one("X-GitHub-Event");
        let delivery = headers.get_one("X-GitHub-Delivery");
        // TODO handle event signing
        let _signature = request.headers().get_one("X-GitHub-Signature");

        if delivery.is_none() || ev_type.is_none() {
            return Outcome::Failure((Status::BadRequest,
                err_msg("missing delivery or event type headers")));
        }
        let delivery = delivery.unwrap();
        let ev_type = ev_type.unwrap();

        if ev_type != "ping" && ev_type != "push" {
            // event type is not handled, return general type
            return Outcome::Success(GitHubEvent::new(ev_type.to_string(), delivery.to_string()));
        }

        // github webhooks will never send more than 5MB
        let mut data = data.open().take(5 * 1024 * 1024);
        let mut contents = String::new();
        match data.read_to_string(&mut contents) {
            Ok(_) => {},
            Err(e) => return Outcome::Failure((Status::InternalServerError, Error::from(e))),
        };

        let mut parsed = match json::parse(&contents) {
            Ok(v) => v,
            Err(e) => return Outcome::Failure((Status::BadRequest, Error::from(e))),
        };

        if ev_type == "push" {
            let full_ref = parsed["ref"].take_string().expect("'push' event missing 'ref' field");
            // docs say 'head' but example has 'after
            let sha = parsed["head"].take_string().or(parsed["after"].take_string());
            if sha.is_none() {
                return Outcome::Failure((Status::BadRequest, err_msg("push event missing 'head' and 'after' fields")));
            }

            return Outcome::Success(GitHubEvent::push(ev_type.to_string(), delivery.to_string(), full_ref, sha.unwrap()));
        } else if ev_type == "ping" {
            let zen = parsed["zen"].take_string().expect("'ping' event missing 'zen' field");
            let hook_id = parsed["hook_id"].as_usize().expect("'ping' event missing 'hook_id' field");

            return Outcome::Success(GitHubEvent::ping(ev_type.to_string(), delivery.to_string(), zen, hook_id));
        } else {
            unreachable!();
        }
    }
}

