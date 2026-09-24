use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};

pub fn echo(input_seq: &[DataType], _mem_db: Arc<MemDb>) -> DataType {
    input_seq[1].clone()
}
