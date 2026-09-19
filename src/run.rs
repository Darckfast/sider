use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio::{io::AsyncWriteExt, net::TcpListener};

use crate::{
    cmd::MemDb,
    cmds,
    expiry::{self, ExpDb},
    read::read_stream,
    serialize::serialize_resp,
};

pub async fn listen() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("listening to :6379");
    let mem_db: MemDb = Arc::new(Mutex::new(HashMap::new()));
    let exp_db: ExpDb = Arc::new(Mutex::new(Vec::new()));

    let mem_db_a = Arc::clone(&mem_db);
    let exp_db_a = Arc::clone(&exp_db);
    tokio::spawn(async move {
        expiry::background_scheduler(mem_db_a, exp_db_a).await;
    });

    loop {
        let Ok((mut socket, _)) = listener.accept().await else {
            eprintln!("Failed to accept client");
            continue;
        };

        let mem_db_b = Arc::clone(&mem_db);
        let exp_db_b = Arc::clone(&exp_db);

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            match read_stream(reader).await {
                Ok(cmd) => match cmds::exec::exec(&cmd, mem_db_b, exp_db_b) {
                    Ok(results) => {
                        let data = serialize_resp(results);
                        if let Err(e) = writer.write_all(data.as_bytes()).await {
                            eprintln!("error writing data: {e}");
                        }

                        if let Err(e) = writer.flush().await {
                            eprintln!("error flushing writer: {e}");
                        }
                    }
                    Err(e) => eprintln!("error executing cmd: {e}"),
                },
                Err(e) => eprintln!("error processing connection {e}"),
            };
        });
    }
}
