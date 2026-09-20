use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};
use anyhow::{Result, bail};

pub fn llen(input_seq: &[DataType], mem_db: MemDb, _exp_db: ExpDb) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let map = mem_db.lock().unwrap();

    if let Some(val) = map.get(key) {
        let len = match val {
            DataType::List(l) => l.len(),
            _ => 0,
        };

        Ok(DataType::Int(len as i64))
    } else {
        Ok(DataType::NullStr)
    }
}
