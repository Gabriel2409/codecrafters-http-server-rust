mod error;
mod http;
mod threadpool;
use std::{
    collections::HashMap,
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    path::PathBuf,
};

use http::{HttpBody, HttpMethod, HttpResponse, HttpStatus};
use threadpool::ThreadPool;

pub use crate::error::{Error, Result};
use crate::http::HttpRequest;

fn handle_connection(mut stream: TcpStream, directory: &str) -> Result<()> {
    let mut reader = BufReader::new(stream);

    // TODO: extract error and map it to a http response
    let http_request = HttpRequest::try_from(&mut reader)?;

    let mut header_map = HashMap::new();

    for header in http_request.headers.iter() {
        header_map.insert(header.key.to_lowercase(), header.value.clone());
    }

    let accepted_encodings = header_map
        .get("accept-encoding")
        .map(|s| s.to_owned())
        .unwrap_or("None".to_owned());

    let http_response = match http_request.path.as_ref() {
        "/" => Ok(HttpResponse::empty_response(HttpStatus::Ok200)),
        x if x.starts_with("/echo/") => {
            let echo = &x[6..];
            HttpResponse::content_response(echo, "text/plain", &accepted_encodings)
        }
        "/user-agent" => match header_map.get("user-agent") {
            None => Ok(HttpResponse::empty_response(HttpStatus::NotFound404)),
            Some(user_agent) => {
                HttpResponse::content_response(user_agent, "text/plain", &accepted_encodings)
            }
        },
        x if x.starts_with("/files/") => {
            let filename = &x[7..];

            let filepath = PathBuf::from(&format!("{}/{}", directory, filename));

            match http_request.method {
                HttpMethod::Get => match filepath.exists() {
                    true => {
                        let content = std::fs::read_to_string(filepath).expect("File should exist");
                        HttpResponse::content_response(
                            &content,
                            "application/octet-stream",
                            &accepted_encodings,
                        )
                    }
                    false => Ok(HttpResponse::empty_response(HttpStatus::NotFound404)),
                },
                HttpMethod::Post => {
                    let dirpath = filepath.parent().expect("Directory should not be none");
                    match dirpath.exists() {
                        true => {
                            let body = http_request.body.expect("POST request should have a body");

                            match body {
                                HttpBody::Text(body) => {
                                    std::fs::write(filepath, body)?;

                                    Ok(HttpResponse::empty_response(HttpStatus::Created201))
                                }
                                _ => todo!(),
                            }
                        }

                        false => Ok(HttpResponse::empty_response(HttpStatus::NotFound404)),
                    }
                }
            }
        }

        _ => Ok(HttpResponse::empty_response(HttpStatus::NotFound404)),
    }
    .unwrap_or(HttpResponse::empty_response(
        HttpStatus::InternalServerError500,
    ));

    stream = reader.into_inner();
    let res: Vec<u8> = http_response.into();
    stream.write_all(&res)?;
    Ok(())
}

fn main() -> Result<()> {
    // NOTE: bind actually behaves bind and listen from the socket api
    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind tcp listener");

    let pool = ThreadPool::build(4)?;

    let mut directory = String::from("");
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "--directory" {
        directory = args[2].to_string();
    }
    //NOTE:similar to accept from the socket api

    // for stream in listener.incoming().take(5) { // disconnects after 5 requests
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let directory = directory.clone();
                // TODO: handle this and return 400 or 500 instead.
                pool.execute(move || {
                    handle_connection(stream, &directory).expect("Could not build response")
                });
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
