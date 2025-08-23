use std::time::Duration;

use log::{debug, info, warn};
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
            for d in [-30, 0, 30, 0] {
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

    pub async fn keep_distance(&mut self, sp: u16) {
        const MAX_TRESHOLD: u16 = 50;
        loop {
            loop {
                let speed = match self.pico.get_ultra().await {
                    Some(pv) => {
                        let s = (pv as i32 - sp as i32) * (0xffff / MAX_TRESHOLD) as i32;
                        info!(
                            "{:>3} cm | {:>3}%",
                            pv,
                            (s.abs() as f64 / 0xffff as f64 * 100 as f64) as u8
                        );
                        s
                    }
                    None => 0,
                };

                self.pico.set_motor(speed, speed).await.unwrap();
                sleep(Duration::from_millis(60)).await;
            }
        }
    }

    pub async fn follow_line(&mut self, speed: f64) {
        #[derive(Debug)]
        enum TrackState {
            OnLine,
            HalfLeft,
            HalfRight,
            Left,
            Right,
            Unknown,
        }

        let mut state = TrackState::Unknown;
        let mut last_speed = None;

        loop {
            let [a, _b, c, _d] = self.pico.get_track().await;

            state = match (a, c) {
                (false, false) => TrackState::OnLine,
                (false, true) => TrackState::HalfRight,
                (true, false) => TrackState::HalfLeft,
                (true, true) => match state {
                    TrackState::OnLine => TrackState::Unknown,
                    TrackState::HalfLeft => TrackState::Left,
                    TrackState::HalfRight => TrackState::Right,
                    TrackState::Left => TrackState::Left,
                    TrackState::Right => TrackState::Right,
                    TrackState::Unknown => TrackState::Unknown,
                },
            };

            let (left, right) = match state {
                TrackState::OnLine => (0.9, 0.9),
                TrackState::HalfLeft => (1.0, 0.75),
                TrackState::HalfRight => (0.75, 1.0),
                TrackState::Left => (1.0, -0.75),
                TrackState::Right => (-0.75, 1.0),
                TrackState::Unknown => (0.0, 0.0),
            };

            let (left, right) = (
                (0xffff as f64 * left * speed) as i32,
                (0xffff as f64 * right * speed) as i32,
            );

            if last_speed != Some((left, right)) {
                last_speed = Some((left, right));
                self.pico.set_motor(right, left).await.unwrap();

                let (r, g, b) = match state {
                    TrackState::OnLine => (0, 255, 0),
                    TrackState::HalfLeft => (128, 128, 0),
                    TrackState::HalfRight => (0, 128, 128),
                    TrackState::Left => (255, 0, 0),
                    TrackState::Right => (0, 0, 255),
                    TrackState::Unknown => (255, 255, 255),
                };

                info!("{:?}", state);
                self.pico.set_led(r, g, b).await.unwrap();
            }

            sleep(Duration::from_millis(2)).await;
        }
    }
}
