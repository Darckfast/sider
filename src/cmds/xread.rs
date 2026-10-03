use core::time;
use std::{sync::Arc, time::Duration};

use tokio::time::timeout;

use crate::{
    cmds::state::MemDb,
    read::{DataType, ID, Seq},
};

impl MemDb {
    async fn xread(&self, key: &str, start: Seq, time_arg: Option<u64>) -> DataType {
        match time_arg {
            Some(ts) => {
                match timeout(
                    Duration::from_millis(ts),
                    self.wait_for(key, Some(start.clone())),
                )
                .await
                {
                    Ok(_) => (),
                    Err(e) => {
                        eprintln!("Timeout exceeded {ts}ms {e}");
                    }
                }
            }
            _ => (),
        }

        if let DataType::Stream(s) = self.get(key) {
            let results: Vec<(ID, DataType)> = s
                .into_iter()
                .filter(|(id, _)| match &id {
                    ID::Sequence(s) => {
                        if s.ms == start.ms {
                            s.seq > start.seq
                        } else {
                            s.ms > start.ms
                        }
                    }
                    _ => false,
                })
                .collect();
            DataType::Stream(results)
        } else {
            DataType::NullArray
        }
    }
}

pub async fn xread(_key: &str, args: &[DataType], db: Arc<MemDb>) -> DataType {
    let op = args[0].clone();
    let (keys, ids, time_arg) = {
        match op {
            DataType::BulkString(bs) if bs == "BLOCK".to_string() => {
                let time_arg = if let DataType::BulkString(bs) = args[1].clone() {
                    bs.parse().unwrap_or(0)
                } else {
                    0
                };
                let mid = (args.len() - 2) / 2;
                let (f, s) = args[3..].split_at(mid);

                (f, s, Some(time_arg))
            }
            _ => {
                let mid = (args.len() - 1) / 2;
                let (f, s) = args[1..].split_at(mid);

                (f, s, None)
            }
        }
    };

    let mut results: Vec<(ID, DataType)> = Vec::new();
    for (key, id) in keys.iter().zip(ids.iter()) {
        if let DataType::BulkString(id) = id
            && let DataType::BulkString(key) = key
        {
            let id = Seq::from_id_str(id);
            let data = db.xread(key, id, time_arg).await;

            results.push((ID::Str(key.to_owned()), data));
        }
    }

    DataType::Stream(results)
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use tokio::time::sleep;

    use crate::{
        cmds::{state::MemDb, xread::xread},
        read::{DataType, ID, Seq},
    };

    #[tokio::test]
    async fn read_single_stream() {
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
        )
        .await;

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

    #[tokio::test]
    async fn read_multiple_streams() {
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
        )
        .await;

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

    #[tokio::test]
    async fn read_stream_blocking() {
        let db = Arc::new(MemDb::new());

        let db_1 = Arc::clone(&db);
        tokio::spawn(async move {
            sleep(Duration::from_millis(10)).await;

            db_1.set(
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
        });

        let results = xread(
            "",
            &[
                DataType::BulkString("BLOCK".to_string()),
                DataType::BulkString("100".to_string()),
                DataType::BulkString("STREAMS".to_string()),
                DataType::BulkString("some_key".to_string()),
                DataType::BulkString("1526985054068-0".to_string()),
            ],
            db,
        )
        .await;

        assert_eq!(
            results,
            DataType::Stream(vec![(
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
            ),])
        );
    }
}
