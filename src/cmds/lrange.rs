use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn lrange(&self, key: &str, start: i64, end: i64) -> DataType {
        let map = self.map.lock().unwrap();
        match map.get(key) {
            Some(e) => match e {
                DataType::List(list) => {
                    if start > end {
                        DataType::EmptyList
                    } else {
                        DataType::List(list[start as usize..=end as usize].to_vec())
                    }
                }
                _ => DataType::EmptyList,
            },
            None => DataType::EmptyList,
        }
    }
}

pub fn lrange(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let map = mem_db.map.lock().unwrap();
    let vals = match map.get(key) {
        Some(e) => match e {
            DataType::List(l) => {
                let mut idxs: Vec<i64> = input_seq[2..]
                    .iter()
                    .map(|v| match v {
                        DataType::Int(v) => *v,
                        DataType::BulkString(bs) => bs.parse().unwrap(),
                        _ => 0,
                    })
                    .collect();

                if idxs[1] > l.len() as i64 - 1 {
                    idxs[1] = l.len() as i64 - 1;
                }

                if idxs[0] < 0 {
                    idxs[0] += l.len() as i64;
                }

                if idxs[1] < 0 {
                    idxs[1] += l.len() as i64;
                }

                if idxs[0] > idxs[1] {
                    DataType::List(vec![])
                } else {
                    DataType::List(l[idxs[0] as usize..=idxs[1] as usize].to_vec())
                }
            }
            _ => DataType::List(vec![]),
        },
        None => DataType::List(vec![]),
    };

    Ok(vals)
}
