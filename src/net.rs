use std::error::Error;
use std::net::SocketAddr;
use tokio;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync;

pub struct Server {}

impl Server {
    pub fn new() -> Self {
        Server {}
    }

    async fn handle(
        socket: TcpStream,
        tx: sync::broadcast::Sender<String>,
    ) -> Result<(), Box<dyn Error>> {
        let (mut reader, mut writer) = socket.into_split();
        let mut rx = tx.subscribe();

        let reader_handle = tokio::spawn(async move {
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

        let writer_handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(message) => {
                        writer.write(message.as_bytes()).await;
                    }
                    Err(e) => eprintln!("failed to recv broadcast {}", e),
                }
            }
        });

        // Wait for either reader or writer to complete.
        // This signals that the connection has been closed.
        let _ = tokio::join!(reader_handle, writer_handle);
        Ok(())
    }

    pub async fn listen(self) -> Result<(), Box<dyn Error>> {
        let listener = TcpListener::bind("127.0.0.1:42042").await?;
        println!("started server on 42024");

        let (tx, _) = sync::broadcast::channel::<String>(100);

        loop {
            match listener.accept().await {
                Ok((socket, _addr)) => {
                    let tx = tx.clone();
                    tokio::spawn(async move {
                        match Self::handle(socket, tx).await {
                            Ok(_) => (),
                            Err(e) => eprintln!("failed handling socket {}", e),
                        };
                    });
                }
                Err(e) => {
                    eprintln!("error accepting connection: {}", e);
                }
            }
        }
    }
}

pub struct Client {
    sender: sync::broadcast::Sender<String>,
}

impl Client {
    pub fn new() -> Self {
        let (tx, _) = sync::broadcast::channel::<String>(100);
        Client { sender: tx }
    }

    pub async fn connect(self) -> Result<Self, Box<dyn Error>> {
        let (mut reader, mut writer) = TcpStream::connect("127.0.0.1:42042").await?.into_split();
        let (tx, mut rx) = sync::broadcast::channel::<String>(100);
        let mut send_rx = self.sender.clone().subscribe();

        let _reader_handle = tokio::spawn(async move {
            loop {
                let mut buf = [0u8; 256];
                let message = match reader.read(&mut buf).await {
                    Ok(0) => continue,
                    Ok(n) => buf[0..n].to_vec(),
                    Err(e) => {
                        eprintln!("failed to read buffer {}", e);
                        continue;
                    }
                };

                let message = match String::from_utf8(message) {
                    Ok(message) => message,
                    Err(e) => {
                        eprintln!("failed to decode buffer to utf8 {}", e);
                        continue;
                    }
                };

                match tx.send(message) {
                    Ok(_) => (),
                    Err(e) => eprintln!("failed to send broadcast {}", e),
                };
            }
        });

        let _writer_handle = tokio::spawn(async move {
            loop {
                match rx.recv().await {
                    Ok(message) => println!("chat {}", message),
                    Err(e) => eprintln!("failed to receive broadcast {}", e),
                };
            }
        });

        let _sender_handle = tokio::spawn(async move {
            loop {
                match send_rx.recv().await {
                    Ok(message) => {
                        writer.write(message.as_bytes()).await;
                    }
                    Err(e) => (), // eprintln!("failed to recieve broadcast {}", e),
                };
            }
        });

        Ok(self)
    }

    pub async fn send(self, message: String) -> Result<Self, Box<dyn Error>> {
        let tx = self.sender.clone();
        tx.send(message)?;

        Ok(self)
    }

    pub async fn disconnect() -> Result<(), Box<dyn Error>> {
        Ok(())
    }
}
