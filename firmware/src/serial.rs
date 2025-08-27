use cobs::CobsDecoder;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{PIN_16, PIN_17, UART0};
use embassy_rp::uart::{
    BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx, Config,
};
use embassy_rp::{bind_interrupts, Peri};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embedded_io_async::{Read, Write};
use postcard::{from_bytes, to_slice};
use serde::{Deserialize, Serialize};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

use crate::drivers::track_sensor::TrackSensorID;

bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

/// pico -> pi
#[derive(Serialize, Debug)]
pub enum SerialData {
    /// measured distance in cm
    UltraSensor(Option<u16>),
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
pub static CMD: Channel<ThreadModeRawMutex, SerialCMD, 64> = Channel::new();

/// channel for outgoing messages
pub static DATA: Channel<ThreadModeRawMutex, SerialData, 64> = Channel::new();

#[embassy_executor::task]
pub async fn serial_init(
    tx_pin: Peri<'static, PIN_16>,
    rx_pin: Peri<'static, PIN_17>,
    uart: Peri<'static, UART0>,
    spawner: Spawner,
) {
    static TX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 64])[..];
    static RX_BUF: StaticCell<[u8; 64]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 64])[..];

    let uart = BufferedUart::new(
        uart,
        tx_pin,
        rx_pin,
        Irqs,
        tx_buf,
        rx_buf,
        Config::default(),
    );

    let (tx, rx) = uart.split();

    spawner.spawn(serial_write_task(tx)).unwrap();
    spawner.spawn(serial_read_task(rx)).unwrap();
}

#[embassy_executor::task]
async fn serial_read_task(mut rx: BufferedUartRx) {
    let mut cobs_buf = [0u8; 128];
    let mut sm = CobsDecoder::new(&mut cobs_buf);
    let mut read_buf = [0u8; 64];

    loop {
        if let Ok(_n) = rx.read(&mut read_buf).await {
            if let Ok(report) = sm.push(&read_buf) {
                match report {
                    Some(report) => {
                        if let Ok(cmd) = from_bytes::<SerialCMD>(&sm.dest()[..report.frame_size()])
                        {
                            CMD.send(cmd).await;
                        }
                    }
                    None => (),
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn serial_write_task(mut tx: BufferedUartTx) {
    let mut cobs_buf = [0u8; 128];
    let mut write_buf = [0u8; 64];

    loop {
        let data = DATA.receive().await;
        let data = to_slice(&data, &mut write_buf).unwrap();

        let len = cobs::encode(&data, &mut cobs_buf);
        cobs_buf[len] = 0x00;
        let _ = tx.write_all(&cobs_buf[..=len]).await;
    }
}
