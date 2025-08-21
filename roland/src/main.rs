use log::info;
use tokio_util::sync::CancellationToken;

use crate::backend::roland::Roland;

mod backend;
mod util;

async fn main_task(mut r: Roland) {
    // r.pico.set_motor(0xffff, -0xffff).await.unwrap();
    r.pico.set_buzzer(400).await.unwrap();
    // r.rgb_led_test().await;
    r.servo_test().await;
    // let mut r_cl = r.clone();
    // tokio::select! {
    //     _ = r.rgb_led_test() => {},
    //     _ = r_cl.keep_distance(30) => {},
    // }
}

#[tokio::main]
async fn main() {
    simple_logger::init_with_level(log::Level::Info).unwrap();

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
