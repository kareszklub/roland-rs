use std::time::Instant;

use log::{debug, error, info};
use tokio_util::sync::CancellationToken;

use crate::backend::roland::Roland;

mod backend;
mod util;

async fn main_task(mut r: Roland) -> anyhow::Result<()> {
    // serial throughput test
    let mut start = Instant::now();
    for i in 1.. {
        r.pico.set_buzzer(0).await?;
        if i % 1000 == 0 {
            let now = Instant::now();
            let d = now - start;
            start = now;
            info!("{:>4}", 1000. / d.as_secs_f64());
        }
    }
    Ok(())
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
            r.reset().await.unwrap();
        }
        _ = token.cancelled() => {}
    }
}
