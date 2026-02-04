use crate::{Message, DEFAULT_PORT};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::broadcast;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};

pub struct IpcServer {
    port: u16,
    tx: broadcast::Sender<Message>,
}

impl IpcServer {
    pub fn new(port: u16) -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { port, tx }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        tracing::info!("IPC Server listening on {}", addr);

        loop {
            let (socket, _) = listener.accept().await?;
            let tx = self.tx.clone();
            tokio::spawn(async move {
                if let Err(e) = handle_connection(socket, tx).await {
                    tracing::error!("Connection error: {}", e);
                }
            });
        }
    }

    pub fn broadcast(&self, msg: Message) {
        let _ = self.tx.send(msg);
    }

    pub fn get_tx(&self) -> broadcast::Sender<Message> {
        self.tx.clone()
    }
}

async fn handle_connection(socket: TcpStream, mut tx: broadcast::Sender<Message>) -> anyhow::Result<()> {
    let mut framed = Framed::new(socket, LengthDelimitedCodec::new());
    let mut rx = tx.subscribe();

    loop {
        tokio::select! {
            // Receive from client
            val = framed.next() => {
                match val {
                    Some(Ok(bytes)) => {
                        let msg: Message = serde_json::from_slice(&bytes)?;
                        match msg {
                            Message::Request { id, method: _, params: _ } => {
                                // Simple echo OK for now
                                let resp = Message::Response {
                                    id,
                                    result: Some(serde_json::json!("ok")),
                                    error: None,
                                };
                                let bytes = serde_json::to_vec(&resp)?;
                                framed.send(bytes.into()).await?;
                            }
                            _ => {}
                        }
                    }
                    Some(Err(e)) => return Err(e.into()),
                    None => return Ok(()),
                }
            }
            // Send broadcast event to client
            Ok(msg) = rx.recv() => {
                let bytes = serde_json::to_vec(&msg)?;
                framed.send(bytes.into()).await?;
            }
        }
    }
}
