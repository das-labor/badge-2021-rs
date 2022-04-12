#![no_std]
#![no_main]

use core::{cell::RefCell, fmt::Write};
use embedded_graphics::mono_font::{
    ascii::{FONT_10X20, FONT_6X10},
    MonoTextStyle,
};
use embedded_graphics::pixelcolor::*;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::*;
use embedded_graphics::text::*;
use esp32::{I2C0, UART0};
use esp32_hal::{gpio, i2c, pac::Peripherals, prelude::*, RtcCntl, Serial, Timer};
use nb::block;
use panic_write::PanicHandler;
use ssd1306;
use ssd1306::mode::DisplayConfig;
use xtensa_lx_rt as _;
use xtensa_lx_rt::entry;

fn draw<D>(display: &mut D, s: &mut Serial<UART0>) -> Result<(), D::Error>
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
    Text::with_baseline(
        ">> Das Labor <<", // Yes.
        Point::new(3, 3),
        text_style,
        Baseline::Top,
    )
    .draw(display)?;

    Text::new(
        "Write Rust!",
        Point::new(10, (display.bounding_box().size.height - 10) as i32 / 2),
        MonoTextStyle::new(&FONT_10X20, Rgb565::WHITE.into()),
    )
    .draw(display)?;

    Ok(())
}

// display 1
fn ssd1306g_1(i2c: i2c::I2C<I2C0>, s: &mut Serial<UART0>) -> Result<(), ()> {
    writeln!(s, "Initialize SSD1306 I2C display 1").unwrap();
    let di = ssd1306::I2CDisplayInterface::new(i2c);
    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    writeln!(s, "{:#?}", display.init()).unwrap();
    draw(&mut display, s).unwrap();
    display.flush().unwrap();

    Ok(())
}

// display 2
fn ssd1306g_2(i2c: i2c::I2C<I2C0>, s: &mut Serial<UART0>) -> Result<(), ()> {
    writeln!(s, "Initialize SSD1306 I2C display 2").unwrap();
    let di = ssd1306::I2CDisplayInterface::new_alternate_address(i2c);
    let mut display = ssd1306::Ssd1306::new(
        di,
        ssd1306::size::DisplaySize128x64,
        ssd1306::rotation::DisplayRotation::Rotate0,
    )
    .into_buffered_graphics_mode();

    writeln!(s, "{:#?}", display.init()).unwrap();
    draw(&mut display, s).unwrap();
    display.flush().unwrap();

    Ok(())
}

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();

    let mut rtccntl = RtcCntl::new(peripherals.RTC_CNTL);
    let mut timer0 = Timer::new(peripherals.TIMG0);
    let mut timer1 = Timer::new(peripherals.TIMG1);
    let serial0 = Serial::new(peripherals.UART0).unwrap();
    // There is NO NEED TO PANIC! EVERYTHING IS OK!
    let mut serial = PanicHandler::new(serial0);

    // Disable watchdog timer
    rtccntl.set_wdt_global_enable(false);

    timer0.disable();
    timer1.disable();

    timer0.start(30_000_000u64);

    let io = gpio::IO::new(peripherals.GPIO, peripherals.IO_MUX);

    let mut led = io.pins.gpio4.into_push_pull_output();

    /* I2C OLED display */
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
    // TODO: let shared_i2c = RefCell::new(i2c);
    // see https://github.com/rust-embedded/embedded-hal/issues/35
    ssd1306g_1(i2c, &mut serial).unwrap();
    // ssd1306g_2(i2c, &mut serial).unwrap();

    /* main loop :) */
    loop {
        writeln!(serial, "====== ON =====\r").unwrap();
        led.set_high().unwrap();
        block!(timer0.wait()).unwrap();
        writeln!(serial, "===== OFF =====\r").unwrap();
        led.set_low().unwrap();
        block!(timer0.wait()).unwrap();
    }
}
