use std::time::Duration;
use tokio;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};

mod net;
use net::Server;

#[tokio::main]
async fn main() {
    tokio::spawn(async {
        if let Err(e) = Server::new().listen().await {
            eprintln!("server error: {}", e);
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    let mut client = TcpStream::connect("127.0.0.1:42042")
        .await
        .expect("failed to connect");

    client
        .write_all(b"Hello, world!")
        .await
        .expect("failed to write");

    tokio::time::sleep(Duration::from_millis(100)).await;
}
