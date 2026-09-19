use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};
use anyhow::{Result, bail};

pub fn rpush(input_seq: &[DataType], mem_db: MemDb, _exp_db: ExpDb) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let map = mem_db.lock().unwrap();
    let vals = match map.get(key) {
        Some(e) => match e {
            DataType::List(l) => {
                let idxs = input_seq[2..].iter().map(|v| match v {
                    DataType::Int(v) => *v,
                    DataType::BulkString(bs) => bs.parse().unwrap(),
                    _ => 0,
                });
                let mut vals: Vec<DataType> = Vec::new();
                for idx in idxs {
                    if let Some(v) = l.get(idx as usize) {
                        vals.push(v.clone());
                    }
                }

                DataType::List(vals)
            }
            _ => DataType::List(vec![]),
        },
        None => DataType::List(vec![]),
    };

    Ok(vals)
}
