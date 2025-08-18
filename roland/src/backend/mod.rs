use tokio_util::sync::CancellationToken;

use crate::backend::pico::Pico;

pub mod pico;
mod serial;

#[derive(Clone)]
pub struct Backend {
    pub pico: Pico,
}

impl Backend {
    pub async fn init(token: CancellationToken) -> anyhow::Result<Self> {
        Ok(Self {
            pico: serial::init(token).await?,
        })
    }
}
