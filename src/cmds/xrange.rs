use std::sync::Arc;

use crate::{
    cmds::state::MemDb,
    read::{DataType, ID},
};

impl MemDb {
    pub fn xrange(&self, key: &str, start: &str, end: &str) -> DataType {
        if let DataType::Stream(s) = self.get(key) {
            let start: u128 = match start {
                "-" => 0,
                o => o.parse().unwrap_or(0),
            };
            let end: u128 = match end {
                "+" => match s.last() {
                    Some(v) => match &v.0 {
                        ID::Sequence(seq) => seq.ms,
                        _ => 0,
                    },
                    None => 0,
                },
                o => o.parse().unwrap_or(0),
            };

            let results = s
                .into_iter()
                .filter(|i| match &i.0 {
                    ID::Sequence(s) => s.ms >= start && s.ms <= end,
                    _ => false,
                })
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
        read::{DataType, ID, Seq},
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

        let val = db.xrange("test", "0", &ms.to_string());

        let mut expected: HashMap<String, DataType> = HashMap::new();
        expected.insert("a".to_string(), DataType::BulkString("true".to_string()));
        let id = Seq { ms: 1, seq: 1 };
        assert_eq!(
            val,
            DataType::Stream(vec![(ID::Sequence(id), DataType::KV(expected))])
        );
    }

    #[test]
    fn get_stream_using_plus_operator() {
        let db = MemDb::new();

        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("0-1".to_string()),
                DataType::BulkString("b".to_string()),
                DataType::BulkString("false".to_string()),
            ],
        );
        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("1-1".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );
        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("2-1".to_string()),
                DataType::BulkString("c".to_string()),
                DataType::BulkString("null".to_string()),
            ],
        );

        let val = db.xrange("test", "1", "+");

        let mut expected: HashMap<String, DataType> = HashMap::new();
        let mut data: Vec<(ID, DataType)> = Vec::new();
        expected.insert("a".to_string(), DataType::BulkString("true".to_string()));

        data.push((ID::Sequence(Seq { ms: 1, seq: 1 }), DataType::KV(expected)));

        expected = HashMap::new();
        expected.insert("c".to_string(), DataType::BulkString("null".to_string()));

        data.push((ID::Sequence(Seq { ms: 2, seq: 1 }), DataType::KV(expected)));

        assert_eq!(val, DataType::Stream(data));
    }

    #[test]
    fn get_stream_using_minus_operator() {
        let db = MemDb::new();

        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("0-1".to_string()),
                DataType::BulkString("b".to_string()),
                DataType::BulkString("false".to_string()),
            ],
        );
        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("1-1".to_string()),
                DataType::BulkString("a".to_string()),
                DataType::BulkString("true".to_string()),
            ],
        );
        let _ = db.xadd(
            "test",
            &[
                DataType::BulkString("2-1".to_string()),
                DataType::BulkString("c".to_string()),
                DataType::BulkString("null".to_string()),
            ],
        );

        let val = db.xrange("test", "-", "1");

        let mut expected: HashMap<String, DataType> = HashMap::new();
        let mut data: Vec<(ID, DataType)> = Vec::new();
        expected.insert("b".to_string(), DataType::BulkString("false".to_string()));

        data.push((ID::Sequence(Seq { ms: 0, seq: 1 }), DataType::KV(expected)));

        expected = HashMap::new();
        expected.insert("a".to_string(), DataType::BulkString("true".to_string()));

        data.push((ID::Sequence(Seq { ms: 1, seq: 1 }), DataType::KV(expected)));

        assert_eq!(val, DataType::Stream(data));
    }
}
