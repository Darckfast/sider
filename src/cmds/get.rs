use std::time::Instant;

use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};
use anyhow::{Result, bail};

pub fn get(input_seq: &[DataType], mem_db: MemDb, exp_db: ExpDb) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let mut map = mem_db.lock().unwrap();
    if let Some(v) = input_seq.get(2) {
        map.insert(key.to_string(), v.clone());
    }

    let val = match map.get(key) {
        Some(v) => v.clone(),
        None => DataType::NullStr,
    };

    let mut exp = exp_db.lock().unwrap();
    let is_expired = exp.iter().find(|(k, v)| *k == *key && *v >= Instant::now());

    // passive
    if let Some(_) = is_expired {
        let _ = map.remove(key);
        exp.retain(|(k, _)| k != key);

        Ok(DataType::NullStr)
    } else {
        Ok(val)
    }
}
