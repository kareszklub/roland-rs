use std::time::{Duration, Instant};

use log::{debug, error, info};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::backend::roland::Roland;

mod backend;
mod util;

async fn main_task(mut r: Roland) -> anyhow::Result<()> {
    let r_cl = r.clone();
    let mut r_cl2 = r.clone();
    tokio::select! {
        ret = r.rgb_led_test() => ret?,
        ret = r_cl.ultra_test() => ret?,
        _ = async move {
            loop {
               for f in [500, 0] {
                   r_cl2.pico.set_buzzer(f).await.unwrap();
                   sleep(Duration::from_millis(500)).await;
               }
           }
        } => {},
    }
    // // serial throughput test
    // let mut start = Instant::now();
    // for i in 1.. {
    //     r.pico.set_buzzer(0).await?;
    //     if i % 1000 == 0 {
    //         let now = Instant::now();
    //         let d = now - start;
    //         start = now;
    //         info!("{:>4}", 1000. / d.as_secs_f64());
    //     }
    // }
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
            let _ = r.reset().await;
        }
        _ = token.cancelled() => {}
    }
}
