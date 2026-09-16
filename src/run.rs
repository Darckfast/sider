#![allow(unused_imports)]
use std::net::TcpListener;

use crate::ping::read_stream;

pub fn listen() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment the code below to pass the first stage
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                read_stream(stream).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
