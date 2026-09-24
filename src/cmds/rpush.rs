use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn rpush(&self, key: &str, val: &[DataType]) -> DataType {
        let notifier = self.notify_for(&key);
        let mut map = self.map.lock().unwrap();

        let entry = map
            .entry(key.to_string())
            .and_modify(|e| match e {
                DataType::List(l) => {
                    l.append(&mut val.to_vec());
                }
                _ => (),
            })
            .or_insert(DataType::List(val.to_vec()));

        notifier.notify_waiters();

        if let DataType::List(l) = entry {
            DataType::Int(l.len() as i64)
        } else {
            DataType::NullStr
        }
    }
}

pub fn rpush(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let val = &input_seq[2..];
    Ok(mem_db.rpush(key, val))
}

#[cfg(test)]
mod tests {
    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn insert_elements() {
        let state = MemDb::new();

        let len = state.rpush(
            "test_1",
            &[DataType::Int(1), DataType::BulkString("test".to_string())],
        );

        assert_eq!(len, DataType::Int(2))
    }

    #[test]
    fn append_elements() {
        let state = MemDb::new();

        let _ = state.rpush(
            "test_1",
            &[DataType::Int(1), DataType::BulkString("test".to_string())],
        );

        let len = state.rpush(
            "test_1",
            &[DataType::Int(1), DataType::BulkString("test".to_string())],
        );

        assert_eq!(len, DataType::Int(4))
    }
}
