use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::read::DataType;

pub type MemDb = Arc<Mutex<HashMap<String, DataType>>>;
