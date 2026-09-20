use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};
use anyhow::{Result, bail};

pub fn lpush(input_seq: &[DataType], mem_db: MemDb, _exp_db: ExpDb) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let mut map = mem_db.lock().unwrap();
    let mut indata = input_seq[2..].to_vec();
    indata.reverse();

    let entry = map
        .entry(key.to_string())
        .and_modify(|e| match e {
            DataType::List(l) => {
                l.splice(..0, indata.clone());
            }
            _ => (),
        })
        .or_insert(DataType::List(indata));

    if let DataType::List(l) = entry {
        Ok(DataType::Int(l.len() as i64))
    } else {
        Ok(DataType::NullStr)
    }
}
