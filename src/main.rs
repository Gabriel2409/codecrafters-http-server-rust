mod error;
mod http;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use http::{HttpBody, HttpHeader, HttpResponse, HttpStatus, HttpVersion};

pub use crate::error::{Error, Result};
use crate::http::HttpRequest;

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind tcp listener");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut reader = BufReader::new(stream);
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
                    _ => HttpResponse::not_found_response(),
                };

                stream = reader.into_inner();
                stream.write_all(String::from(http_response).as_bytes())?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
