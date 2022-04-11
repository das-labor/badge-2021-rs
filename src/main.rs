#![no_std]
#![no_main]

use esp32_hal::{pac::Peripherals, prelude::*};
use panic_halt as _;
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    loop {}
}
