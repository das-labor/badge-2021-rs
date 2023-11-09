use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::BinaryColor;

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
