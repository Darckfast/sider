use std::sync::Arc;

use tokio::{io::AsyncWriteExt, net::TcpListener};

use crate::{cmds, cmds::state::MemDb, read::read_stream, serialize::serialize_resp};

pub async fn listen() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("listening to :6379");
    let mem_db = Arc::new(MemDb::new());
    let mem_db_a = Arc::clone(&mem_db);

    tokio::spawn(async move {
        mem_db_a.bg_expiry_check().await;
    });

    loop {
        let Ok((mut socket, _)) = listener.accept().await else {
            eprintln!("Failed to accept client");
            continue;
        };

        let mem_db_b = Arc::clone(&mem_db);

        tokio::spawn(async move {
            let (reader, mut writer) = socket.split();
            match read_stream(reader).await {
                Ok(cmd) => match cmds::exec::exec(&cmd, mem_db_b) {
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
