use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

use anyhow::Result;

pub async fn read_stream(stream: &mut TcpStream) -> Result<()> {
    let mut reader = BufReader::new(stream);
    let mut buf = Vec::new();

    while reader.read_until(b'\n', &mut buf).await? > 0 {
        println!("Received {buf:?}");

        let cmd = std::str::from_utf8(&buf)?;
        match &cmd {
            _o => {
                reader.get_mut().write_all(b"+PONG\r\n").await?;
            }
        }

        buf.clear();
    }

    Ok(())
}
