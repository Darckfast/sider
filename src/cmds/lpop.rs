use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn lpop(&self, key: &str, total: Option<usize>) -> DataType {
        let mut map = self.map.lock().unwrap();

        if let Some(val) = map.get_mut(key) {
            let el = match val {
                DataType::List(list) if let Some(num) = total => {
                    Some(DataType::List(list.split_off(list.len() - num)))
                }
                DataType::List(list) => list.pop(),
                _ => None,
            };

            el.unwrap_or(DataType::NullStr)
        } else {
            DataType::NullStr
        }
    }
}

pub fn lpop(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };
    let total = if let Some(DataType::BulkString(num)) = input_seq.get(2) {
        let num: usize = num.parse().unwrap();
        Some(num)
    } else {
        None
    };

    Ok(mem_db.lpop(key, total))
}
