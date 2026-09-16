use tokio::net::TcpListener;

use crate::ping::read_stream;

pub async fn listen() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();
    println!("listening to :6379");

    loop {
        match listener.accept().await {
            Ok((mut stream, _)) => {
                println!("accepted new connection");
                tokio::spawn(async move {
                    match read_stream(&mut stream).await {
                        Ok(_) => (),
                        Err(e) => eprintln!("error processing connection {e}"),
                    };
                });
            }
            Err(e) => eprintln!("error: {e}"),
        };
    }
}
