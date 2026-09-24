use std::fmt::format;

use crate::read::DataType;

const SEP: &'static str = "\r\n";

pub fn serialize_resp(ds: DataType) -> String {
    let mut rs = String::new();
    let s = match ds {
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
    };

    rs += &s;

    rs
}
