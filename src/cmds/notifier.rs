use std::sync::Arc;

use tokio::sync::Notify;

use crate::{cmds::state::MemDb, read::DataType};

impl MemDb {
    pub fn notify_for(&self, key: &str) -> Arc<Notify> {
        let mut notifier = self.notify.lock().unwrap();

        notifier
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    pub async fn wait_for(&self, key: &str) {
        let notifier = self.notify_for(key);
        loop {
            {
                let map = self.map.lock().unwrap();
                if let Some(v) = map.get(key) {
                    match v {
                        DataType::List(list) if list.len() > 0 => return,
                        _ => (),
                    }
                    // return v.clone();
                }
            }

            notifier.notified().await;
        }
    }
}
