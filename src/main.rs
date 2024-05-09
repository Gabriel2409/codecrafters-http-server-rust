mod error;
use std::{io::Write, net::TcpListener};

pub use crate::error::{Error, Result};

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind tcp listener");

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                stream.write_all(b"HTTP/1.1 200 OK\r\n\r\n")?;
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
