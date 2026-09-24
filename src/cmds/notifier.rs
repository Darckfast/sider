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

    pub async fn wait_for(&self, key: &str) -> DataType {
        let notifier = self.notify_for(key);
        loop {
            {
                let map = self.map.lock().unwrap();
                if let Some(v) = map.get(key) {
                    return v.clone();
                }
            }

            notifier.notified().await;
        }
    }
}
