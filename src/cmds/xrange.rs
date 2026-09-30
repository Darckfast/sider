use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};

impl MemDb {
    fn xrange(&self, key: &str, start: &str, end: &str) -> DataType {
        if let DataType::Stream(s) = self.get(key) {
            let start: u128 = match start {
                "-" => 0,
                o => o.parse().unwrap_or(0),
            };
            let end: u128 = match end {
                "+" => match s.last() {
                    Some(v) => v.0.ms,
                    None => 0,
                },
                o => o.parse().unwrap_or(0),
            };

            let results = s
                .into_iter()
                .filter(|i| i.0.ms >= start && i.0.ms <= end)
                .collect();

            DataType::Stream(results)
        } else {
            DataType::NullArray
        }
    }
}

pub fn xrange(key: &str, args: &[DataType], db: Arc<MemDb>) -> DataType {
    let mut args = args.iter();
    if let (Some(DataType::BulkString(start)), Some(DataType::BulkString(end))) =
        (args.next(), args.next())
    {
        db.xrange(key, start, end)
    } else {
        DataType::NullArray
    }
}

#[cfg(test)]
mod tests {
    use std::{
        collections::HashMap,
        time::{SystemTime, UNIX_EPOCH},
    };

    use crate::{
        cmds::state::MemDb,
        read::{DataType, ID},
    };

    #[test]
    fn get_stream_range() {
        let db = MemDb::new();

        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("1-1".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );

        let ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let val = db.xrange("test", 0, ms);

        let mut expected: HashMap<String, DataType> = HashMap::new();
        expected.insert("a".to_string(), DataType::BulkString("true".to_string()));
        let id = ID { ms: 1, seq: 1 };
        assert_eq!(val, DataType::Stream(vec![(id, expected)]));
    }
}
