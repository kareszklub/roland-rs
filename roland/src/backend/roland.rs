use std::time::Duration;

use log::info;
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;

use crate::backend::{pico::Pico, serial};

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
}
