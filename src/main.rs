use std::time::Duration;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

mod net;
use net::{Client, Server};

#[tokio::main]
async fn main() {
    // Start server.
    tokio::spawn(async {
        if let Err(e) = Server::new().listen().await {
            eprintln!("server error: {}", e);
        }
    });

    tokio::time::sleep(Duration::from_millis(100)).await;

    // Start client.
    tokio::spawn(async {
        let client = Client::new().connect().await.expect("client failure");

        client
            .send("Hello, world!".to_string())
            .await
            .expect("failed to write")
    });

    tokio::time::sleep(Duration::from_millis(100)).await;
}
