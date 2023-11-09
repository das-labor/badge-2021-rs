use embedded_graphics::image::{Image, ImageRaw, ImageRawLE};
use embedded_graphics::pixelcolor::{raw::LittleEndian, BinaryColor, Rgb565};
use embedded_graphics::prelude::*;

/// Graphics for the SPI TFT color displays
/// Converted with ffmpeg, see
/// # https://github.com/ImageMagick/ImageMagick/discussions/2787
/// ffmpeg -vcodec png -i image.png -vcodec rawvideo -f rawvideo -pix_fmt rgb565 image.raw
pub type Img<'a> = Image<'a, ImageRaw<'a, Rgb565, LittleEndian>>;

const FERRIS_L: &[u8] = include_bytes!("./ferris_large.raw");
const FERRIS_L_WIDTH: u32 = 86;
const FERRIS_L_RAW: ImageRawLE<Rgb565> = ImageRaw::new(FERRIS_L, FERRIS_L_WIDTH);
const FERRIS_L_POS: Point = Point::new(37, 10);
pub const FERRIS_L_IMG: Img = Image::new(&FERRIS_L_RAW, FERRIS_L_POS);

const LOGO2022: &[u8] = include_bytes!("./labortage2022.raw");
const LOGO2022_WIDTH: u32 = 121;
const LOGO2022_RAW: ImageRawLE<Rgb565> = ImageRaw::new(LOGO2022, LOGO2022_WIDTH);
const LOGO2022_POS: Point = Point::new(20, 0);
pub const LOGO2022_IMG: Img = Image::new(&LOGO2022_RAW, LOGO2022_POS);

// Images can be converted via ImageMagick, then renamed to *.raw:
// `convert image.bmp -depth 1 -monochrome image.gray`
const ANT1B: &[u8] = include_bytes!("./ant1.raw");
const ANT2B: &[u8] = include_bytes!("./ant2.raw");
const ANT3B: &[u8] = include_bytes!("./ant3.raw");
const LOGO_2021: &[u8] = include_bytes!("./labortage2021.raw");
const LOGO_2023: &[u8] = include_bytes!("./labortage2023.raw");
const RUST: &[u8] = include_bytes!("./rust.raw");
const FERRIS: &[u8] = include_bytes!("./ferris.raw");

pub const ANT1B_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(ANT1B, 128);
pub const ANT2B_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(ANT2B, 64);
pub const ANT3B_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(ANT3B, 64);
pub const LOGO_2021_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(LOGO_2021, 64);
pub const LOGO_2023_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(LOGO_2023, 64);
pub const RUST_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(RUST, 64);

pub const FERRIS_RAW: ImageRaw<BinaryColor> = ImageRaw::<BinaryColor>::new(FERRIS, 128);
