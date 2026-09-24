use std::{
    sync::Arc,
    time::{Duration, Instant},
};

use crate::{cmds::state::MemDb, read::DataType};
use anyhow::{Result, bail};

impl MemDb {
    pub fn set(&self, key: &str, val: DataType, expiration: Option<Duration>) {
        let notifier = self.notify_for(&key);
        let mut map = self.map.lock().unwrap();

        map.insert(key.to_string(), val);
        drop(map);

        if let Some(delay) = expiration {
            let mut exp = self.expiry.lock().unwrap();
            let mut updated = false;
            for (k, ts) in &mut exp.iter_mut() {
                if key == *k {
                    *ts = Instant::now() + delay;
                    updated = true;
                    break;
                }
            }

            if !updated {
                exp.push((key.to_string(), Instant::now() + delay));
            }
        }

        notifier.notify_waiters();
    }
}

pub fn set(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let mut delay: Option<Duration> = None;

    if let Some(DataType::BulkString(ex_arg)) = input_seq.get(3)
        && let Some(ex_val) = input_seq.get(4)
    {
        let ex_val: u64 = match ex_val {
            DataType::Int(v) => *v as u64,
            DataType::BulkString(bs) => bs.parse()?,
            o => bail!("wrong data type for the expiry argument {o:?}"),
        };

        delay = match ex_arg.to_uppercase().as_str() {
            "EX" => Some(Duration::from_secs(ex_val)),
            "PX" => Some(Duration::from_millis(ex_val)),
            o => bail!("argument {o} not supported"),
        };
    }

    if let Some(v) = input_seq.get(2) {
        mem_db.set(key, v.clone(), delay);
    }

    Ok(DataType::SimpleStr("OK".to_string()))
}

#[cfg(test)]
mod tests {
    use std::{i64, thread, time::Duration};

    use crate::{cmds::state::MemDb, read::DataType};

    #[test]
    fn set_and_get_string() {
        let state = MemDb::new();

        state.set("test_1", DataType::SimpleStr("My-String".to_string()), None);
        let val = state.get("test_1");

        assert_eq!(val, DataType::SimpleStr("My-String".to_string()));
    }

    #[test]
    fn set_and_get_int() {
        let state = MemDb::new();

        state.set("test_2", DataType::Int(i64::MAX), None);
        let val = state.get("test_2");
        assert_eq!(val, DataType::Int(i64::MAX));
    }

    #[test]
    fn set_and_expire_data() {
        let state = MemDb::new();

        state.set(
            "test_3",
            DataType::SimpleStr("My-Expired-String".to_string()),
            Some(Duration::from_secs(0)),
        );

        thread::sleep(Duration::from_millis(1));

        let val = state.get("test_3");

        assert_eq!(val, DataType::NullStr);
    }
}
