use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};

use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    SimpleStr(String),
    BulkString(String),
    Int(i64),
    NullStr,
    List(Vec<DataType>),
}

pub async fn read_stream<Reader>(stream: Reader) -> Result<Vec<DataType>>
where
    Reader: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    let mut con: Vec<DataType> = Vec::new();
    let mut expected: usize = 0;
    let mut count: usize = 0;

    loop {
        buf.clear();
        let br = reader.read_line(&mut buf).await?;
        if br == 0 {
            break;
        }

        buf = buf.trim().to_string();
        if buf.len() == 0 {
            break;
        }

        match buf.chars().nth(0) {
            Some(ch) => match ch {
                '*' => {
                    let data = &buf[1..].to_string();
                    let len: usize = data.parse().expect("should have len");
                    expected += len;
                    continue;
                }
                '$' => {
                    let data = &buf[1..].to_string();
                    let len: usize = data.parse().expect("should have len");
                    let mut data_buf = vec![0u8; len];

                    reader.read_exact(&mut data_buf).await?;
                    let val = std::str::from_utf8(&data_buf)?;

                    con.push(DataType::BulkString(val.to_string()));

                    let mut trash = [0u8; 2];
                    reader.read_exact(&mut trash).await?;

                    count += 1;
                }
                _ => {
                    eprintln!("unmapped: {buf}");
                }
            },
            None => (),
        }

        if count == expected {
            break;
        }
    }

    Ok(con)
}
