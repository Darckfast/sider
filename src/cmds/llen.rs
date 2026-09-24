use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

pub fn llen(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let len = match mem_db.get(key) {
        DataType::List(l) => l.len(),
        _ => 0,
    };

    Ok(DataType::Int(len as i64))
}
