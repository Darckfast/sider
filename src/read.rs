use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use anyhow::Result;

#[derive(Debug)]
enum DataType {
    Str(String),
    Int(i64),
}

pub async fn read_stream(stream: &mut TcpStream) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();
    let mut con: Vec<DataType> = Vec::new();

    while reader.read_until(b'\n', &mut buf).await? > 0 {
        buf.truncate(buf.len() - 2);
        println!("Received {buf:?}");

        match &buf[0] {
            b'*' => {
                let raw_data = std::str::from_utf8(&buf[1..]).expect("should have string");
                let len: i64 = raw_data.parse().expect("should be i64");
                println!("array: len {len}");
            }
            b'$' => {
                let mut d_buf = Vec::new();
                reader.read_until(b'\n', &mut d_buf).await?;
                let val = std::str::from_utf8(&d_buf)?;
                con.push(DataType::Str(val.to_string()));
                println!("bulk string: {val}");
            }
            _ => {
                let val = std::str::from_utf8(&buf)?;
                eprintln!("{val}");
            }
        }

        buf.clear();
    }

    dbg!(con);
    Ok(())
}
