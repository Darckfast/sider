use std::{sync::Arc, time::Instant};

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn get(&self, key: &str) -> DataType {
        let mut map = self.map.lock().unwrap();
        let val = match map.get(key) {
            Some(v) => v.clone(),
            None => return DataType::NullStr,
        };

        let mut exp = self.expiry.lock().unwrap();

        let is_expired = exp.iter().find(|(k, v)| *k == *key && *v <= Instant::now());

        if let Some(_) = is_expired {
            let _ = map.remove(key);
            exp.retain(|(k, _)| k != key);

            DataType::NullStr
        } else {
            val
        }
    }
}

pub fn get(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    Ok(mem_db.get(key))
}

#[cfg(test)]
mod tests {
    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn get_data() {
        let state = MemDb::new();

        let val = state.get("test_1");

        assert_eq!(val, DataType::NullStr)
    }
}
