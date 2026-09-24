use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn lpush(&self, key: &str, val: &mut [DataType]) -> DataType {
        val.reverse();

        let notifier = self.notify_for(&key);
        let mut map = self.map.lock().unwrap();

        let entry = map
            .entry(key.to_string())
            .and_modify(|e| match e {
                DataType::List(l) => {
                    l.splice(..0, val.to_vec());
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

pub fn lpush(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let mut val = input_seq[2..].to_vec();

    Ok(mem_db.lpush(key, &mut val))
}

#[cfg(test)]
mod tests {
    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn insert_elements() {
        let state = MemDb::new();

        let len = state.lpush("test-1", &mut [DataType::Int(1), DataType::Int(2)]);

        assert_eq!(len, DataType::Int(2))
    }
}
