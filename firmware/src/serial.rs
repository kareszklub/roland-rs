use cobs::CobsDecoder;
use embassy_executor::Spawner;
use embassy_rp::peripherals::{PIN_16, PIN_17, UART0};
use embassy_rp::uart::{
    BufferedInterruptHandler, BufferedUart, BufferedUartRx, BufferedUartTx, Config,
};
use embassy_rp::{bind_interrupts, Peri};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embedded_io_async::{BufRead, Read, Write};
use log::debug;
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
    Reset,
}

/// channel for incoming messages
pub static CMD: Channel<ThreadModeRawMutex, SerialCMD, 64> = Channel::new();

/// channel for outgoing messages
pub static DATA: Channel<ThreadModeRawMutex, SerialData, 64> = Channel::new();

static BUF_SIZE: usize = 4096;

#[embassy_executor::task]
pub async fn serial_init(
    tx_pin: Peri<'static, PIN_16>,
    rx_pin: Peri<'static, PIN_17>,
    uart: Peri<'static, UART0>,
    spawner: Spawner,
) {
    static TX_BUF: StaticCell<[u8; BUF_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; BUF_SIZE])[..];
    static RX_BUF: StaticCell<[u8; BUF_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; BUF_SIZE])[..];

    let mut config = Config::default();
    config.baudrate = 115200;
    let uart = BufferedUart::new(uart, tx_pin, rx_pin, Irqs, tx_buf, rx_buf, config);

    let (tx, rx) = uart.split();

    spawner.spawn(serial_write_task(tx)).unwrap();
    spawner.spawn(serial_read_task(rx)).unwrap();
}

#[embassy_executor::task]
async fn serial_read_task(mut rx: BufferedUartRx) {
    let mut cobs_buf = [0u8; BUF_SIZE];
    let mut sm = CobsDecoder::new(&mut cobs_buf);
    let mut read_buf = [0u8; BUF_SIZE];

    loop {
        match rx.read(&mut read_buf).await {
            Ok(n) => {
                let mut consumed: usize = 0;

                while consumed < n {
                    match sm.push(&read_buf[consumed..n]) {
                        Ok(Some(report)) => {
                            let data = &sm.dest()[..report.frame_size()];

                            match from_bytes::<SerialCMD>(data) {
                                Ok(cmd) => {
                                    CMD.send(cmd).await;
                                }
                                Err(e) => debug!("Error deserializing CMD: {:?}", e),
                            }

                            consumed += report.parsed_size();
                        }
                        Ok(None) => consumed = n,
                        Err(e) => {
                            debug!("Error pushing to state machine: {:?}", e);
                            consumed = n;
                        }
                    }
                }
            }
            Err(e) => debug!("Error reading rx buffer: {:?}", e),
        }
    }
}

#[embassy_executor::task]
async fn serial_write_task(mut tx: BufferedUartTx) {
    let mut cobs_buf = [0u8; 64];
    let mut write_buf = [0u8; 64];

    loop {
        let data = DATA.receive().await;

        // debug!("Sent Data: {:?}", data);

        let data = to_slice(&data, &mut write_buf).unwrap();

        let len = cobs::encode(&data, &mut cobs_buf);
        cobs_buf[len] = 0x00;
        let _ = tx.write_all(&cobs_buf[..=len]).await;
    }
}
