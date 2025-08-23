use log::{debug, error, info};
use tokio::sync::{broadcast, mpsc, watch};
use tokio_util::sync::CancellationToken;

use crate::backend::{
    pico::sensors::{Sensors, TrackData, UltraData},
    serial::{SerialCMD, SerialData, TrackSensorID},
};

mod sensors;

/// wrapper around the pico serial communication channels used for state management and other
/// abstractions
/// this can be cheaply cloned
#[derive(Clone)]
pub struct Pico {
    cmd_tx: mpsc::Sender<SerialCMD>,
    sensor_data: Sensors,
}

impl Pico {
    pub fn new(
        cmd_tx: mpsc::Sender<SerialCMD>,
        data_rx: broadcast::Receiver<SerialData>,
        token: CancellationToken,
    ) -> Self {
        let (ultra_sensor, _) = watch::channel(None);
        let (track_sensor, _) = watch::channel([false; 4]);

        let sensor_data = Sensors {
            ultra_sensor,
            track_sensor,
        };

        {
            let sensor_data = sensor_data.clone();
            tokio::spawn(async move {
                tokio::select! {
                    ret = Self::data_task(data_rx, sensor_data) => {
                        match ret {
                            Ok(()) => debug!("[Pico Data] task shutting down"),
                            Err(e) => error!("[Pico Data] task shutting down: {}",e),
                        }
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
        sensor_data: Sensors,
    ) -> anyhow::Result<()> {
        loop {
            match data_rx.recv().await? {
                SerialData::UltraSensor(dist) => {
                    sensor_data.ultra_sensor.send_replace(Some(dist));
                }
                SerialData::TrackSensor((id, val)) => {
                    let mut current = *sensor_data.track_sensor.borrow();

                    current[match id {
                        TrackSensorID::L1 => 0,
                        TrackSensorID::L2 => 1,
                        TrackSensorID::R1 => 2,
                        TrackSensorID::R2 => 3,
                    }] = val;

                    sensor_data.track_sensor.send_replace(current);
                }
            }
        }
    }

    /// Reset all hardware peripherals to a neutral state
    /// WARN: calling this function causes the serial to close
    ///
    ///
    /// this should be called before terminating the program, in avoidance of some very serious
    /// consequences (RIP camera holder, you won't be forgotten)
    pub async fn reset(&mut self) -> anyhow::Result<()> {
        self.cmd_tx
            .send(SerialCMD::Reset(vec![
                SerialCMD::Buzzer(0),
                SerialCMD::LED((0, 0, 0)),
                SerialCMD::Servo(0),
                SerialCMD::HBridge((0, 0)),
            ]))
            .await?;
        Ok(())
    }

    /// get a receiver handle for the ultra sensor
    pub fn subscribe_ultra(&self) -> watch::Receiver<UltraData> {
        self.sensor_data.ultra_sensor.subscribe()
    }

    /// gets the current state of the track sensor
    pub fn get_track(&self) -> [bool; 4] {
        *self.sensor_data.track_sensor.borrow()
    }

    /// get a receiver handle for the track sensor
    pub fn subscribe_track(&self) -> watch::Receiver<TrackData> {
        self.sensor_data.track_sensor.subscribe()
    }

    /// sets the buzzer to the specified frequency (Hz)
    /// NOTE: a `freq` of `0` turns off the buzzer
    pub async fn set_buzzer(&mut self, freq: u16) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::Buzzer(freq)).await?;
        Ok(())
    }

    /// sets the RGB LEDs to the specified rgb color (0 to 255)
    pub async fn set_led(&mut self, r: u8, g: u8, b: u8) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::LED((r, g, b))).await?;
        Ok(())
    }

    /// sets the servo to the specified orientation (-90° to 90°, 0° is the midpoint)
    pub async fn set_servo(&mut self, deg: i8) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::Servo(deg)).await?;
        Ok(())
    }

    /// sets the motor speeds as specified (both -0xffff to 0xffff, sign means direction)
    pub async fn set_motor(&mut self, left: i32, right: i32) -> anyhow::Result<()> {
        self.cmd_tx.send(SerialCMD::HBridge((left, right))).await?;
        Ok(())
    }
}
