use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use serde::Deserialize;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;

use crate::backend::roland::Roland;
use crate::server::message::ClientMessage;

pub struct Server {
    roland: Roland,
}

impl Server {
    pub fn new(roland: Roland) -> Self {
        Self { roland }
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let addr = format!("0.0.0.0:9001");

        let listener = TcpListener::bind(&addr).await?;

        debug!("Listening at {}", addr);

        while let Ok((stream, addr)) = listener.accept().await {
            if let Err(e) = self.handle_connection(stream, addr).await {
                error!("Connection error with {}: {:?}", addr, e);
            }
        }

        Ok(())
    }

    async fn handle_connection(
        &mut self,
        stream: TcpStream,
        addr: std::net::SocketAddr,
    ) -> anyhow::Result<()> {
        let ws_stream = accept_async(stream).await?;
        info!("New WS connection from {}", addr);

        let (mut write, mut read) = ws_stream.split();

        while let Some(msg) = read.next().await {
            let msg = msg?;
            if msg.is_text() {
                match serde_json::from_str::<ClientMessage>(msg.to_text()?) {
                    Ok(parsed) => {
                        info!("Got message: {:?}", parsed);
                        self.handle_message(parsed).await?;
                    }
                    Err(e) => {
                        debug!("Invalid JSON from {}: {}", addr, e);
                        write
                            .send(WsMessage::Text(format!("Error: {}", e).into()))
                            .await?;
                    }
                }
            } else if msg.is_close() {
                info!("WS client {} disconnected", addr);
                break;
            }
        }

        Ok(())
    }

    async fn handle_message(&mut self, msg: ClientMessage) -> anyhow::Result<()> {
        match msg {
            ClientMessage::Buzzer(freq) => {
                self.roland.pico.set_buzzer(freq).await?;
            }
            ClientMessage::LED((r, g, b)) => {
                self.roland.pico.set_led(r, g, b).await?;
            }
            ClientMessage::Servo(deg) => {
                self.roland.pico.set_servo(deg).await?;
            }
            ClientMessage::Motor((l, r)) => {
                let l = (l * 0xffff as f32).round() as i32;
                let r = (r * 0xffff as f32).round() as i32;
                self.roland.pico.set_motor(l, r).await?;
            }
            ClientMessage::FollowLine => {
                todo!("Yeah I have no idea how to implement this");
            }
        }

        Ok(())
    }
}
