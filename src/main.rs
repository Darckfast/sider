use codecrafters_redis::run;

#[tokio::main]
async fn main() {
    run::listen().await;
}
