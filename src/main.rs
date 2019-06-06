use url;
use futures::future;
use hyper::{Body, Request, Response, Server, StatusCode, Method, header};
use serde::Serialize;
use hyper::rt::{self, Future};
use hyper::service::service_fn;
use chrono_tz;
use chrono::{DateTime, TimeZone};
use std::collections::HashMap;
use when::{self, DateTimeError};

#[derive(Serialize, Debug)]
struct ParsedQuery {
    source_str: String,
    merge_dist: usize,
    timezone: String,
    exact_match: bool,
}

#[derive(Serialize, Debug)]
struct ServerResponse {

    #[serde(flatten)]
    parsed_args: ParsedQuery,

    result: Result<Vec<String>, DateTimeError>,
}

fn prepare_result<Tz: TimeZone>(parsed: Vec<Result<DateTime<Tz>, DateTimeError>>, into_unix_ts: bool)
    -> Result<Vec<String>, DateTimeError> {

    let mut result = Vec::new();

    for item in parsed {
        let x = match item {
            Ok(x) => {
                if into_unix_ts {
                    format!("{}", x.timestamp())
                } else {
                    format!("{:?}", x)
                }
            },
            Err(err) => return Err(err),
        };
        result.push(x);
    }

    Ok(result)
}

fn str2bool(input: &str) -> bool {
    let input = input.to_lowercase();
    if input == "false" || input == "0" || input == "f" {
        return false
    }
    true
}

fn parse_timezone(tz: &str) -> chrono_tz::Tz {
    let tz: chrono_tz::Tz = tz.parse().unwrap();
    tz
}


fn do_parse(hash_query: HashMap<String, String>, into_unix_ts: bool) -> (ParsedQuery,
                                                     Result<Vec<std::string::String>, DateTimeError>) {
    let default_tz = "Europe/Moscow".to_owned();

    // parse get arguments
    let input_str = hash_query
        .get("input")
        .unwrap();

    let tz_str = hash_query
        .get("tz")
        .unwrap_or(&default_tz);

    let timezone = parse_timezone(tz_str);

    let exact_match = str2bool(hash_query
        .get("exact_match")
        .unwrap_or(&String::from("false")));

    let merge_dist = hash_query
        .get("dist")
        .unwrap_or(&String::from("5"))
        .parse::<usize>()
        .unwrap();

    let query = ParsedQuery {
        source_str: input_str.clone(),
        timezone: tz_str.clone(),
        exact_match,
        merge_dist,
    };

    let parser = when::parser::Parser::new(Box::new(when::en), timezone, merge_dist,
                                           exact_match);

    (query, prepare_result(parser.recognize(&input_str), into_unix_ts))

}

type BoxFut = Box<Future<Item = Response<Body>, Error = hyper::Error> + Send>;

fn handler(req: Request<Body>) -> BoxFut {

    let mut response = Response::new(Body::empty());

    match (req.method(), req.uri().path()) {
        // Parse natural language time/date at /get
        (&Method::GET, "/get") => {
            let parsed_url = url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes());
            let hash_query: HashMap<_, _> = parsed_url.into_owned().collect();

            let (query_params, result) = do_parse(hash_query, false);

            let resp = serde_json::to_string(&ServerResponse {
                parsed_args: query_params,
                result,
            }).unwrap();

            response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(resp))
                .unwrap();

        }

        (&Method::GET, "/get_unix") => {
            let parsed_url = url::form_urlencoded::parse(req.uri().query().unwrap().as_bytes());
            let hash_query: HashMap<_, _> = parsed_url.into_owned().collect();

            let (query_params, result) = do_parse(hash_query, true);

            let resp = serde_json::to_string(&ServerResponse {
                parsed_args: query_params,
                result,
            }).unwrap();

            response = Response::builder()
                .status(StatusCode::OK)
                .header(header::CONTENT_TYPE, "application/json")
                .header(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*")
                .body(Body::from(resp))
                .unwrap();

        }

        // The 404 Not Found route...
        _ => {
            *response.status_mut() = StatusCode::NOT_FOUND;
        }
    }

    Box::new(future::ok(response))
}

fn main() {
    pretty_env_logger::init();
    let addr = ([0, 0, 0, 0], 3000).into();

    let server = Server::bind(&addr)
        .serve(|| service_fn(handler))
        .map_err(|e| eprintln!("server error: {}", e));

    println!("Listening on http://{}", addr);
    rt::run(server);
}
