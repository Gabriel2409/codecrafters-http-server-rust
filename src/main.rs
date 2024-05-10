mod error;
mod http;
mod threadpool;
use std::{
    io::{BufReader, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
};

use http::{HttpMethod, HttpResponse};
use threadpool::ThreadPool;

pub use crate::error::{Error, Result};
use crate::http::HttpRequest;

fn handle_connection(mut stream: TcpStream, directory: &str) -> Result<()> {
    let mut reader = BufReader::new(stream);
    // TODO: extract error and map it to a http response
    let http_request = HttpRequest::try_from(&mut reader)?;

    let http_response = match http_request.path.as_ref() {
        "/" => HttpResponse::empty_response(),
        x if x.starts_with("/echo/") => {
            let echo = &x[6..];
            HttpResponse::plain_text_response(echo)
        }
        "/user-agent" => {
            let mut user_agent = None;
            for header in http_request.headers {
                if header.key == "User-Agent" {
                    user_agent = Some(header.value);
                    break;
                }
            }
            match user_agent {
                None => HttpResponse::not_found_response(),
                Some(user_agent) => HttpResponse::plain_text_response(&user_agent),
            }
        }
        x if x.starts_with("/files/") => {
            let filename = &x[7..];

            let filepath = PathBuf::from(&format!("{}/{}", directory, filename));

            match http_request.method {
                HttpMethod::Get => match filepath.exists() {
                    true => {
                        let content = std::fs::read_to_string(filepath).expect("File should exist");
                        HttpResponse::file_response(&content)
                    }
                    false => HttpResponse::not_found_response(),
                },
            }
        }

        _ => HttpResponse::not_found_response(),
    };

    stream = reader.into_inner();
    stream.write_all(String::from(http_response).as_bytes())?;
    Ok(())
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind tcp listener");

    let pool = ThreadPool::build(4)?;

    let mut directory = String::from("");
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 3 && args[1] == "--directory" {
        directory = args[2].to_string();
    }
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
