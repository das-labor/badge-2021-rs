#![no_std]
#![no_main]

use core::fmt::Write;

use esp32_hal::{gpio, pac::Peripherals, pac::GPIO, pac::IO_MUX, prelude::*, Serial, Timer};
use nb::block;
use panic_halt as _;
use xtensa_lx_rt as _;
use xtensa_lx_rt::entry;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take().unwrap();

    let mut rtccntl = peripherals.RTC_CNTL;
    let mut timer0 = Timer::new(peripherals.TIMG0);
    let mut timer1 = Timer::new(peripherals.TIMG1);
    let mut serial0 = Serial::new(peripherals.UART0).unwrap();

    // Disable watchdog timer
    timer0.disable();
    timer1.disable();

    timer0.start(10_000_000u64);

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio2.into_push_pull_output();

    loop {
        writeln!(serial0, "====== ON =====\r").unwrap();
        led.set_high().unwrap();
        block!(timer0.wait()).unwrap();
        writeln!(serial0, "===== OFF =====\r").unwrap();
        led.set_low().unwrap();
        block!(timer0.wait()).unwrap();
    }
}
