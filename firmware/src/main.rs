#![no_std]
#![no_main]

use embassy_executor::Spawner;
use {defmt_rtt as _, panic_probe as _};

use crate::hardware::Hardware;

mod drivers;
mod hardware;
mod serial;

// firmware metadata
#[unsafe(link_section = ".bi_entries")]
#[used]
pub static PICOTOOL_ENTRIES: [embassy_rp::binary_info::EntryAddr; 4] = [
    embassy_rp::binary_info::rp_program_name!(c"Roland uC"),
    embassy_rp::binary_info::rp_program_description!(c"Roland uC firmware written in Rust"),
    embassy_rp::binary_info::rp_cargo_version!(),
    embassy_rp::binary_info::rp_program_build_attribute!(),
];

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    Hardware::init(embassy_rp::init(Default::default()), spawner).await;
}
