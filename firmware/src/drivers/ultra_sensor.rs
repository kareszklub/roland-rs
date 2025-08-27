use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Level, Output, Pin, Pull},
    Peri,
};
use embassy_time::{with_timeout, Duration, Instant, Timer};
use heapless::Deque;

use crate::serial::{SerialData, DATA};

pub struct UltraSensor {
    trig: Output<'static>,
    echo: Input<'static>,
    data: Deque<u16, 4>,
}

#[embassy_executor::task]
async fn ultra_sensor_task(mut ultra: UltraSensor) {
    // for these, refer to the ultra sensor datasheet
    const MIN_DIST: u16 = 2;
    const MAX_DIST: u16 = 400;

    loop {
        let mut done = false;
        let _ = with_timeout(Duration::from_millis(60), async {
            ultra.trig.set_high();
            Timer::after_micros(10).await;
            ultra.trig.set_low();

            ultra.echo.wait_for_high().await;

            let rise = Instant::now();
            ultra.echo.wait_for_low().await;
            let fall = Instant::now();

            let dist = ((fall - rise).as_micros() * 343 / 10000 / 2) as u16;

            if (MIN_DIST..=MAX_DIST).contains(&dist) {
                ultra.push_data(dist);
                DATA.send(SerialData::UltraSensor(Some(ultra.get_dist().unwrap())))
                    .await;
            } else {
                DATA.send(SerialData::UltraSensor(None)).await;
            }
            done = true;

            Timer::after_millis(60).await;
        })
        .await;
        if !done {
            DATA.send(SerialData::UltraSensor(None)).await;
        }
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
            data: Deque::new(),
        };

        spawner.spawn(ultra_sensor_task(s)).unwrap();
    }

    fn push_data(&mut self, d: u16) {
        if self.data.len() == self.data.capacity() {
            self.data.pop_front();
        }
        let _ = self.data.push_back(d);
    }

    fn get_dist(&self) -> Option<u16> {
        if self.data.is_empty() {
            None
        } else {
            Some(self.data.iter().sum::<u16>() / self.data.len() as u16)
        }
    }
}
