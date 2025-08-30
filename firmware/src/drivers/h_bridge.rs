use embassy_rp::{
    gpio::{Input, Level, Output, Pin, Pull},
    pwm::Pwm,
    Peri,
};

use crate::drivers::pwm::PWM;

pub struct HBridge<'a> {
    l1: Output<'a>,
    l2: Output<'a>,
    r1: Output<'a>,
    r2: Output<'a>,
    pwm: PWM<'a>,
}

impl<'a> HBridge<'a> {
    pub fn new(
        l1: Peri<'a, impl Pin>,
        l2: Peri<'a, impl Pin>,
        r1: Peri<'a, impl Pin>,
        r2: Peri<'a, impl Pin>,
        pwm: Pwm<'a>,
        pwm_freq: u16,
    ) -> Self {
        let mut s = Self {
            pwm: PWM::new(pwm),
            l1: Output::new(l1, Level::Low),
            l2: Output::new(l2, Level::Low),
            r1: Output::new(r1, Level::Low),
            r2: Output::new(r2, Level::Low),
        };

        s.pwm.set_freq(pwm_freq);
        s
    }

    /// the input speed must be between -0xffff and 0xffff
    pub fn drive(&mut self, l: i32, r: i32) {
        let l = l.clamp(-0xffff, 0xffff);
        let r = r.clamp(-0xffff, 0xffff);

        self.pwm.set_duty_b(l.unsigned_abs() as u16);
        self.pwm.set_duty_a(r.unsigned_abs() as u16);

        self.l1
            .set_level(if l > 0 { Level::High } else { Level::Low });
        self.l2
            .set_level(if l < 0 { Level::High } else { Level::Low });
        self.r1
            .set_level(if r > 0 { Level::High } else { Level::Low });
        self.r2
            .set_level(if r < 0 { Level::High } else { Level::Low });
    }
}
