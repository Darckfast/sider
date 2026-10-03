use std::sync::Arc;

use tokio::sync::Notify;

use crate::{
    cmds::state::MemDb,
    read::{DataType, ID, Seq},
};

impl MemDb {
    pub fn notify_for(&self, key: &str) -> Arc<Notify> {
        let mut notifier = self.notify.lock().unwrap();

        notifier
            .entry(key.to_string())
            .or_insert_with(|| Arc::new(Notify::new()))
            .clone()
    }

    pub async fn wait_for(&self, key: &str, id: Option<Seq>) {
        let notifier = self.notify_for(key);
        loop {
            {
                let map = self.map.lock().unwrap();
                if let Some(v) = map.get(key) {
                    match v {
                        DataType::List(list) if list.len() > 0 => return,
                        DataType::Stream(stream) if let Some(id) = id.as_ref() => {
                            for (key, _) in stream {
                                match key {
                                    ID::Sequence(key) => {
                                        dbg!(key, id);
                                        if key.ms == id.ms && key.seq == id.seq {
                                            return;
                                        } else if key.ms > id.ms {
                                            return;
                                        }
                                    }
                                    _ => (),
                                }
                            }
                        }
                        _ => (),
                    }
                    // return v.clone();
                }
            }

            notifier.notified().await;
        }
    }
}
