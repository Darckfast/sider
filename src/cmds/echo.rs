use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};

pub fn echo(input_seq: &[DataType], _mem_db: MemDb, _exp_db: ExpDb) -> DataType {
    input_seq[1].clone()
}
