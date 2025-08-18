use crate::backend::pico::Pico;

pub mod pico;
mod serial;

pub struct Backend {
    pub pico: Pico,
}

impl Backend {
    pub async fn init() -> anyhow::Result<Self> {
        Ok(Self {
            pico: serial::init().await?,
        })
    }
}
