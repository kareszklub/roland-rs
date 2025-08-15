use embassy_time::Timer;

use crate::{
    serial::{SerialCMD, CMD},
    util::color::{HSV, RGB},
};

#[embassy_executor::task]
pub async fn servo_test() {
    loop {
        for i in 0..90 {
            CMD.send(SerialCMD::Servo(i)).await;
            Timer::after_millis(5).await;
        }
        for i in (-90..90).rev() {
            CMD.send(SerialCMD::Servo(i)).await;
            Timer::after_millis(5).await;
        }
        for i in -90..=0 {
            CMD.send(SerialCMD::Servo(i)).await;
            Timer::after_millis(5).await;
        }
    }
}

#[embassy_executor::task]
pub async fn rgb_led_test() {
    loop {
        for i in 0..=360 {
            let rgb = RGB::from_hsv(&HSV {
                h: i as f64,
                s: 1.0,
                v: 1.0,
            });

            CMD.send(SerialCMD::LED((rgb.r, rgb.g, rgb.b))).await;

            Timer::after_millis(2000 / 360).await;
        }
    }
}

#[embassy_executor::task]
pub async fn buzzer_test() {
    loop {
        for f in [
            130.81, 138.59, 146.83, 155.56, 164.81, 174.61, 185.0, 196.0, 207.65, 220.0, 233.08,
            246.94,
        ] {
            CMD.send(SerialCMD::Buzzer(f as u16)).await;
            Timer::after_millis(500).await;
        }
    }
}

#[embassy_executor::task]
pub async fn motor_test() {
    loop {
        for s in [0xffff, -0xffff] {
            CMD.send(SerialCMD::HBridge((s, s))).await;
            Timer::after_secs(3).await;
        }
    }
}
