use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};

pub fn ping(_input_seq: &[DataType], _mem_db: MemDb, _exp_db: ExpDb) -> DataType {
    DataType::SimpleStr("PONG".to_string())
}
