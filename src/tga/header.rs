use crate::utils::compat::{String, ToString, format};

#[derive(Debug, Clone, Copy)]
pub enum ImageType {
    NoImageData = 0,
    ColorMapped = 1,
    TrueColor = 2,
    Grayscale = 3,
    RleColorMapped = 9,
    RleTrueColor = 10,
    RleGrayscale = 11,
}

impl ImageType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::NoImageData),
            1 => Some(Self::ColorMapped),
            2 => Some(Self::TrueColor),
            3 => Some(Self::Grayscale),
            9 => Some(Self::RleColorMapped),
            10 => Some(Self::RleTrueColor),
            11 => Some(Self::RleGrayscale),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct TgaHeader {
    pub id_length: u8,
    pub color_map_type: u8,
    pub image_type: ImageType,
    pub color_map_origin: u16,
    pub color_map_length: u16,
    pub color_map_depth: u8,
    pub x_origin: u16,
    pub y_origin: u16,
    pub width: u16,
    pub height: u16,
    pub pixel_depth: u8,
    pub image_descriptor: u8,
}

impl TgaHeader {
    pub fn parse(data: &[u8]) -> Result<Self, String> {
        if data.len() < 18 {
            return Err("File too small for TGA header".to_string());
        }

        let image_type = ImageType::from_u8(data[2])
            .ok_or_else(|| format!("Unsupported TGA image type: {}", data[2]))?;

        Ok(Self {
            id_length: data[0],
            color_map_type: data[1],
            image_type,
            color_map_origin: u16::from_le_bytes([data[3], data[4]]),
            color_map_length: u16::from_le_bytes([data[5], data[6]]),
            color_map_depth: data[7],
            x_origin: u16::from_le_bytes([data[8], data[9]]),
            y_origin: u16::from_le_bytes([data[10], data[11]]),
            width: u16::from_le_bytes([data[12], data[13]]),
            height: u16::from_le_bytes([data[14], data[15]]),
            pixel_depth: data[16],
            image_descriptor: data[17],
        })
    }

    pub fn is_top_left(&self) -> bool {
        (self.image_descriptor & 0x20) != 0
    }
}
