use log::info;
use std::time::Duration;
use tokio::time::sleep;

mod hardware;
mod serial;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut hw = serial::init().await.expect("Failed to init serial");

    // blink example
    loop {
        for v in [255, 0] {
            hw.set_led(v, v, v).await.unwrap();
            info!("Dist: {:?}", hw.get_ultra().await);
            sleep(Duration::from_millis(200)).await;
        }
    }
}
