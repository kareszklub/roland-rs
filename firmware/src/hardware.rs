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
    log::logger_task,
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
            SerialCMD::Reset => {
                hw.buzzer.freq(0);
                hw.led.set_color(0, 0, 0);
                hw.servo.deg(0);
                hw.hb.drive(0, 0);
            }
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
        spawner
            .spawn(serial_init(p.PIN_16, p.PIN_17, p.UART0, spawner))
            .unwrap();

        spawner.spawn(logger_task(p.USB)).unwrap();

        let buzzer = Buzzer::new(Pwm::new_output_a(
            p.PWM_SLICE0,
            p.PIN_0,
            pwm::Config::default(),
        ));

        let led = RGBLed::new(
            Pwm::new_output_ab(p.PWM_SLICE1, p.PIN_18, p.PIN_19, pwm::Config::default()),
            Pwm::new_output_a(p.PWM_SLICE2, p.PIN_20, pwm::Config::default()),
            2000,
        );

        let servo = Servo::new(
            Pwm::new_output_a(p.PWM_SLICE6, p.PIN_28, pwm::Config::default()),
            2100,
            4800,
            8300,
        );

        let hb = HBridge::new(
            p.PIN_13,
            p.PIN_12,
            p.PIN_11,
            p.PIN_10,
            Pwm::new_output_ab(p.PWM_SLICE7, p.PIN_14, p.PIN_15, pwm::Config::default()),
            2000,
        );

        UltraSensor::init(p.PIN_21, p.PIN_22, spawner);

        TrackSensor::init(p.PIN_2, p.PIN_3, p.PIN_4, p.PIN_5, spawner);

        let hw = Self {
            buzzer,
            led,
            servo,
            hb,
        };

        spawner.spawn(hardware_task(hw)).unwrap();
    }
}
