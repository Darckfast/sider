use std::sync::Arc;

use crate::{
    cmds::state::MemDb,
    read::{DataType, ID, Seq},
};

impl MemDb {
    fn xread(&self, key: &str, start: u128) -> DataType {
        if let DataType::Stream(s) = self.get(key) {
            let results = s
                .into_iter()
                .filter(|i| match &i.0 {
                    ID::Sequence(s) => s.ms > start,
                    _ => false,
                })
                .collect();

            DataType::Stream(results)
        } else {
            DataType::NullArray
        }
    }
}

pub fn xread(_key: &str, args: &[DataType], db: Arc<MemDb>) -> DataType {
    let mid = (args.len() - 1) / 2;
    let op = args[0].clone();
    let (keys, ids) = args[1..].split_at(mid);

    let mut results: Vec<(ID, DataType)> = Vec::new();
    if let DataType::BulkString(op) = op {
        match op.as_str() {
            "STREAMS" => {
                for (key, id) in keys.iter().zip(ids.iter()) {
                    if let DataType::BulkString(id) = id
                        && let DataType::BulkString(key) = key
                    {
                        let id = Seq::from_id_str(id);
                        let data = db.xread(key, id.ms);
                        results.push((ID::Str(key.clone()), data));
                    }
                }
            }
            "BLOCK" => (),
            op => eprintln!("un-mapped XREAD operator {op}"),
        }
    }

    DataType::Stream(results)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        cmds::{state::MemDb, xread::xread},
        read::{DataType, ID, Seq},
    };

    #[test]
    fn read_single_stream() {
        let db = Arc::new(MemDb::new());

        let data = vec![
            (
                ID::Sequence(Seq {
                    ms: 1526985054069,
                    seq: 0,
                }),
                DataType::KV(hashmap! {
                    "temperature".to_string() => DataType::BulkString("36".to_string()),
                    "humidity".to_string() => DataType::BulkString("95".to_string()),
                }),
            ),
            (
                ID::Sequence(Seq {
                    ms: 1526985054079,
                    seq: 0,
                }),
                DataType::KV(hashmap! {
                    "temperature".to_string() => DataType::BulkString("34".to_string()),
                    "humidity".to_string() => DataType::BulkString("97".to_string()),
                }),
            ),
        ];

        db.set("some_key", DataType::Stream(data), None);

        let results = xread(
            "some_key",
            &[
                DataType::BulkString("STREAMS".to_string()),
                DataType::BulkString("some_key".to_string()),
                DataType::BulkString("1526985054069-0".to_string()),
            ],
            db,
        );

        assert_eq!(
            results,
            DataType::Stream(vec![(
                ID::Str("some_key".to_string()),
                DataType::Stream(vec![(
                    ID::Sequence(Seq {
                        ms: 1526985054079,
                        seq: 0
                    }),
                    DataType::KV(hashmap! {
                        "temperature".to_string() => DataType::BulkString("34".to_string()),
                        "humidity".to_string() => DataType::BulkString("97".to_string()),
                    }),
                )])
            )])
        );
    }

    #[test]
    fn read_multiple_streams() {
        let db = Arc::new(MemDb::new());

        db.set(
            "some_key",
            DataType::Stream(vec![(
                ID::Sequence(Seq {
                    ms: 1526985054069,
                    seq: 0,
                }),
                DataType::KV(hashmap! {
                    "temperature".to_string() => DataType::BulkString("36".to_string()),
                }),
            )]),
            None,
        );

        db.set(
            "other_key",
            DataType::Stream(vec![(
                ID::Sequence(Seq {
                    ms: 1526985054079,
                    seq: 0,
                }),
                DataType::KV(hashmap! {
                    "humidity".to_string() => DataType::BulkString("97".to_string()),
                }),
            )]),
            None,
        );

        let results = xread(
            "",
            &[
                DataType::BulkString("STREAMS".to_string()),
                DataType::BulkString("some_key".to_string()),
                DataType::BulkString("other_key".to_string()),
                DataType::BulkString("1526985054068-0".to_string()),
                DataType::BulkString("1526985054078-0".to_string()),
            ],
            db,
        );

        assert_eq!(
            results,
            DataType::Stream(vec![
                (
                    ID::Str("some_key".to_string()),
                    DataType::Stream(vec![(
                        ID::Sequence(Seq {
                            ms: 1526985054069,
                            seq: 0
                        }),
                        DataType::KV(hashmap! {
                            "temperature".to_string() => DataType::BulkString("36".to_string()),
                        }),
                    )])
                ),
                (
                    ID::Str("other_key".to_string()),
                    DataType::Stream(vec![(
                        ID::Sequence(Seq {
                            ms: 1526985054079,
                            seq: 0
                        }),
                        DataType::KV(hashmap! {
                            "humidity".to_string() => DataType::BulkString("97".to_string()),
                        }),
                    )])
                ),
            ])
        );
    }
}
