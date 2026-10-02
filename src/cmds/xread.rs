use std::sync::Arc;

use crate::{
    cmds::state::MemDb,
    read::{DataType, IDS},
};

pub fn xread(_key: &str, args: &[DataType], db: Arc<MemDb>) -> DataType {
    let mid = (args.len() - 1) / 2;
    let (keys, ids) = args[1..].split_at(mid);

    let mut out: Vec<(IDS, DataType)> = Vec::new();
    for (key, id) in keys.iter().zip(ids.iter()) {
        if let DataType::BulkString(id) = id
            && let DataType::BulkString(key) = key
        {
            let data = db.xrange(key, id, "+");
            out.push((IDS::Str(key.clone()), data));
        }
    }

    DataType::Stream(out)
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, sync::Arc};

    use crate::{
        cmds::{state::MemDb, xread::xread},
        read::{DataType, ID, IDS},
    };

    #[test]
    fn read_single_stream() {
        let key = "some_key";

        let db = Arc::new(MemDb::new());
        let mut data: Vec<(IDS, DataType)> = Vec::new();
        let mut map: HashMap<String, DataType> = HashMap::new();

        map.insert(
            "temperature".to_string(),
            DataType::BulkString("36".to_string()),
        );
        map.insert(
            "humidity".to_string(),
            DataType::BulkString("95".to_string()),
        );
        data.push((
            IDS::Sequence(ID {
                ms: 1526985054069,
                seq: 0,
            }),
            DataType::KV(map),
        ));

        db.set(key, DataType::Stream(data), None);

        data = Vec::new();
        map = HashMap::new();

        map.insert(
            "temperature".to_string(),
            DataType::BulkString("37".to_string()),
        );
        map.insert(
            "humidity".to_string(),
            DataType::BulkString("94".to_string()),
        );
        data.push((
            IDS::Sequence(ID {
                ms: 1526985054079,
                seq: 0,
            }),
            DataType::KV(map.clone()),
        ));

        db.set(key, DataType::Stream(data), None);
        let results = xread(
            key,
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
                IDS::Str("some_key".to_string()),
                DataType::Stream(vec![(
                    IDS::Sequence(ID {
                        ms: 1526985054079,
                        seq: 0
                    }),
                    DataType::KV(map)
                )])
            )])
        );
    }
}
