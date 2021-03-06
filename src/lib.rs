pub mod borders;
pub mod defaults;
pub mod options;
pub mod img;
pub mod types;
pub mod utils;
#[cfg(feature = "wasm")]
pub mod wasm;

pub use img::Image;

use crate::types::{Point, Size, OutputSize, Rotation};
use chrono::Utc;
use options::BorderOptions;
use image::imageops;
use std::path::PathBuf;
use image::{RgbaImage, Rgba, ImageError};
use std::cmp::{max, min};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct ImageBorders {
    img: img::Image,
    #[allow(dead_code)]
    result: Option<RgbaImage>,
}

impl ImageBorders {
    pub fn new(img: img::Image) -> ImageBorders {
        utils::set_panic_hook();
        ImageBorders { img, result: None }
    }

    #[allow(dead_code)]
    pub fn save_jpeg(
        &self,
        buffer: RgbaImage,
        output_path: Option<PathBuf>,
        quality: Option<u8>,
    ) -> Result<(), ImageError> {
        self.img.save_jpeg_to_file(buffer, output_path, quality)
    }

    pub fn save(&self, buffer: RgbaImage, output_path: Option<PathBuf>) -> Result<(), ImageError> {
        self.img.save_to_file(buffer, output_path)
    }

    pub fn apply(&mut self, options: BorderOptions) -> Result<RgbaImage, ImageError> {
        let mut size = Size {
            width: self.img.buffer.width(),
            height: self.img.buffer.height(),
        };
        if let Some(OutputSize { width, height }) = options.output_size {
            if let Some(width) = width {
                size.width = width;
            };
            if let Some(height) = height {
                size.height = height;
            };
        };

        let mut final_image = RgbaImage::new(size.width, size.height);
        for p in final_image.pixels_mut() {
            *p = if options.preview {
                defaults::GRAY
            } else {
                defaults::WHITE
            };
        }

        let mut photo = self.img.buffer.clone();
        let output_is_portrait = size.width <= size.height;
        let rem = max(size.width, size.height) as f32 / 1000.0;

        // rotate the image
        if let Some(rotate_angle) = options.rotate_angle {
            photo = match rotate_angle {
                Rotation::Rotate0 => photo,
                Rotation::Rotate90 => imageops::rotate90(&photo),
                Rotation::Rotate180 => imageops::rotate180(&photo),
                Rotation::Rotate270 => imageops::rotate270(&photo),
            };
        };

        let photo_is_portrait = photo.width() <= photo.height();

        // crop the image
        if let Some(crop_opts) = options.crop {
            let crop_top = (crop_opts.top.unwrap_or(0) as f32 * rem) as u32;
            let crop_right = photo.width() - ((crop_opts.right.unwrap_or(0) as f32 * rem) as u32);
            let crop_bottom =
                photo.height() - ((crop_opts.bottom.unwrap_or(0) as f32 * rem) as u32);
            let crop_left = (crop_opts.left.unwrap_or(0) as f32 * rem) as u32;

            let crop_width = max(0, crop_right as i64 - crop_left as i64) as u32;
            let crop_height = max(0, crop_bottom as i64 - crop_top as i64) as u32;
            photo =
                imageops::crop(&mut photo, crop_left, crop_top, crop_width, crop_height).to_image()
        };

        // resize the image to fit the screen
        let (mut fit_width, mut fit_height) = utils::resize_dimensions(
            photo.width(),
            photo.height(),
            size.width,
            size.height,
            false,
        );

        if let Some(scale_factor) = options.scale_factor {
            // scale the image by factor
            fit_width = (fit_width as f32 * utils::clamp(scale_factor, 0f32, 1f32)) as u32;
            fit_height = (fit_height as f32 * utils::clamp(scale_factor, 0f32, 1f32)) as u32;
            // println!("scaling to {} x {}", fit_width, fit_height);
        };

        let start = Utc::now().time();
        photo = imageops::resize(&photo, fit_width, fit_height, defaults::FILTER_TYPE);
        println!(
            "fitting to {} x {} took {:?}",
            fit_width,
            fit_height,
            Utc::now().time() - start,
        );

        let overlay_x = ((size.width - photo.width()) / 2) as i64;
        let overlay_y = ((size.height - photo.height()) / 2) as i64;
        // println!("overlaying at {} {}", overlay_x, overlay_y);

        // create the black borders
        if let Some(border_width) = options.border_width {
            let black_color = Rgba([0, 0, 0, 255]);
            let top_left = Point {
                x: max(
                    0,
                    overlay_x as i32 - (border_width.left as f32 * rem) as i32,
                ) as u32,
                y: max(0, overlay_y as i32 - (border_width.top as f32 * rem) as i32) as u32,
            };
            let btm_right = Point {
                x: max(
                    0,
                    (overlay_x + photo.width() as i64) as i32
                        + (border_width.right as f32 * rem) as i32,
                ) as u32,
                y: max(
                    0,
                    (overlay_y + photo.height() as i64) as i32
                        + (border_width.bottom as f32 * rem) as i32,
                ) as u32,
            };
            img::fill_rect(&mut final_image, &black_color, top_left, btm_right);
        };

        imageops::overlay(&mut final_image, &photo, overlay_x, overlay_y);

        // add the film borders
        // let mut fb = image::load_from_memory_with_format(FILM_BORDER_BYTES, ImageFormat::Png)?
        //     .as_rgba8()
        //     .ok_or_else(|| {
        //         ImageError::IoError(IOError::new(
        //             ErrorKind::Other,
        //             "failed to read film border image data",
        //         ))
        //     })?
        //     .clone();
        let mut fb = borders::BORDER1.clone();

        if photo_is_portrait {
            fb = imageops::rotate90(&fb);
        };
        let mut fb_width = fit_width;
        let mut fb_height = (fb.height() as f32 * (fit_width as f32 / fb.width() as f32)) as u32;
        if !photo_is_portrait {
            fb_height = fit_height;
            fb_width = (fb.width() as f32 * (fit_height as f32 / fb.height() as f32)) as u32;
        };
        let start = Utc::now().time();
        let filter_type = imageops::FilterType::Triangle;
        fb = imageops::resize(&fb, fb_width, fb_height, filter_type);
        println!(
            "fitting border to {} x {} took {:?}",
            fb_width,
            fb_height,
            Utc::now().time() - start,
        );

        let fade_transition_direction = if photo_is_portrait {
            img::Direction::Vertical
        } else {
            img::Direction::Horizontal
        };
        let fade_width = (0.05 * fit_height as f32) as u32;
        let fb_useable_frac = 0.75;

        // top border
        let mut top_fb = fb.clone();
        let top_fb_crop = Size {
            width: if photo_is_portrait {
                fb.width()
            } else {
                min(
                    (fb_useable_frac * photo.width() as f32) as u32,
                    (fb_useable_frac * fb.width() as f32) as u32,
                )
            },
            height: if photo_is_portrait {
                min(
                    (fb_useable_frac * photo.height() as f32) as u32,
                    (fb_useable_frac * fb.height() as f32) as u32,
                )
            } else {
                fb.height()
            },
        };
        top_fb =
            imageops::crop(&mut top_fb, 0, 0, top_fb_crop.width, top_fb_crop.height).to_image();
        let fade_dim = if photo_is_portrait {
            top_fb_crop.height
        } else {
            top_fb_crop.width
        };
        img::fade_out(
            &mut top_fb,
            max(0, fade_dim - fade_width),
            fade_dim - 1,
            fade_transition_direction,
        );
        imageops::overlay(&mut final_image, &top_fb, overlay_x, overlay_y);

        // bottom border
        let mut btm_fb = fb.clone();
        let btm_fb_crop = Size {
            width: if photo_is_portrait {
                fb.width()
            } else {
                min(
                    (fb_useable_frac * photo.width() as f32) as u32,
                    (fb_useable_frac * fb.width() as f32) as u32,
                )
            },
            height: if photo_is_portrait {
                min(
                    (fb_useable_frac * photo.height() as f32) as u32,
                    (fb_useable_frac * fb.height() as f32) as u32,
                )
            } else {
                fb.height()
            },
        };
        let btm_fb_x = if photo_is_portrait {
            0
        } else {
            btm_fb.width() - btm_fb_crop.width
        };
        let btm_fb_y = if photo_is_portrait {
            btm_fb.height() - btm_fb_crop.height
        } else {
            0
        };

        btm_fb = imageops::crop(
            &mut btm_fb,
            btm_fb_x,
            btm_fb_y,
            btm_fb_crop.width,
            btm_fb_crop.height,
        )
        .to_image();
        img::fade_out(&mut btm_fb, fade_width, 0, fade_transition_direction);
        imageops::overlay(
            &mut final_image,
            &btm_fb,
            if photo_is_portrait {
                overlay_x
            } else {
                overlay_x + (photo.width() - btm_fb_crop.width) as i64
            },
            overlay_y + (fit_height - btm_fb_crop.height) as i64,
        );

        // intermediate borders
        let inter_fb_crop = Size {
            width: if photo_is_portrait {
                fb.width()
            } else {
                min(
                    (0.5 * photo.width() as f32) as u32,
                    (0.5 * fb.width() as f32) as u32,
                )
            },
            height: if photo_is_portrait {
                min(
                    (0.5 * photo.height() as f32) as u32,
                    (0.5 * fb.height() as f32) as u32,
                )
            } else {
                fb.height()
            },
        };

        let (start, end, step_size) = if photo_is_portrait {
            (
                top_fb_crop.height - fade_width,
                fit_height - btm_fb_crop.height + fade_width,
                inter_fb_crop.height as usize,
            )
        } else {
            (
                top_fb_crop.width - fade_width,
                fit_width - btm_fb_crop.width + fade_width,
                inter_fb_crop.width as usize,
            )
        };

        // println!("from {} to {} with step size {}", start, end, step_size);
        for i in (start..=end).step_by(step_size) {
            println!("{}", i);
            let mut inter_fb = fb.clone();
            let (inter_fb_x, inter_fb_y, inter_fb_width, inter_fb_height) = if photo_is_portrait {
                (
                    0,
                    (0.25 * fb.height() as f32) as u32 - fade_width,
                    inter_fb_crop.width,
                    min(inter_fb_crop.height, end - i) + 2 * fade_width,
                )
            } else {
                (
                    (0.25 * fb.width() as f32) as u32 - fade_width,
                    0,
                    min(inter_fb_crop.width, end - i) + 2 * fade_width,
                    inter_fb_crop.height,
                )
            };
            inter_fb = imageops::crop(
                &mut inter_fb,
                inter_fb_x,
                inter_fb_y,
                inter_fb_width,
                inter_fb_height,
            )
            .to_image();
            img::fade_out(&mut inter_fb, fade_width, 0, fade_transition_direction);
            let fade_dim = if photo_is_portrait {
                inter_fb_height
            } else {
                inter_fb_width
            };
            img::fade_out(
                &mut inter_fb,
                fade_dim - fade_width,
                fade_dim - 1,
                fade_transition_direction,
            );
            imageops::overlay(
                &mut final_image,
                &inter_fb,
                if photo_is_portrait {
                    overlay_x
                } else {
                    overlay_x - (fade_width + i) as i64
                },
                if photo_is_portrait {
                    overlay_y - (fade_width + i) as i64
                } else {
                    overlay_y
                },
            );
        }

        // show the center of the final image
        if options.preview {
            let highlight_color = Rgba([255, 0, 0, 50]);
            let mut ctr_tl = Point {
                x: 0,
                y: (size.height - size.width) / 2,
            };
            let mut ctr_br = Point {
                x: size.width,
                y: ((size.height - size.width) / 2) + size.width,
            };
            if !output_is_portrait {
                ctr_tl = Point {
                    x: (size.width - size.height) / 2,
                    y: 0,
                };
                ctr_br = Point {
                    x: ((size.width - size.height) / 2) + size.height,
                    y: size.height,
                };
            }
            img::fill_rect(&mut final_image, &highlight_color, ctr_tl, ctr_br);
        };

        Ok(final_image)
    }
}
