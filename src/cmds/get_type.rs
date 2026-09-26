use std::sync::Arc;

use anyhow::{Result, bail};

use crate::{cmds::state::MemDb, read::DataType};

impl MemDb {
    fn get_type(&self, key: &str) -> DataType {
        let val = match self.get(key) {
            DataType::BulkString(_) => "string",
            DataType::Stream(_) => "stream",
            DataType::NullStr => "none",
            _ => "unknown",
        }
        .to_string();

        DataType::SimpleStr(val)
    }
}

pub fn get_type(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    Ok(mem_db.get_type(key))
}
