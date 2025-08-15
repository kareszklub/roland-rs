use std::time::Duration;
use tokio::{
    sync::{broadcast, mpsc},
    time::sleep,
};

use crate::serial::{SerialCMD, SerialData};

mod serial;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let (cmd_tx, cmd_rx) = mpsc::channel::<SerialCMD>(32);
    // TODO: figure out how to shard serial data correctly
    let (data_tx, data_rx) = broadcast::channel::<SerialData>(32);

    serial::init(cmd_rx, data_tx)
        .await
        .expect("Failed to init serial");

    // blink example
    loop {
        // cmd_tx.send(SerialCMD::LED((255, 255, 255))).await.unwrap();
        // sleep(Duration::from_millis(200)).await;
        // cmd_tx.send(SerialCMD::LED((0, 0, 0))).await.unwrap();
        // sleep(Duration::from_millis(200)).await;
    }
}
