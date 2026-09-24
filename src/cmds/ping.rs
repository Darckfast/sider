use std::sync::Arc;

use crate::{cmds::state::MemDb, read::DataType};

pub fn ping(_input_seq: &[DataType], _mem_db: Arc<MemDb>) -> DataType {
    DataType::SimpleStr("PONG".to_string())
}
