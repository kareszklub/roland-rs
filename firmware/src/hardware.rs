use embassy_executor::Spawner;
use embassy_rp::{
    pwm::{self, Pwm},
    Peripherals,
};

use crate::{
    drivers::{
        buzzer::Buzzer, h_bridge::HBridge, rgb_led::RGBLed, servo::Servo,
        track_sensor::TrackSensor, ultra_sensor::UltraSensor,
    },
    serial::{serial_init, SerialCMD, CMD},
};

/// manages all incoming hardware commands
#[embassy_executor::task]
async fn hardware_task(mut hw: Hardware) {
    loop {
        match CMD.receive().await {
            SerialCMD::Buzzer(freq) => hw.buzzer.freq(freq),
            SerialCMD::LED((r, g, b)) => hw.led.set_color(r, g, b),
            SerialCMD::Servo(deg) => hw.servo.deg(deg),
            SerialCMD::HBridge((l_speed, r_speed)) => hw.hb.drive(l_speed, r_speed),
        }
    }
}

/// wrapper for all external peripherals
pub struct Hardware {
    buzzer: Buzzer<'static>,
    led: RGBLed<'static>,
    servo: Servo<'static>,
    hb: HBridge<'static>,
}

impl Hardware {
    /// initialize all hardware from the given peripherals singleton
    pub async fn init(p: Peripherals, spawner: Spawner) {
        spawner.spawn(serial_init(p.USB, spawner)).unwrap();

        let buzzer = Buzzer::new(Pwm::new_output_a(
            p.PWM_SLICE2,
            p.PIN_4,
            pwm::Config::default(),
        ));

        let led = RGBLed::new(
            Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_18, p.PIN_19, pwm::Config::default()),
            Pwm::new_output_a(p.PWM_SLICE3, p.PIN_22, pwm::Config::default()),
            2000,
        );

        let servo = Servo::new(
            Pwm::new_output_a(p.PWM_SLICE0, p.PIN_16, pwm::Config::default()),
            2400,
            5200,
            8520,
        );

        let hb = HBridge::new(
            Pwm::new_output_ab(p.PWM_SLICE6, p.PIN_12, p.PIN_13, pwm::Config::default()),
            Pwm::new_output_ab(p.PWM_SLICE5, p.PIN_10, p.PIN_11, pwm::Config::default()),
            p.PIN_15,
            p.PIN_14,
            2000,
            true,
            false,
        );

        UltraSensor::init(p.PIN_21, p.PIN_20, spawner);

        TrackSensor::init(p.PIN_6, p.PIN_7, p.PIN_8, p.PIN_9, spawner);

        let hw = Self {
            buzzer,
            led,
            servo,
            hb,
        };

        spawner.spawn(hardware_task(hw)).unwrap();
    }
}
