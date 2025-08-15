use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Level, Output, Pin, Pull},
    Peri,
};
use embassy_time::{Instant, Timer};

use crate::serial::{SerialData, DATA};

pub struct UltraSensor {
    trig: Output<'static>,
    echo: Input<'static>,
}

#[embassy_executor::task]
async fn ultra_sensor_task(mut ultra: UltraSensor) {
    loop {
        ultra.trig.set_high();
        Timer::after_micros(10).await;
        ultra.trig.set_low();

        ultra.echo.wait_for_high().await;

        let rise = Instant::now();
        ultra.echo.wait_for_low().await;
        let fall = Instant::now();

        let dist = (fall - rise).as_micros() * 350 / 10000 / 2;
        DATA.send(SerialData::UltraSensor(dist as u16)).await;

        Timer::after_millis(60).await;
    }
}

impl UltraSensor {
    /// initialize an ultra sensor and start the task
    pub fn init(
        trig_pin: Peri<'static, impl Pin>,
        echo_pin: Peri<'static, impl Pin>,
        spawner: Spawner,
    ) {
        let s = Self {
            trig: Output::new(trig_pin, Level::Low),
            echo: Input::new(echo_pin, Pull::None),
        };

        spawner.spawn(ultra_sensor_task(s)).unwrap();
    }
}
