use log::info;
use std::time::Duration;
use tokio::time::sleep;

use crate::backend::Backend;

mod backend;

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let mut backend = Backend::init().await.expect("Failed to init backend");

    // blink example
    loop {
        for v in [255, 0] {
            backend.pico.set_led(v, v, v).await.unwrap();
            // info!("Dist: {:?}", backend.pico.get_ultra().await);
            info!("Track: {:?}", backend.pico.get_track().await);
            sleep(Duration::from_millis(50)).await;
        }
    }
}
