use tokio_util::sync::CancellationToken;

use crate::backend::{pico::Pico, serial};

/// this is a
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
}
