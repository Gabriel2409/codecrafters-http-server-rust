mod error;
mod http;
use std::{
    io::{BufRead, BufReader, Read, Write},
    net::TcpListener,
};

use http::{HttpResponse, HttpStatus, HttpVersion};

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
                    "/" => HttpResponse {
                        status: HttpStatus::Ok200,
                        version: HttpVersion::V1_1,
                        headers: vec![],
                        body: None,
                    },
                    _ => HttpResponse {
                        status: HttpStatus::NotFound404,
                        version: HttpVersion::V1_1,
                        headers: vec![],
                        body: None,
                    },
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
