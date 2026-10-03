use std::{collections::HashMap, sync::Arc};

use crate::{
    cmds::state::MemDb,
    read::{DataType, ID, Seq},
};

impl MemDb {
    pub fn xadd(&self, key: &str, args: &[DataType]) -> DataType {
        let mut db = self.map.lock().unwrap();

        let val = db
            .entry(key.to_string())
            .or_insert(DataType::Stream(Vec::new()));

        let id = match args.first() {
            Some(arg) => match arg {
                DataType::BulkString(bs) => {
                    let mut parts = bs.split("-");
                    let (ms, seq) = (parts.next(), parts.next());

                    match ms {
                        Some(ms)
                            if let Some(seq) = seq
                                && ms != "*"
                                && seq != "*" =>
                        {
                            let ms: u128 = ms.parse().unwrap_or(0);
                            let seq: u8 = seq.parse().unwrap_or(0);

                            if ms == 0 && seq == 0 {
                                return DataType::Error(
                                    "ERR The ID specified in XADD must be greater than 0-0"
                                        .to_string(),
                                );
                            }
                            Seq { ms, seq }
                        }
                        Some(ms)
                            if let Some(seq) = seq
                                && seq == "*"
                                && ms != "*" =>
                        {
                            let ms: u128 = ms.parse().unwrap_or(0);
                            let mut id = Seq { ms, seq: 0 };

                            if ms == 0 {
                                id.seq = 1
                            }

                            if let DataType::Stream(val) = val {
                                let exists = val.iter().find(|(aid, _)| match aid {
                                    ID::Sequence(aid) => aid.ms == id.ms,
                                    _ => false,
                                });

                                if let Some((ID::Sequence(aid), _)) = exists {
                                    id.seq = aid.seq + 1;
                                }
                            }

                            id
                        }
                        _ => Seq::new(),
                    }
                }
                _ => Seq::new(),
            },
            None => Seq::new(),
        };

        if let DataType::Stream(val) = val {
            if let Some((ID::Sequence(last_id), _)) = val.last()
                && last_id.ms >= id.ms
            {
                return DataType::Error(
                    "ERR The ID specified in XADD is equal or smaller than the target stream top item"
                        .to_string(),
                );
            }

            let mut submap = HashMap::new();

            for arg in args[1..].chunks(2) {
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

            val.push((ID::Sequence(id.clone()), DataType::KV(submap)));
        }

        DataType::SimpleStr(format!("{id}"))
    }
}

pub fn xadd(key: &str, args: &[DataType], db: Arc<MemDb>) -> DataType {
    db.xadd(key, args)
}

#[cfg(test)]
mod tests {
    use crate::{
        cmds::state::MemDb,
        read::{DataType, ID, Seq},
    };

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

        let result = db.get("test");

        assert_eq!(id, DataType::SimpleStr("1-1".to_string()));
        assert_eq!(
            result,
            DataType::Stream(vec![(
                ID::Sequence(Seq { ms: 1, seq: 1 }),
                DataType::KV(hashmap! {
                    "a".to_string() => DataType::BulkString("true".to_string())
                })
            )])
        );
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
                "ERR The ID specified in XADD is equal or smaller than the target stream top item"
                    .to_string()
            )
        )
    }

    #[test]
    fn create_stream_gen_id_with_star_seq() {
        let db = MemDb::new();

        let id = db.xadd(
            "test",
            &[
                DataType::BulkString("1-*".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );

        assert_eq!(id, DataType::SimpleStr("1-0".to_string()))
    }

    #[test]
    fn create_stream_gen_id_with_star() {
        let db = MemDb::new();

        let id = db.xadd(
            "test",
            &[
                DataType::BulkString("*".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );

        assert_ne!(id, DataType::SimpleStr("1-0".to_string()))
    }
}
