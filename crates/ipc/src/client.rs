use crate::Message;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

pub struct IpcClient {
    tx: mpsc::Sender<Message>,
}

impl IpcClient {
    pub async fn connect(port: u16) -> anyhow::Result<(Self, mpsc::Receiver<Message>)> {
        let addr = format!("127.0.0.1:{}", port);
        let socket = TcpStream::connect(addr).await?;
        let mut framed = Framed::new(socket, LengthDelimitedCodec::new());

        let (tx, mut rx) = mpsc::channel::<Message>(100);
        let (out_tx, out_rx) = mpsc::channel(100);

        tokio::spawn(async move {
            loop {
                tokio::select! {
                    Some(msg) = rx.recv() => {
                         if let Ok(bytes) = serde_json::to_vec(&msg) {
                             if let Err(e) = framed.send(bytes.into()).await {
                                 tracing::error!("Failed to send: {}", e);
                                 break;
                             }
                         }
                    }
                    val = framed.next() => {
                        match val {
                            Some(Ok(bytes)) => {
                                if let Ok(msg) = serde_json::from_slice::<Message>(&bytes) {
                                    if out_tx.send(msg).await.is_err() {
                                        break;
                                    }
                                }
                            }
                            Some(Err(e)) => {
                                tracing::error!("Read error: {}", e);
                                break;
                            }
                            None => break,
                        }
                    }
                }
            }
        });

        Ok((Self { tx }, out_rx))
    }

    pub async fn send(&self, msg: Message) -> anyhow::Result<()> {
        self.tx.send(msg).await.map_err(|_| anyhow::anyhow!("Connection closed"))
    }
}
