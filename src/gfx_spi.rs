use esp32_hal::{prelude::*, Delay};
// https://docs.rs/shared-bus/latest/shared_bus/struct.BusManager.html#method.acquire_spi
// use shared_bus::{SpiProxy, NullMutex};
// use esp32_hal::{spi::SPI, peripherals::SPI2, prelude::*, Delay};

use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;

use embedded_hal::blocking::spi::Write as SPI_Write;
use embedded_hal::digital::v2::OutputPin;

use st7735_lcd::{self, Orientation, ST7735};

const FERRIS_L: &[u8] = include_bytes!("./ferris_large.raw");
const FERRIS_L_WIDTH: u32 = 86;
const LOGO2022: &[u8] = include_bytes!("./labortage2022.raw");
const LOGO2022_WIDTH: u32 = 121;

const ST7735_OFF_X: u16 = 1;
const ST7735_OFF_Y: u16 = 26;

const ST7735_RGB: bool = false;
const ST7735_INVERTED: bool = true;
const ST7735_WIDTH: u32 = 160;
const ST7735_HEIGHT: u32 = 80;

pub fn init_displays<SPI, DC, RST1>(
    spi: SPI,
    delay: &mut Delay,
    dc: DC,
    rst1: RST1,
) -> ST7735<SPI, DC, RST1>
where
    SPI: SPI_Write<u8>,
    DC: OutputPin,
    RST1: OutputPin,
{
    let mut display1 = ST7735::new(
        spi,
        dc,
        rst1,
        ST7735_RGB,
        ST7735_INVERTED,
        ST7735_WIDTH,
        ST7735_HEIGHT,
    );
    display1.init(delay).unwrap();
    display1.set_offset(ST7735_OFF_X, ST7735_OFF_Y);
    display1.set_orientation(&Orientation::Landscape).unwrap();
    display1.clear(Rgb565::BLACK).unwrap();

    // TODO: shared_bus, tuple
    /*
    let mut display2 = ST7735::new(spi2, dc, rst2, rgb, inverted, width, height);
    display2.init(&mut delay).unwrap();
    display2.set_offset(26, 1);
    display2.clear(Rgb565::BLACK).unwrap();
    display2.set_orientation(&Orientation::Landscape).unwrap();
    */

    display1
}

pub fn splash<SPI, DC, RST>(display1: &mut ST7735<SPI, DC, RST>, delay: &mut Delay)
where
    SPI: SPI_Write<u8>,
    DC: OutputPin,
    RST: OutputPin,
{
    let image_r1: ImageRawLE<Rgb565> = ImageRaw::new(LOGO2022, LOGO2022_WIDTH);
    let image_r2: ImageRawLE<Rgb565> = ImageRaw::new(FERRIS_L, FERRIS_L_WIDTH);
    let image1 = Image::new(&image_r1, Point::new(20, 0));
    let image2 = Image::new(&image_r2, Point::new(37, 10));

    image1.draw(display1).unwrap();
    delay.delay_ms(800u32);
    display1.clear(Rgb565::BLACK).unwrap();
    image2.draw(display1).unwrap();
    // image2.draw(&mut display2).unwrap();
}
