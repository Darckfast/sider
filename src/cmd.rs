use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use anyhow::{Result, bail};

use crate::{expiry::ExpDb, read::DataType};

pub type MemDb = Arc<Mutex<HashMap<String, DataType>>>;

pub fn exec_cmd(cmd_seq: &[DataType], mem_db: MemDb, exp_db: ExpDb) -> Result<Vec<DataType>> {
    match &cmd_seq[0] {
        DataType::BulkString(c) => match c.to_uppercase().as_str() {
            "PING" => Ok(vec![DataType::SimpleStr("PONG".to_string())]),
            "ECHO" => Ok(cmd_seq[1..].to_vec()),
            "SET" => {
                let key = match &cmd_seq[1] {
                    DataType::BulkString(bs) => bs,
                    _ => bail!("wrong key value type"),
                };

                let mut map = mem_db.lock().unwrap();
                map.insert(key.to_string(), cmd_seq[2].clone());

                let expiry = match cmd_seq.get(3) {
                    Some(arg) if *arg == DataType::BulkString("EX".to_string()) => {
                        let exp_val: u64 = match cmd_seq.get(4) {
                            Some(a) => match a {
                                DataType::Int(v) => *v as u64,
                                DataType::BulkString(bs) => bs.parse().unwrap(),
                                _ => 0,
                            },
                            None => 0,
                        };

                        Some(Duration::from_secs(exp_val))
                    }
                    Some(arg) if *arg == DataType::BulkString("PX".to_string()) => {
                        let exp_val: u64 = match cmd_seq.get(4) {
                            Some(a) => match a {
                                DataType::Int(v) => *v as u64,
                                DataType::BulkString(bs) => bs.parse().unwrap(),
                                _ => 0,
                            },
                            None => 0,
                        };

                        Some(Duration::from_millis(exp_val))
                    }
                    _ => None,
                };

                if let Some(exp_val) = expiry {
                    let mut exp = exp_db.lock().unwrap();
                    exp.push((key.to_string(), Instant::now() + exp_val));
                }

                Ok(vec![DataType::SimpleStr("OK".to_string())])
            }
            "GET" => {
                let mut map = mem_db.lock().unwrap();
                let key = match &cmd_seq[1] {
                    DataType::BulkString(bs) => bs,
                    _ => bail!("wrong key value type"),
                };

                let val = match map.get(key) {
                    Some(v) => v.clone(),
                    None => DataType::NullStr,
                };

                let mut exp = exp_db.lock().unwrap();
                let is_expired = exp.iter().find(|(k, v)| k == key && *v >= Instant::now());

                // passive
                match is_expired {
                    Some(_) => {
                        let _ = map.remove(key);
                        exp.retain(|(k, _)| k != key);
                        return Ok(vec![DataType::NullStr]);
                    }
                    None => (),
                }

                Ok(vec![val])
            }
            "RPUSH" => {
                let key = match &cmd_seq[1] {
                    DataType::BulkString(bs) => bs,
                    _ => bail!("wrong key value type"),
                };

                let mut map = mem_db.lock().unwrap();
                let entry = map
                    .entry(key.to_string())
                    .and_modify(|e| match e {
                        DataType::List(l) => {
                            l.append(&mut cmd_seq[2..].to_vec());
                        }
                        _ => (),
                    })
                    .or_insert(DataType::List(cmd_seq[2..].to_vec()));

                if let DataType::List(l) = entry {
                    Ok(vec![DataType::Int(l.len() as i64)])
                } else {
                    Ok(vec![])
                }
            }
            cmd => {
                eprintln!("unmapped cmd: {cmd}");
                Ok(vec![])
            }
        },
        _ => Ok(vec![]),
    }
}
