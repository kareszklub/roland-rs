use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, Peri};
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

#[embassy_executor::task]
pub async fn logger_task(usb: Peri<'static, USB>) {
    let driver = Driver::new(usb, Irqs);
    embassy_usb_logger::run!(1024, log::LevelFilter::Trace, driver);
}
