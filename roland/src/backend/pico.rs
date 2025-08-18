use std::{sync::Arc, time::Instant};

use log::{error, info};
use tokio::sync::{RwLock, broadcast, mpsc};
use tokio_util::sync::CancellationToken;

use crate::backend::serial::TrackSensorID;

use super::serial::{SerialCMD, SerialData};

struct SensorData {
    ultra_sensor: Option<(u16, Instant)>,
    track_sensor: [bool; 4],
}

#[derive(Clone)]
pub struct Pico {
    cmd_tx: mpsc::Sender<SerialCMD>,
    sensor_data: Arc<RwLock<SensorData>>,
}

impl Pico {
    pub fn new(
        cmd_tx: mpsc::Sender<SerialCMD>,
        data_rx: broadcast::Receiver<SerialData>,
        token: CancellationToken,
    ) -> Self {
        let sensor_data = Arc::new(RwLock::new(SensorData {
            ultra_sensor: None,
            track_sensor: [false; 4],
        }));

        {
            let sensor_data = sensor_data.clone();
            tokio::spawn(async move {
                tokio::select! {
                    _ = Self::data_task(data_rx, sensor_data) => {
                        token.cancel();
                    },
                    _ = token.cancelled() => {
                        info!("Pico task shutting down");
                    }
                };
            });
        }

        Self {
            cmd_tx,
            sensor_data,
        }
    }

    async fn data_task(
        mut data_rx: broadcast::Receiver<SerialData>,
        sensor_data: Arc<RwLock<SensorData>>,
    ) {
        loop {
            match data_rx.recv().await {
                Ok(data) => {
                    let mut current = sensor_data.write().await;
                    match data {
                        SerialData::UltraSensor(dist) => {
                            current.ultra_sensor = Some((dist, Instant::now()));
                        }
                        SerialData::TrackSensor((id, val)) => {
                            current.track_sensor[match id {
                                TrackSensorID::L1 => 0,
                                TrackSensorID::L2 => 1,
                                TrackSensorID::R1 => 2,
                                TrackSensorID::R2 => 3,
                            }] = val;
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to receive serial data: {}", e);
                    break;
                }
            }
        }
    }

    /// Reset all hardware peripherals to a neutral state
    ///
    /// this should be called before terminating the program, in avoidance of some very serious
    /// consequences (RIP camera holder, you won't be forgotten)
    pub async fn reset(&mut self) -> anyhow::Result<()> {
        self.set_buzzer(0).await?;
        self.set_led(0, 0, 0).await?;
        self.set_servo(0).await?;
        self.set_motor(0, 0).await?;
        Ok(())
    }

    /// returns the latest measured distance in cm
    pub async fn get_ultra(&self) -> Option<u16> {
        let (dist, time) = self.sensor_data.read().await.ultra_sensor?;

        // don't return too old data
        if (Instant::now() - time).as_millis() > 100 {
            None
        } else {
            Some(dist)
        }
    }

    pub async fn get_track(&self) -> [bool; 4] {
        self.sensor_data.read().await.track_sensor
    }

    pub async fn set_buzzer(&mut self, freq: u16) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::Buzzer(freq)).await?;
        Ok(())
    }

    pub async fn set_led(&mut self, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::LED((r, g, b))).await?;
        Ok(())
    }

    pub async fn set_servo(&mut self, deg: i8) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::Servo(deg)).await?;
        Ok(())
    }

    pub async fn set_motor(&mut self, left: i32, right: i32) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::HBridge((left, right))).await?;
        Ok(())
    }
}
