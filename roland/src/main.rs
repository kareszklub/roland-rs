use log::info;
use tokio_util::sync::CancellationToken;

use crate::backend::roland::Roland;

mod backend;
mod util;

async fn main_task(mut r: Roland) {
    tokio::select! {
        _ = r.follow_line(0.9) => {},
    }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let token = CancellationToken::new();

    let r = Roland::init(token.clone())
        .await
        .expect("Failed to init backend");

    {
        let token = token.clone();
        let mut r = r.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("^C interrupt received, cleanup started");

            let _ = r.pico.reset().await;

            info!("Cleanup finished, shutdown initiated");
            token.cancel();
        });
    }

    tokio::select! {
        _ = main_task(r) => {}
        _ = token.cancelled() => {
            info!("Main task shutting down");
        }
    }
}
