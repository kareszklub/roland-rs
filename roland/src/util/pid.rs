use std::time::Instant;

use log::info;

pub struct PID {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,

    int: f64,
    pub int_min: f64,
    pub int_max: f64,

    last_t: Option<Instant>,
    last_e: f64,

    pub sp: f64,
}

impl PID {
    pub fn new(kp: f64, ki: f64, kd: f64, int_min: f64, int_max: f64, sp: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            int: 0.,
            int_min,
            int_max,
            last_t: None,
            last_e: 0.,
            sp,
        }
    }

    pub fn step(&mut self, pv: f64) -> f64 {
        let now = Instant::now();

        match self.last_t {
            Some(last_t) => {
                let dt = (now - last_t).as_secs_f64();
                self.last_t = Some(now);

                let e = self.sp - pv;

                self.int = (self.int + e * dt).clamp(self.int_min, self.int_max);

                let p = self.kp * e;
                let i = self.ki * self.int;
                let d = self.kd * (e - self.last_e) / dt;

                self.last_e = e;

                p + i + d
            }
            // in the first step, no time reference point is available, so only proportional can be
            // safely calculated
            None => {
                self.last_t = Some(now);
                self.kp * (self.sp - pv)
            }
        }
    }
}
