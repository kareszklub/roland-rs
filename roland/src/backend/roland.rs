use std::time::Duration;

use log::info;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::{
    backend::{pico::Pico, serial},
    util::color::{HSV, RGB},
};

#[derive(Clone)]
pub struct Roland {
    pub pico: Pico,
}

impl Roland {
    pub async fn init(token: CancellationToken) -> anyhow::Result<Self> {
        Ok(Self {
            pico: serial::init(token).await?,
        })
    }

    pub async fn track_sensor_test(&self) {
        loop {
            info!("{:?}", self.pico.get_track().await);
            sleep(Duration::from_millis(10)).await;
        }
    }

    pub async fn servo_test(&mut self) {
        loop {
            for d in [-90, 0, 90, 0] {
                self.pico.set_servo(d).await.unwrap();
                sleep(Duration::from_secs(2)).await;
            }
        }
    }

    pub async fn rgb_led_test(&mut self) {
        loop {
            for h in 0..360 {
                let rgb = RGB::from_hsv(&HSV {
                    h: h as f64,
                    s: 1.0,
                    v: 1.0,
                });
                self.pico.set_led(rgb.r, rgb.g, rgb.b).await.unwrap();
                sleep(Duration::from_millis(3000 / 360)).await;
            }
        }
    }
}
