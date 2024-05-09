mod error;
use std::net::TcpListener;

pub use crate::error::{Error, Result};

fn main() -> Result<()> {
    // Uncomment this block to pass the first stage

    let listener = TcpListener::bind("127.0.0.1:4221").expect("Could not bind tcp listener");

    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                println!("accepted new connection");
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
    Ok(())
}
