use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::time::interval;

use crate::cmd::MemDb;

pub type ExpDb = Arc<Mutex<Vec<(String, Instant)>>>;

pub async fn background_scheduler(mem_db: MemDb, exp_db: ExpDb) {
    let mut inter = interval(Duration::from_millis(100));
    inter.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    loop {
        inter.tick().await;
        let mut list = exp_db.lock().unwrap();
        let mut map = mem_db.lock().unwrap();

        list.retain(|(key, ts)| {
            if Instant::now() >= *ts {
                println!("Entry: {key} expired");
                let _ = map.remove(key);
                false
            } else {
                true
            }
        });
    }
}
