use std::error::Error;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Server {}
    }

    pub async fn listen(self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:42042").await?;
        println!("started server on 42024");

        let (tx, _) = sync::broadcast::channel::<String>(100);

        loop {
            match listener.accept().await {
                Ok((socket, addr)) => {
                    let tx = tx.clone();

                    tokio::spawn(async move {
                        let (mut reader, mut writer) = socket.into_split();
                        let mut rx = tx.subscribe();

                        tokio::spawn(async move {
                            loop {
                                let mut buf: [u8; 256] = [0u8; 256];
                                match reader.read(&mut buf).await {
                                    Ok(0) => continue,
                                    Ok(n) => {
                                        println!("read {:?} bytes", n);
                                        let message = match String::from_utf8(buf[0..n].to_vec()) {
                                            Ok(message) => message,
                                            Err(e) => {
                                                eprintln!("failed to decode to utf8 {}", e);
                                                continue;
                                            }
                                        };

                                        match tx.send(message) {
                                            Ok(_) => (),
                                            Err(e) => eprintln!("failed to broadcast {}", e),
                                        };
                                    }
                                    Err(e) => eprintln!("failed to read socket {}", e),
                                };
                            }
                        });

                        tokio::spawn(async move {
                            loop {
                                match rx.recv().await {
                                    Ok(message) => {
                                        writer.write(message.as_bytes()).await;
                                    }
                                    Err(e) => eprintln!("failed to recv broadcast {}", e),
                                }
                            }
                        });
                    });
                }
                Err(e) => {
                    eprintln!("error accepting connection: {}", e);
                }
            }
        }
    }
}
