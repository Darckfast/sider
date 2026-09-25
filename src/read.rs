use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};

use anyhow::Result;

#[derive(Debug, Clone, PartialEq)]
pub enum DataType {
    SimpleStr(String),
    BulkString(String),
    Int(i64),
    UInt(u64),
    NullStr,
    NullArray,
    // Error(String),
    List(Vec<DataType>),
    EmptyList,
}

pub async fn read_stream<Reader>(stream: Reader) -> Result<Vec<DataType>>
where
    Reader: AsyncRead + Unpin,
{
    let mut reader = BufReader::new(stream);
    let mut buf = String::new();
    let mut con: Vec<DataType> = Vec::new();
    let mut expected: usize = 0;

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

        if let Some(ch) = buf.chars().nth(0) {
            match ch {
                '*' => {
                    let data = &buf[1..].to_string();
                    let len: usize = data
                        .parse()
                        .expect("buffer has invalid RESP data - expected length after *");
                    expected += len;
                }
                '$' => {
                    let data = &buf[1..].to_string();
                    let len: usize = data
                        .parse()
                        .expect("buffer has invalid RESP data - expect length after $");
                    let mut data_buf = vec![0u8; len];

                    reader.read_exact(&mut data_buf).await?;
                    let val = std::str::from_utf8(&data_buf)?;

                    con.push(DataType::BulkString(val.to_string()));

                    let mut trash = [0u8; 2];
                    reader.read_exact(&mut trash).await?;

                    expected -= 1;
                }
                o => {
                    eprintln!("unmapped operator: {o} {buf}");
                }
            }
        }

        if expected == 0 {
            break;
        }
    }

    Ok(con)
}
