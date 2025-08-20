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
                info!("Servo set to {}", d);
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

    pub async fn ultra_test(&mut self) {
        loop {
            let d = match self.pico.get_ultra().await {
                Some(d) => d,
                None => 0,
            };

            info!("{:>3} cm", d);
            sleep(Duration::from_millis(60)).await;
        }
    }

    pub async fn motor_test(&mut self) {
        loop {
            for i in (0..100).chain((0..=100).rev()) {
                let s = ((i as f64 / 100.0) * 0xffff as f64).round() as i32;
                let s = 0xffff;
                self.pico.set_motor(s, s).await.unwrap();
                sleep(Duration::from_millis(20)).await;
            }
        }
    }
}
