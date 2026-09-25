use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn llen(&self, key: &str) -> DataType {
        let len = match self.get(key) {
            DataType::List(list) => list.len(),
            _ => 0,
        };

        DataType::UInt(len as u64)
    }
}

pub fn llen(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    Ok(mem_db.llen(key))
}

#[cfg(test)]
mod tests {
    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn get_len() {
        let db = MemDb::new();

        db.set("test", DataType::List(vec![DataType::Int(1)]), None);

        let len = db.llen("test");

        assert_eq!(len, DataType::UInt(1))
    }
}
