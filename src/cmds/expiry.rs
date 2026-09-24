use std::time::{Duration, Instant};

use tokio::time::interval;

use crate::cmds::state::MemDb;

impl MemDb {
    pub async fn bg_expiry_check(&self) {
        let mut inter = interval(Duration::from_mins(1));
        inter.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        loop {
            inter.tick().await;
            let mut list = self.expiry.lock().unwrap();
            let mut map = self.map.lock().unwrap();

            list.retain(|(key, ts)| {
                if Instant::now() >= *ts {
                    println!("Entry {key} expired");
                    let _ = map.remove(key);
                    false
                } else {
                    true
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{sync::Arc, time::Duration};

    use tokio::time::sleep;

    use crate::{cmds::state::MemDb, read::DataType};

    #[tokio::test]
    async fn expirt_with_active_check() {
        let state = Arc::new(MemDb::new());

        state.set(
            "test_3",
            DataType::SimpleStr("My-Expired-String".to_string()),
            Some(Duration::from_nanos(0)),
        );

        sleep(Duration::from_millis(1)).await;
        let state_1 = Arc::clone(&state);
        tokio::spawn(async move {
            state_1.bg_expiry_check().await;
        });

        let val = state.get("test_3");

        assert_eq!(val, DataType::NullStr)
    }
}
