use image::Rgba;
use image::imageops::FilterType;

pub static WHITE: Rgba<u8> = Rgba([255_u8, 255, 255, 255]);
pub static GRAY: Rgba<u8> = Rgba([200, 200, 200, 255]);
pub static FILTER_TYPE: FilterType = FilterType::Lanczos3;
