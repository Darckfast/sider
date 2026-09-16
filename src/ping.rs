use std::{
    io::{Read, Write},
    net::TcpStream,
};

pub fn read_stream(mut stream: TcpStream) -> std::io::Result<()> {
    let mut buf = [0u8; 1024];

    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                eprintln!("Connection closed by the peer");
                break;
            }
            Ok(bs) => {
                let data = &buf[..bs];
                match data {
                    b"PING\n" => {
                        stream.write(b"+PONG\r\n").unwrap();
                    }
                    _o => {
                        stream.write(b"+PONG\r\n").unwrap();
                    }
                }
                println!("Received {bs} bytes: {data:?}");
            }
            Err(e) => {
                eprintln!("Error reading stream: {e}");
                return Err(e);
            }
        }
    }

    Ok(())
}
