use crate::shared_pin::SharedPin;
use core::cell::RefCell;
use esp32_hal::{
    gpio::{Gpio16, Output, PushPull},
    prelude::*,
    spi::{master::Spi, FullDuplexMode},
    Delay,
};
use shared_bus::{BusManager, NullMutex, SpiProxy};
use static_cell::StaticCell;

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

type SpiMutex<'a, S, M> = NullMutex<Spi<'a, S, M>>;

type DcPin = Gpio16<Output<PushPull>>;

static DC_PIN: StaticCell<RefCell<DcPin>> = StaticCell::new();

pub fn init_displays<'a, SPI, RST1, RST2, CS1, CS2>(
    spi_bus: &'a BusManager<SpiMutex<'a, SPI, FullDuplexMode>>,
    delay: &mut Delay,
    dc: DcPin,
    rst1: RST1,
    rst2: RST2,
    cs1: &mut CS1,
    cs2: &mut CS2,
) -> (
    ST7735<SpiProxy<'a, SpiMutex<'a, SPI, FullDuplexMode>>, SharedPin<DcPin>, RST1>,
    ST7735<SpiProxy<'a, SpiMutex<'a, SPI, FullDuplexMode>>, SharedPin<DcPin>, RST2>,
)
where
    SpiProxy<'a, SpiMutex<'a, SPI, FullDuplexMode>>: SPI_Write<u8>,
    RST1: OutputPin,
    RST2: OutputPin,
    CS1: OutputPin,
    CS2: OutputPin,
{
    let dc_pin: &'static mut RefCell<DcPin> = DC_PIN.init(dc.into());

    let _ = cs2.set_high();
    let _ = cs1.set_low();
    let mut display1 = ST7735::new(
        spi_bus.acquire_spi(),
        SharedPin(dc_pin),
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

    let _ = cs1.set_high();
    let _ = cs2.set_low();
    let mut display2 = ST7735::new(
        spi_bus.acquire_spi(),
        SharedPin(dc_pin),
        rst2,
        ST7735_RGB,
        ST7735_INVERTED,
        ST7735_WIDTH,
        ST7735_HEIGHT,
    );
    display2.init(delay).unwrap();
    display2.set_offset(ST7735_OFF_X, ST7735_OFF_Y);
    display2.set_orientation(&Orientation::Landscape).unwrap();
    display2.clear(Rgb565::BLACK).unwrap();

    (display1, display2)
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
