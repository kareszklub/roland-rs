use log::{debug, info};
use std::time::Duration;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::backend::Backend;

mod backend;

async fn main_task(mut backend: Backend) {
    // blink example
    loop {
        for v in [255, 0] {
            backend.pico.set_led(v, v, v).await.unwrap();
            info!("Dist: {:?}", backend.pico.get_ultra().await);
            sleep(Duration::from_millis(70)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

    let token = CancellationToken::new();

    let backend = Backend::init(token.clone())
        .await
        .expect("Failed to init backend");

    {
        let token = token.clone();
        let mut backend = backend.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("^C interrupt received, cleanup started");

            let _ = backend.pico.reset().await;

            info!("Cleanup finished, shutdown initiated");
            token.cancel();
        });
    }

    tokio::select! {
        _ = main_task(backend) => {}
        _ = token.cancelled() => {
            info!("Main task shutting down");
        }
    }
}
