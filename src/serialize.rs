use crate::read::DataType;

const SEP: &'static str = "\r\n";

pub fn serialize_resp(ds: DataType) -> String {
    match ds {
        DataType::SimpleStr(s) => format!("+{s}{SEP}"),
        DataType::BulkString(bs) => {
            format!("${}{SEP}{bs}{SEP}", bs.len())
        }
        DataType::NullStr => {
            format!("$-1{SEP}")
        }
        DataType::Int(v) => {
            format!(":{v}{SEP}")
        }
        DataType::UInt(v) => {
            format!(":{v}{SEP}")
        }
        DataType::NullArray => {
            format!("*-1\r\n")
        }
        DataType::EmptyList => {
            format!("*0\r\n")
        }
        DataType::List(items) => {
            let mut serial = format!("*{}{SEP}", items.len());
            for item in items {
                serial = format!("{serial}{}", serialize_resp(item));
            }

            serial
        }
        DataType::Stream(stream) => {
            let mut serial = format!("*{}{SEP}", stream.len());
            for (id, map) in stream {
                serial = format!(
                    "{serial}*2{SEP}{}*{}{SEP}",
                    serialize_resp(DataType::BulkString(id.to_string())),
                    map.len(),
                );

                for (key, value) in map {
                    serial = format!(
                        "{serial}{}{}",
                        serialize_resp(DataType::BulkString(key)),
                        serialize_resp(value)
                    );
                }
            }

            serial
        }
        DataType::Error(e) => {
            format!("-{e}{SEP}")
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashMap, i64};

    use crate::{
        read::{DataType, ID},
        serialize::serialize_resp,
    };

    #[test]
    fn simple_str() {
        let simple_str = DataType::SimpleStr("my-string".to_string());

        let val = serialize_resp(simple_str);

        assert_eq!(&val, "+my-string\r\n");
    }

    #[test]
    fn bulk_str() {
        let blk = DataType::BulkString("my-string".to_string());

        let val = serialize_resp(blk);

        assert_eq!(&val, "$9\r\nmy-string\r\n");
    }

    #[test]
    fn null_str() {
        let n_str = DataType::NullStr;

        let val = serialize_resp(n_str);

        assert_eq!(&val, "$-1\r\n");
    }

    #[test]
    fn int() {
        let i = DataType::Int(i64::MAX);

        let val = serialize_resp(i);

        assert_eq!(val, ":9223372036854775807\r\n");
    }

    #[test]
    fn stream() {
        let mut data: Vec<(ID, HashMap<String, DataType>)> = Vec::new();
        let mut map: HashMap<String, DataType> = HashMap::new();

        map.insert("test".to_string(), DataType::BulkString("1".to_string()));
        data.push((ID { ms: 1, seq: 0 }, map));

        let stream = DataType::Stream(data);

        let serial_str = serialize_resp(stream);

        assert_eq!(
            &serial_str,
            "*1\r\n*2\r\n$3\r\n1-0\r\n*1\r\n$4\r\ntest\r\n$1\r\n1\r\n"
        );
    }
}
