use embassy_executor::Spawner;
use embassy_rp::peripherals::USB;
use embassy_rp::usb::{Driver, InterruptHandler};
use embassy_rp::{bind_interrupts, Peri};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use embassy_usb::class::cdc_acm::{CdcAcmClass, Receiver, Sender, State};
use embassy_usb::UsbDevice;
use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};
use static_cell::StaticCell;

use crate::drivers::track_sensor::TrackSensorID;

bind_interrupts!(pub struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

/// pico -> pi
#[derive(Serialize, Debug)]
pub enum SerialData {
    /// measured distance in cm
    UltraSensor(u16),
    /// sensor id and value
    TrackSensor((TrackSensorID, bool)),
}

/// pi -> pico
#[derive(Deserialize, Debug)]
pub enum SerialCMD {
    /// frequency (Hz)
    Buzzer(u16),
    /// RGB color (0 to 255)
    LED((u8, u8, u8)),
    /// rotation in degrees (0 is the midpoint, -90 to 90)
    Servo(i8),
    /// duty cycle (sign is direction)
    HBridge((i32, i32)),
}

/// channel for incoming messages
pub static CMD: Channel<ThreadModeRawMutex, SerialCMD, 16> = Channel::new();

/// channel for outgoing messages
pub static DATA: Channel<ThreadModeRawMutex, SerialData, 16> = Channel::new();

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver<'static, USB>>) -> ! {
    usb.run().await
}

#[embassy_executor::task]
async fn usb_read_task(mut rx: Receiver<'static, Driver<'static, USB>>) {
    let mut buf = [0u8; 64];

    loop {
        if let Ok(n) = rx.read_packet(&mut buf).await {
            if let Ok(cmd) = from_bytes::<SerialCMD>(&buf[..n]) {
                CMD.send(cmd).await;
            }
        }
    }
}

#[embassy_executor::task]
async fn usb_write_task(mut tx: Sender<'static, Driver<'static, USB>>) {
    let mut buf = [0u8; 64];

    loop {
        let data = DATA.receive().await;
        let _ = tx.write_packet(to_slice(&data, &mut buf).unwrap()).await;
    }
}

#[embassy_executor::task]
pub async fn serial_init(peri_usb: Peri<'static, USB>, spawner: Spawner) {
    let driver = Driver::new(peri_usb, Irqs);

    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe);
        config.manufacturer = Some("Kareszklub");
        config.product = Some("Roland uC firmware");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 64]> = StaticCell::new();

        let builder = embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 64]),
        );
        builder
    };

    let mut class = {
        static STATE: StaticCell<State> = StaticCell::new();
        let state = STATE.init(State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    let usb = builder.build();

    // run the USB task
    spawner.spawn(usb_task(usb)).unwrap();

    class.wait_connection().await;
    let (tx, rx) = class.split();

    spawner.spawn(usb_read_task(rx)).unwrap();
    spawner.spawn(usb_write_task(tx)).unwrap();
}
