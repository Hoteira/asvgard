use std::convert::TryFrom;

#[derive(Debug)]
pub enum Chunk {
    Header(IhdrChunk),
    Palette(PlteChunk),
    ImageData(IdatChunk),
    End(IendChunk),
    Text(TextChunk),
    Physical(PhysChunk),
    Unknown(RawChunk),
}

#[derive(Debug)]
pub struct RawChunk {
    pub length: u32,
    pub chunk_type: [u8; 4],
    pub data: Vec<u8>,
    pub crc: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorType {
    Grayscale = 0,
    RGB = 2,
    Indexed = 3,
    GrayscaleAlpha = 4,
    RGBA = 6,
}

impl TryFrom<u8> for ColorType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ColorType::Grayscale),
            2 => Ok(ColorType::RGB),
            3 => Ok(ColorType::Indexed),
            4 => Ok(ColorType::GrayscaleAlpha),
            6 => Ok(ColorType::RGBA),
            _ => Err(format!("Invalid ColorType value: {}", value)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterlaceMethod {
    None = 0,
    Adam7 = 1,
}

impl TryFrom<u8> for InterlaceMethod {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(InterlaceMethod::None),
            1 => Ok(InterlaceMethod::Adam7),
            _ => Err(format!("Invalid InterlaceMethod value: {}", value)),
        }
    }
}

#[derive(Debug)]
pub struct IhdrChunk {
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,       // 1, 2, 4, 8, 16
    pub color_type: ColorType,
    pub compression: u8,     // Must be 0 (Deflate)
    pub filter: u8,          // Must be 0 (Adaptive)
    pub interlace: InterlaceMethod,
}

#[derive(Debug, Clone, Copy)]
pub struct Rgb {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

#[derive(Debug)]
pub struct PlteChunk {
    pub palette: Vec<Rgb>,
}

#[derive(Debug)]
pub struct IdatChunk {
    pub compressed_data: Vec<u8>,
}

#[derive(Debug)]
pub struct IendChunk;

#[derive(Debug)]
pub struct TextChunk {
    pub keyword: String,
    pub text: String,
}

#[derive(Debug)]
pub struct PhysChunk {
    pub pixels_per_unit_x: u32,
    pub pixels_per_unit_y: u32,
    pub unit_specifier: u8,
}