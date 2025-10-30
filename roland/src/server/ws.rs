use std::str::FromStr;

use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use log::{debug, error, info};
use serde::Deserialize;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::protocol::Message as WsMessage;
use tokio_tungstenite::{WebSocketStream, accept_async};
use tokio_util::sync::CancellationToken;

use crate::backend::roland::Roland;
use crate::server::message::{ClientMessage, ServerMessage};

#[derive(Deserialize, Debug, PartialEq)]
enum ControlState {
    ManualControl,
    FollowLine,
    KeepDistance,
}

impl FromStr for ControlState {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "ManualControl" => Ok(ControlState::ManualControl),
            "FollowLine" => Ok(ControlState::FollowLine),
            "KeepDistance" => Ok(ControlState::KeepDistance),
            _ => Err(()),
        }
    }
}

pub struct Server {
    roland: Roland,
    state: ControlState,
    auto_cancel: Option<CancellationToken>,
}

impl Server {
    pub fn new(roland: Roland) -> Self {
        Self {
            roland,
            state: ControlState::ManualControl,
            auto_cancel: None,
        }
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

        let (write, read) = ws_stream.split();

        let r = self.roland.clone();
        tokio::select! {
            _ = self.read_task(read, addr) => {},
            _ = Self::write_task(write, r) => {},
        };

        Ok(())
    }

    async fn write_task(
        mut write: SplitSink<WebSocketStream<TcpStream>, WsMessage>,
        r: Roland,
    ) -> anyhow::Result<()> {
        let (write_tx, mut write_rx) = mpsc::channel::<WsMessage>(32);

        let write_rx_task = async move {
            while let Some(msg) = write_rx.recv().await {
                write.send(msg).await?;
            }
            Ok::<(), anyhow::Error>(())
        };

        let ultra_task = {
            let r = r.clone();
            let write_tx = write_tx.clone();
            async move {
                let mut ultra_rx = r.pico.subscribe_ultra();
                loop {
                    let ultra = *ultra_rx.borrow_and_update();
                    if write_tx
                        .send(WsMessage::Text(
                            serde_json::to_string(&ServerMessage::Ultra { ultra })
                                .unwrap()
                                .into(),
                        ))
                        .await
                        .is_err()
                    {
                        break;
                    }
                    ultra_rx.changed().await?;
                }
                Ok::<(), anyhow::Error>(())
            }
        };

        let track_task = async move {
            let mut track_rx = r.pico.subscribe_track();
            loop {
                let track = *track_rx.borrow_and_update();
                if write_tx
                    .send(WsMessage::Text(
                        serde_json::to_string(&ServerMessage::Track { track })
                            .unwrap()
                            .into(),
                    ))
                    .await
                    .is_err()
                {
                    break;
                }
                track_rx.changed().await?;
            }
            Ok::<(), anyhow::Error>(())
        };

        let ret;
        tokio::select! {
            cur_ret = write_rx_task => { ret = cur_ret; },
            cur_ret = ultra_task => { ret = cur_ret; },
            cur_ret = track_task => { ret = cur_ret; },
        };

        ret
    }

    async fn read_task(
        &mut self,
        mut read: SplitStream<WebSocketStream<TcpStream>>,
        addr: std::net::SocketAddr,
    ) -> anyhow::Result<()> {
        while let Some(msg) = read.next().await {
            let msg = msg?;
            if msg.is_text() {
                match serde_json::from_str::<ClientMessage>(msg.to_text()?) {
                    Ok(parsed) => {
                        info!("Got message: {:?}", parsed);
                        self.handle_message(parsed).await?;
                    }
                    Err(e) => {
                        error!("Invalid JSON from {}: {}", addr, e);
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
            ClientMessage::ControlState(state) => match ControlState::from_str(&state) {
                Ok(state) => {
                    info!("Swiching state to: {:?}", state);
                    self.change_state(state).await?;
                }
                Err(()) => {
                    debug!("Invalid control state: {}", state);
                }
            },
        }

        Ok(())
    }

    /// FIXME: a bunch of code duplication here, I don't like this
    async fn change_state(&mut self, new_state: ControlState) -> anyhow::Result<()> {
        if self.state == new_state {
            return Ok(());
        }
        self.state = new_state;
        match self.state {
            ControlState::ManualControl => {
                self.auto_cancel
                    .clone()
                    .expect(
                        "If the device is switching back to manual control, it should have a token.",
                    )
                    .cancel();
                self.auto_cancel = None;
                self.roland.pico.soft_reset().await?;
            }
            ControlState::FollowLine => {
                assert!(self.auto_cancel.is_none());
                let token = CancellationToken::new();
                self.auto_cancel = Some(token.clone());

                let mut r = self.roland.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        ret = r.follow_line(0.8) => {
                            match ret {
                                Ok(()) => (),
                                Err(e) => error!("[Line Follower] error: {}", e),
                            }
                        }
                        _ = token.cancelled() => {},
                    }
                });
            }
            ControlState::KeepDistance => {
                assert!(self.auto_cancel.is_none());
                let token = CancellationToken::new();
                self.auto_cancel = Some(token.clone());

                let mut r = self.roland.clone();
                tokio::spawn(async move {
                    tokio::select! {
                        ret = r.keep_distance(40) => {
                            match ret {
                                Ok(()) => (),
                                Err(e) => error!("[Distance Keeper] error: {}", e),
                            }
                        }
                        _ = token.cancelled() => {},
                    }
                });
            }
        }
        Ok(())
    }
}
