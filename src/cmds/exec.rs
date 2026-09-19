use crate::{cmd::MemDb, cmds, expiry::ExpDb, read::DataType};
use anyhow::{Result, bail};

pub fn exec(input_seq: &[DataType], mem_db: MemDb, exp_db: ExpDb) -> Result<DataType> {
    let cmd = input_seq
        .first()
        .ok_or(anyhow::Error::msg("RESP data is empty"))?;
    match cmd {
        DataType::BulkString(bs) => {
            let data = match bs.to_uppercase().as_str() {
                "ECHO" => cmds::echo::echo(input_seq, mem_db, exp_db),
                "PING" => cmds::ping::ping(input_seq, mem_db, exp_db),
                "SET" => cmds::set::set(input_seq, mem_db, exp_db)?,
                "GET" => cmds::get::get(input_seq, mem_db, exp_db)?,
                "RPUSH" => cmds::rpush::rpush(input_seq, mem_db, exp_db)?,
                "LRANGE" => cmds::lrange::rpush(input_seq, mem_db, exp_db)?,
                cmd => bail!("command: {cmd} not supported"),
            };

            Ok(data)
        }
        _ => todo!("implement other data type"),
    }
}
