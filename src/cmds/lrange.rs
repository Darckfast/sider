use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn lrange(&self, key: &str, start: i64, end: i64) -> DataType {
        let map = self.map.lock().unwrap();
        match map.get(key) {
            Some(e) => match e {
                DataType::List(list) => {
                    if start > end {
                        DataType::EmptyList
                    } else {
                        DataType::List(list[start as usize..=end as usize].to_vec())
                    }
                }
                _ => DataType::EmptyList,
            },
            None => DataType::EmptyList,
        }
    }
}

pub fn lrange(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let mut position_iter = input_seq[2..].iter().map(|v| match v {
        DataType::Int(v) => *v,
        DataType::BulkString(bs) => bs.parse().unwrap(),
        _ => 0,
    });

    let (start, end) = match (position_iter.next(), position_iter.next()) {
        (Some(s), Some(e)) => (s, e),
        _ => bail!("Missing params - expected Start and End"),
    };

    let vals = mem_db.lrange(key, start, end);

    Ok(vals)
}

#[cfg(test)]
mod tests {
    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn get_range() {
        let db = MemDb::new();

        db.set(
            "test",
            DataType::List(vec![
                DataType::Int(1),
                DataType::BulkString("My-String".to_string()),
                DataType::Int(2),
            ]),
            None,
        );

        let range = db.lrange("test", 0, 1);

        assert_eq!(
            range,
            DataType::List(vec![
                DataType::Int(1),
                DataType::BulkString("My-String".to_string())
            ])
        )
    }

    #[test]
    fn get_range_with_start_bigger_than_end() {
        let db = MemDb::new();

        db.set(
            "test",
            DataType::List(vec![
                DataType::Int(1),
                DataType::BulkString("My-String".to_string()),
                DataType::Int(2),
            ]),
            None,
        );

        let range = db.lrange("test", 1, 0);

        assert_eq!(range, DataType::EmptyList)
    }

    #[test]
    fn get_range_with_empty_list() {
        let db = MemDb::new();

        db.set("test", DataType::List(vec![]), None);

        let range = db.lrange("test", 1, 0);

        assert_eq!(range, DataType::EmptyList)
    }
}
