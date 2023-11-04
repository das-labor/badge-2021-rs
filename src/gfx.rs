use embedded_graphics::{
    image::{Image, ImageRaw},
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
use shared_bus::{I2cProxy, NullMutex};
use ssd1306::size::DisplaySize128x64;
use ssd1306::{mode::BufferedGraphicsMode, prelude::I2CInterface, Ssd1306};

// Images can be converted via ImageMagick, then renamed to *.raw:
// `convert image.bmp -depth 1 -monochrome image.gray`
const ANT1B: &[u8] = include_bytes!("./ant1.raw");
const ANT2B: &[u8] = include_bytes!("./ant2.raw");
const ANT3B: &[u8] = include_bytes!("./ant3.raw");
const LOGO_2021: &[u8] = include_bytes!("./labortage2021.raw");
const LOGO_2023: &[u8] = include_bytes!("./labortage2023.raw");
const RUST: &[u8] = include_bytes!("./rust.raw");
const FERRIS: &[u8] = include_bytes!("./ferris.raw");

pub type LCD128x64<'a> = Ssd1306<
    I2CInterface<I2cProxy<'a, NullMutex<I2C<'a, I2C0>>>>,
    DisplaySize128x64,
    BufferedGraphicsMode<DisplaySize128x64>,
>;

pub fn splash<'a>(d1: &mut LCD128x64, d2: &mut LCD128x64, delay: &mut Delay) {
    let l1 = ImageRaw::<BinaryColor>::new(ANT2B, 64);
    let l2 = ImageRaw::<BinaryColor>::new(RUST, 64);
    let r1 = ImageRaw::<BinaryColor>::new(LOGO_2021, 64);
    let r2 = ImageRaw::<BinaryColor>::new(ANT3B, 64);

    let il1 = Image::new(&l1, Point::new(0, 0));
    let il2 = Image::new(&l2, Point::new(64, 0));
    let ir1 = Image::new(&r1, Point::new(0, 0));
    let ir2 = Image::new(&r2, Point::new(64, 0));

    il1.draw(d1).unwrap();
    il2.draw(d1).unwrap();
    ir1.draw(d2).unwrap();
    ir2.draw(d2).unwrap();

    d1.flush().unwrap();
    d2.flush().unwrap();
    delay.delay_ms(1000u32);

    let l = ImageRaw::<BinaryColor>::new(ANT1B, 128);
    let r = ImageRaw::<BinaryColor>::new(FERRIS, 128);
    let il = Image::new(&l, Point::new(0, 0));
    let ir = Image::new(&r, Point::new(0, 0));
    il.draw(d1).unwrap();
    ir.draw(d2).unwrap();
    d1.flush().unwrap();
    d2.flush().unwrap();
    delay.delay_ms(1000u32);

    let l = ImageRaw::<BinaryColor>::new(LOGO_2023, 64);
    let il = Image::new(&l, Point::new(32, 0));
    il.draw(d1).unwrap();
    d1.flush().unwrap();
    delay.delay_ms(1000u32);
}

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
