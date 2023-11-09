use embedded_graphics::{
    image::Image,
    mono_font::{
        ascii::{FONT_10X20, FONT_6X10},
        MonoTextStyle,
    },
    pixelcolor::*,
    prelude::*,
    primitives::*,
    text::*,
};
use esp32_hal::{i2c::I2C, peripherals::I2C0, prelude::*, Delay};
use shared_bus::{BusManager, I2cProxy, NullMutex};

use ssd1306::{
    mode::{BasicMode, BufferedGraphicsMode /* , DisplayConfig */},
    prelude::I2CInterface,
    rotation::DisplayRotation::Rotate0,
    size::DisplaySize128x64,
    I2CDisplayInterface, Ssd1306,
};

use crate::res;

type I2cMutex<'a> = NullMutex<I2C<'a, I2C0>>;
type I2cDev<'a> = I2CInterface<I2cProxy<'a, I2cMutex<'a>>>;

type BufferedDisplay128x64 = BufferedGraphicsMode<DisplaySize128x64>;
type LCD128x64<'a> = Ssd1306<I2cDev<'a>, DisplaySize128x64, BufferedDisplay128x64>;

type BasicLCD128x64<'a> = Ssd1306<I2cDev<'a>, DisplaySize128x64, BasicMode>;

// TODO: Can we get the mode change in here? Borrow checker not liketh...
pub fn init_displays<'a>(
    i2c_bus: &'a BusManager<I2cMutex<'a>>,
) -> (BasicLCD128x64<'a>, BasicLCD128x64<'a>) {
    let i2c_dev1 = I2CDisplayInterface::new(i2c_bus.acquire_i2c());
    let i2c_dev2 = I2CDisplayInterface::new_alternate_address(i2c_bus.acquire_i2c());

    let d1 = Ssd1306::new(i2c_dev1, DisplaySize128x64, Rotate0);
    // let d1 = &mut d1.into_buffered_graphics_mode();
    // d1.init().expect("display 1 init");

    let d2 = Ssd1306::new(i2c_dev2, DisplaySize128x64, Rotate0);
    // let d2 = &mut d2.into_buffered_graphics_mode();
    // d2.init().expect("display 2 init");

    (d1, d2)
}

pub fn splash<'a>(d1: &mut LCD128x64, d2: &mut LCD128x64, delay: &mut Delay) {
    let ant1 = Image::new(&res::ANT1B_RAW, Point::new(0, 0));
    let ant2 = Image::new(&res::ANT2B_RAW, Point::new(0, 0));
    let ant3 = Image::new(&res::ANT3B_RAW, Point::new(64, 0));
    let logo2021 = Image::new(&res::LOGO_2021_RAW, Point::new(0, 0));
    let rust = Image::new(&res::RUST_RAW, Point::new(64, 0));
    let ferris = Image::new(&res::FERRIS_RAW, Point::new(0, 0));
    let logo2023 = Image::new(&res::LOGO_2023_RAW, Point::new(32, 0));
    // Those fit on the same screen as pairs
    // ant2 holds hands to the right, "holding" the Rust gear logo
    // ant3 holds hands to the left, "holding" the Labortage 2021 logo
    ant2.draw(d1).unwrap();
    rust.draw(d1).unwrap();
    logo2021.draw(d2).unwrap();
    ant3.draw(d2).unwrap();
    d1.flush().unwrap();
    d2.flush().unwrap();
    delay.delay_ms(1000u32);
    // ant scratching head + Ferris the Rust mascot
    ant1.draw(d1).unwrap();
    ferris.draw(d2).unwrap();
    d1.flush().unwrap();
    d2.flush().unwrap();
    delay.delay_ms(1000u32);
    // Labortage 2023 logo
    logo2023.draw(d1).unwrap();
    d1.flush().unwrap();
    delay.delay_ms(1000u32);
}

/// Draw a message with a title on the display.
pub fn draw<'a, D>(display: &mut D, title: &'a str, msg: &'a str) -> Result<(), D::Error>
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
