use embassy_executor::Spawner;
use embassy_rp::peripherals::{DMA_CH0, PIN_16, PIN_17, UART0};
use embassy_rp::uart::{Async, Config, InterruptHandler, UartTx};
use embassy_rp::{bind_interrupts, Peri};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;
use heapless::String;
use {defmt_rtt as _, panic_probe as _};

bind_interrupts!(struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
});

/// channel for debug logs
pub static LOG: Channel<ThreadModeRawMutex, String<64>, 64> = Channel::new();

#[embassy_executor::task]
async fn log_tx_task(mut tx: UartTx<'static, Async>) {
    loop {
        tx.write(b"Balls\n").await.unwrap();
        Timer::after_secs(1).await;
    }
}

macro_rules! log {
    ($($arg:tt)*) => {
        use core::fmt::Write;

        let mut buf: String<64> = String::new();
        let _ = write!(&mut buf, $($arg)*);
        LOG.send(buf).await;
    };
}

#[embassy_executor::task]
pub async fn logger_task(
    uart: Peri<'static, UART0>,
    tx_pin: Peri<'static, PIN_16>,
    // RX is currently not utilized
    _rx_pin: Peri<'static, PIN_17>,
    dma: Peri<'static, DMA_CH0>,
    spawner: Spawner,
) {
    let uart_tx = UartTx::new(uart, tx_pin, dma, Config::default());
    spawner.spawn(log_tx_task(uart_tx)).unwrap();
    // log!("Nig{}", "hts");
}
