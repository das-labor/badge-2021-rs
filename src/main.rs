#![no_std]
#![no_main]

// extern crate shared_bus;
use core::{cell::RefCell, fmt::Write, str};
use embedded_graphics::mono_font::{
    ascii::{FONT_10X20, FONT_6X10},
    MonoTextStyle,
};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use esp32::UART0;
use esp32_hal::{
    ehal::digital::v2::InputPin,
    gpio::{Gpio12, Gpio2, IO},
    i2c,
    pac::{Interrupt, Peripherals},
    prelude::*,
    RtcCntl, Serial, Timer,
};
use esp_hal_common::{
    gpio::{Event, Pin},
    interrupt, Cpu, Input, PullDown, PullUp,
};
use nb::block;
use panic_write::PanicHandler;
use shared_bus;
use ssd1306;
use ssd1306::mode::DisplayConfig;
use xtensa_lx::mutex::{Mutex, SpinLockMutex};
use xtensa_lx_rt as _;
use xtensa_lx_rt::entry;

fn draw<'a, D>(
    display: &mut D,
    //    s: &mut Serial<UART0>,
    title: &'a str,
    msg: &'a str,
) -> Result<(), D::Error>
where
    D: DrawTarget + Dimensions,
    D::Color: From<Rgb565>,
{
    display.clear(Rgb565::BLACK.into())?;

    Rectangle::new(display.bounding_box().top_left, display.bounding_box().size)
        .into_styled(
            PrimitiveStyleBuilder::new()
                .fill_color(Rgb565::BLUE.into())
                .stroke_color(Rgb565::YELLOW.into())
                .stroke_width(1)
                .build(),
        )
        .draw(display)?;

    let text_style = MonoTextStyle::new(&FONT_6X10, Rgb565::WHITE.into());
    Text::with_baseline(title, Point::new(3, 3), text_style, Baseline::Top).draw(display)?;

    Text::new(
        msg,
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    Ok(())
}

static mut SERIAL: SpinLockMutex<RefCell<Option<Serial<UART0>>>> =
    SpinLockMutex::new(RefCell::new(None));
static mut JBTN2: SpinLockMutex<RefCell<Option<Gpio12<Input<PullDown>>>>> =
    SpinLockMutex::new(RefCell::new(None));
static mut PBTN2: SpinLockMutex<RefCell<Option<Gpio2<Input<PullUp>>>>> =
    SpinLockMutex::new(RefCell::new(None));

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut rtccntl = RtcCntl::new(peripherals.RTC_CNTL);
    let mut timer0 = Timer::new(peripherals.TIMG0);
    let mut timer1 = Timer::new(peripherals.TIMG1);
    let serial0 = Serial::new(peripherals.UART0).unwrap();
    // There is NO NEED TO PANIC! EVERYTHING IS OK!
    // let mut serial = PanicHandler::new(serial0);

    // Disable watchdog timer
    rtccntl.set_wdt_global_enable(false);

    timer0.disable();
    timer1.disable();

    timer0.start(30_000u64);

    let io = IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio4.into_push_pull_output();
    let mut pbtn2 = io.pins.gpio2.into_pull_up_input();
    let mut jbtn2 = io.pins.gpio12.into_pull_down_input();
    jbtn2.listen(Event::FallingEdge);
    pbtn2.listen(Event::FallingEdge);

    // https://github.com/esp-rs/esp-hal/blob/main/esp32-hal/examples/gpio_interrupt.rs
    unsafe {
        (&SERIAL).lock(|data| (*data).replace(Some(serial0)));
        (&PBTN2).lock(|data| (*data).replace(Some(pbtn2)));
        (&JBTN2).lock(|data| (*data).replace(Some(jbtn2)));
    }

    interrupt::enable(
        Cpu::ProCpu,
        Interrupt::GPIO,
        interrupt::CpuInterrupt::Interrupt1LevelPriority1,
    );

    /* I2C OLED displays */
    let sda = io.pins.gpio21;
    let scl = io.pins.gpio22;

    let i2c = i2c::I2C::new(
        peripherals.I2C0,
        sda,
        scl,
        400_000, // 400kHz
        &mut (peripherals.DPORT),
    )
    .unwrap();
    // Instantiate
    let i2c_bus = shared_bus::BusManagerSimple::new(i2c);
    let di1 = ssd1306::I2CDisplayInterface::new(i2c_bus.acquire_i2c());
    let di2 = ssd1306::I2CDisplayInterface::new_alternate_address(i2c_bus.acquire_i2c());
    // Initialize
    let mut d1 = ssd1306::Ssd1306::new(
        di1,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    // writeln!(serial, "{:#?}", d1.init()).unwrap();

    let mut d2 = ssd1306::Ssd1306::new(
        di2,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();
    // writeln!(serial, "{:#?}", d2.init()).unwrap();
    // Draw! :)
    draw(
        &mut d1,
        /* &mut serial, */ ">> Das Labor <<",
        "Write Rust!",
    )
    .unwrap();
    d1.flush().unwrap();

    draw(
        &mut d2,
        /* &mut serial, */ "\\o/ *woop woop* \\o/",
        "Party hard!",
    )
    .unwrap();
    d2.flush().unwrap();

    let mut t: u32 = 0;

    /* main loop :) */
    loop {
        if t == 500 {
            // writeln!(serial, "====== ON =====\r").unwrap();
            led.set_high().unwrap();
        }
        if t == 1000 {
            // writeln!(serial, "===== OFF =====\r").unwrap();
            led.set_low().unwrap();
        }
        /*
        if jbtn2.is_low().unwrap() {
            writeln!(serial, "== JBTN2 LOW ==\r").unwrap();
        }
        if jbtn2.is_high().unwrap() {
            writeln!(serial, "== JBTN2 HIGH =\r").unwrap();
            draw(&mut d2, &mut serial, "btntonz buttonz", "Push harder!").unwrap();
            d2.flush().unwrap();
        }
        if pbtn2.is_low().unwrap_or(false) {
            writeln!(serial, "== SBTN2 LOW ==\r").unwrap();
        } else {
            writeln!(serial, "== SBTN2 HIGH =\r").unwrap();
        }
        if pbtn2.is_high().unwrap() {
            writeln!(serial, "== SBTN2 HIGH =\r").unwrap();
            draw(&mut d2, &mut serial, "btntonz buttonz", "Push harder!").unwrap();
            d2.flush().unwrap();
        }
        */
        t += 1;
        if t > 1000 {
            t = 1;
        }
        block!(timer0.wait()).unwrap();
    }
}

#[no_mangle]
pub fn level1_interrupt() {
    unsafe {
        (&SERIAL).lock(|data| {
            let mut serial = data.borrow_mut();
            let serial = serial.as_mut().unwrap();
            writeln!(serial, "Interrupt").ok();
        });
    }

    interrupt::clear(
        Cpu::ProCpu,
        interrupt::CpuInterrupt::Interrupt1LevelPriority1,
    );

    unsafe {
        (&PBTN2).lock(|data| {
            let mut button = data.borrow_mut();
            let button = button.as_mut().unwrap();
            button.clear_interrupt();
        });
    }

    unsafe {
        (&JBTN2).lock(|data| {
            let mut button = data.borrow_mut();
            let button = button.as_mut().unwrap();
            button.clear_interrupt();
        });
    }
}
