use std::sync::Arc;

use crate::{
    cmds::{self, state::MemDb},
    read::DataType,
};
use anyhow::{Result, bail};

pub async fn exec(input_seq: &[DataType], mem_db: Arc<MemDb>) -> Result<DataType> {
    let cmd = input_seq
        .first()
        .ok_or(anyhow::Error::msg("RESP data is empty"))?;

    let key = match input_seq.get(1).ok_or(anyhow::Error::msg("Missing key"))? {
        DataType::BulkString(bs) => bs,
        _ => bail!("Wrong key DataType"),
    };

    let args = &input_seq[2..];

    match cmd {
        DataType::BulkString(bs) => {
            let data = match bs.to_uppercase().as_str() {
                "ECHO" => cmds::echo::echo(input_seq),
                "PING" => cmds::ping::ping(),
                "SET" => cmds::set::set(input_seq, mem_db)?,
                "GET" => cmds::get::get(input_seq, mem_db)?,
                "RPUSH" => cmds::rpush::rpush(input_seq, mem_db)?,
                "LPUSH" => cmds::lpush::lpush(input_seq, mem_db)?,
                "LRANGE" => cmds::lrange::lrange(input_seq, mem_db)?,
                "LLEN" => cmds::llen::llen(input_seq, mem_db)?,
                "LPOP" => cmds::lpop::lpop(input_seq, mem_db)?,
                "BLPOP" => cmds::blpop::blpop(input_seq, mem_db).await?,
                "TYPE" => cmds::get_type::get_type(input_seq, mem_db)?,
                "XADD" => cmds::xadd::xadd(key, args, mem_db),
                "XRANGE" => cmds::xrange::xrange(key, args, mem_db),
                cmd => {
                    dbg!(input_seq);
                    bail!("command {cmd} not supported")
                }
            };

            Ok(data)
        }
        _ => todo!("implement other data type"),
    }
}
