use std::{sync::Arc, time::Duration};

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};
use tokio::time::timeout;

impl MemDb {
    async fn blpop(&self, key: &str, time_arg: u64) -> DataType {
        match self.get(key) {
            DataType::List(list) => {
                if list.len() > 0 {
                    self.lpop(key, None)
                } else {
                    match timeout(Duration::from_secs(time_arg), self.wait_for(key)).await {
                        Ok(_) => self.lpop(key, None),
                        Err(e) => {
                            eprintln!("Timeout exceeded {e}");
                            DataType::NullArray
                        }
                    }
                }
            }
            _ => DataType::NullArray,
        }
    }
}

pub async fn blpop(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let time_arg: u64 = match &input_seq.get(2).unwrap_or(&DataType::Int(0)) {
        DataType::BulkString(bs) => bs.parse().unwrap(),
        DataType::Int(v) => *v as u64,
        _ => bail!("wrong timeout value type"),
    };

    Ok(mem_db.blpop(key, time_arg).await)
}
