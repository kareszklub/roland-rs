use embassy_rp::{
    gpio::{Input, Level, Output, Pin, Pull},
    pwm::Pwm,
    Peri,
};

use crate::drivers::pwm::PWM;

pub struct HBridge<'a> {
    l_pwm: PWM<'a>,
    r_pwm: PWM<'a>,
    /// HIGH on enable, LOW on disable
    sleep: Output<'a>,
    /// LOW on fault condition, currently unused
    _fault: Input<'a>,
    /// software fix for a motor going in the wrong direction
    l_flip: bool,
    /// software fix for a motor going in the wrong direction
    r_flip: bool,
}

impl<'a> HBridge<'a> {
    pub fn new(
        l_pwm: Pwm<'a>,
        r_pwm: Pwm<'a>,
        sleep: Peri<'a, impl Pin>,
        fault: Peri<'a, impl Pin>,
        pwm_freq: u16,
        l_flip: bool,
        r_flip: bool,
    ) -> Self {
        let mut s = Self {
            l_pwm: PWM::new(l_pwm),
            r_pwm: PWM::new(r_pwm),
            sleep: Output::new(sleep, Level::Low),
            _fault: Input::new(fault, Pull::Up),
            l_flip,
            r_flip,
        };

        s.l_pwm.set_freq(pwm_freq);
        s.r_pwm.set_freq(pwm_freq);
        s
    }

    /// the input speed must be between -0xffff and 0xffff
    pub fn drive(&mut self, l: i32, r: i32) {
        let l = l.clamp(-0xffff, 0xffff);
        let r = r.clamp(-0xffff, 0xffff);

        let l = if self.l_flip { -l } else { l };
        let r = if self.r_flip { -r } else { r };

        if l == 0 && r == 0 {
            self.sleep.set_low();
            return;
        } else {
            self.sleep.set_high();
        }

        self.l_pwm
            .set_duty_a(if l > 0 { l.unsigned_abs() as u16 } else { 0 });
        self.l_pwm
            .set_duty_b(if l < 0 { l.unsigned_abs() as u16 } else { 0 });

        self.r_pwm
            .set_duty_a(if r > 0 { r.unsigned_abs() as u16 } else { 0 });
        self.r_pwm
            .set_duty_b(if r < 0 { r.unsigned_abs() as u16 } else { 0 });
    }
}
