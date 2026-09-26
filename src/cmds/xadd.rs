use std::{collections::HashMap, sync::Arc};

use crate::{
    cmds::state::MemDb,
    read::{DataType, ID},
};

impl MemDb {
    fn xadd(&self, key: &str, args: &[DataType]) -> DataType {
        let mut db = self.map.lock().unwrap();

        let id = match args.first() {
            Some(arg) => match arg {
                DataType::BulkString(bs) => {
                    let mut parts = bs.split("-");
                    let (ms, seq) = (parts.next(), parts.next());

                    if let Some(ms) = ms
                        && let Some(seq) = seq
                    {
                        let ms: u128 = ms.parse().unwrap_or(0);
                        let seq: u8 = seq.parse().unwrap_or(0);

                        ID { ms, seq }
                    } else {
                        ID::new()
                    }
                }
                _ => ID::new(),
            },
            None => ID::new(),
        };

        let val = db
            .entry(key.to_string())
            .or_insert(DataType::Stream(Vec::new()));

        if let DataType::Stream(val) = val {
            if let Some((last_id, _)) = val.last()
                && last_id.ms >= id.ms
            {
                return DataType::Error(
                    "ERR The ID specified in XADD is equal or smaller than the target stream top item"
                        .to_string(),
                );
            }

            let mut submap = HashMap::new();

            for arg in args.chunks(2) {
                match arg {
                    [key, value] => {
                        let key = match key {
                            DataType::BulkString(bs) => bs,
                            _ => "",
                        }
                        .to_string();

                        submap.insert(key, value.clone());
                    }
                    _ => (),
                }
            }

            val.push((id.clone(), submap));
        }

        DataType::SimpleStr(format!("{id}"))
    }
}

pub fn xadd(key: &str, args: &[DataType], db: Arc<MemDb>) -> DataType {
    db.xadd(key, args)
}

#[cfg(test)]
mod tests {
    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn add_stream() {
        let db = MemDb::new();

        let id = db.xadd(
            "test",
            &[
                DataType::BulkString("1-1".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );

        assert_eq!(id, DataType::SimpleStr("1-1".to_string()))
    }

    #[test]
    fn validate_id() {
        let db = MemDb::new();

        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("1-1".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );

        let err = db.xadd(
            "test",
            &[
                DataType::BulkString("0-1".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );

        assert_eq!(
            err,
            DataType::Error(
                "The ID specified in XADD is equal or smaller than the target stream top item"
                    .to_string()
            )
        )
    }
}
