use std::time::{Duration, Instant};

use crate::{cmd::MemDb, expiry::ExpDb, read::DataType};
use anyhow::{Result, bail};

pub fn set(input_seq: &[DataType], mem_db: MemDb, exp_db: ExpDb) -> Result<DataType> {
    let key = match &input_seq[1] {
        DataType::BulkString(bs) => bs,
        _ => bail!("wrong key value type"),
    };

    let mut map = mem_db.lock().unwrap();

    if let Some(v) = input_seq.get(2) {
        map.insert(key.to_string(), v.clone());
    }

    if let Some(DataType::BulkString(ex_arg)) = input_seq.get(3)
        && let Some(ex_val) = input_seq.get(4)
    {
        let ex_val: u64 = match ex_val {
            DataType::Int(v) => *v as u64,
            DataType::BulkString(bs) => bs.parse()?,
            o => bail!("wrong data type for the expiry argument {o:?}"),
        };

        let delay = match ex_arg.to_uppercase().as_str() {
            "EX" => Duration::from_secs(ex_val),
            "PX" => Duration::from_millis(ex_val),
            o => bail!("argument {o} not supported"),
        };

        let mut exp = exp_db.lock().unwrap();
        let mut updated = false;
        for (k, ts) in &mut exp.iter_mut() {
            if key == k {
                *ts = Instant::now() + delay;
                updated = true;
                break;
            }
        }

        if !updated {
            exp.push((key.to_string(), Instant::now() + delay));
        }
    }

    Ok(DataType::SimpleStr("OK".to_string()))
}
