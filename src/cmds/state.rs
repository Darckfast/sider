use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use tokio::sync::Notify;

use crate::read::DataType;

pub struct MemDb {
    pub map: Mutex<HashMap<String, DataType>>,
    pub expiry: Mutex<Vec<(String, Instant)>>,
    pub notify: Mutex<HashMap<String, Arc<Notify>>>,
}

impl MemDb {
    pub fn new() -> Self {
        Self {
            map: Mutex::new(HashMap::new()),
            expiry: Mutex::new(Vec::new()),
            notify: Mutex::new(HashMap::new()),
        }
    }
}
