use log::{debug, error, info};
use tokio_util::sync::CancellationToken;

use crate::{backend::roland::Roland, server::ws::Server};

mod backend;
mod server;
mod util;

async fn main_task(r: Roland) -> anyhow::Result<()> {
    let mut server = Server::new(r);
    server.run().await
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Debug).unwrap();

    let token = CancellationToken::new();

    let mut r = Roland::init(token.clone())
        .await
        .expect("Failed to init backend");

    {
        let mut r = r.clone();
        tokio::spawn(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("^C interrupt received, shutdown initiated");
            r.reset().await.unwrap();
        });
    }

    tokio::select! {
        ret = main_task(r.clone()) => {
            match ret {
                Ok(()) => debug!("[Main] task shutting down"),
                Err(e) => error!("[Main] task shutting down: {}", e),
            }
            let _ = r.reset().await;
        }
        _ = token.cancelled() => {}
    }
}
