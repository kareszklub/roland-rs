use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Level, Pin, Pull},
    Peri,
};
use serde::Serialize;

use crate::serial::{SerialData, DATA};

#[derive(Serialize, Debug, Clone)]
pub enum TrackSensorID {
    L1,
    L2,
    R1,
    R2,
}

pub struct TrackSensor {}

#[embassy_executor::task(pool_size = 4)]
async fn track_sensor_task(mut pin: Input<'static>, id: TrackSensorID) {
    let mut state = pin.get_level() == Level::High;
    loop {
        pin.wait_for_any_edge().await;
        state = !state;
        DATA.send(SerialData::TrackSensor((id.clone(), state)))
            .await;
    }
}

impl TrackSensor {
    /// start all tasks for reading and sending track sensor data
    pub fn init(
        l1_pin: Peri<'static, impl Pin>,
        l2_pin: Peri<'static, impl Pin>,
        r1_pin: Peri<'static, impl Pin>,
        r2_pin: Peri<'static, impl Pin>,
        spawner: Spawner,
    ) {
        spawner
            .spawn(track_sensor_task(
                Input::new(l1_pin, Pull::None),
                TrackSensorID::L1,
            ))
            .unwrap();
        spawner
            .spawn(track_sensor_task(
                Input::new(l2_pin, Pull::None),
                TrackSensorID::L2,
            ))
            .unwrap();
        spawner
            .spawn(track_sensor_task(
                Input::new(r1_pin, Pull::None),
                TrackSensorID::R1,
            ))
            .unwrap();
        spawner
            .spawn(track_sensor_task(
                Input::new(r2_pin, Pull::None),
                TrackSensorID::R2,
            ))
            .unwrap();
    }
}
