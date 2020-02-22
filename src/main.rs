use image::{ImageFormat, Rgba};
use imageproc::drawing::draw_text_mut;
use rusttype::{FontCollection, Scale};
use std::env;
use std::io::Write;
mod fastblur;

fn main() {
    let (input, output) = if env::args().count() == 3 {
        (env::args().nth(1).unwrap(), env::args().nth(2).unwrap())
    } else {
        panic!("Please enter an input file and an output file");
    };

    let icon_font = Vec::from(include_bytes!("Material-Design-Iconic-Font.ttf") as &[u8]);
    let text_font = Vec::from(include_bytes!("Roboto-Regular.ttf") as &[u8]);
    let icon_font = FontCollection::from_bytes(icon_font)
        .unwrap()
        .into_font()
        .unwrap();
    let text_font = FontCollection::from_bytes(text_font)
        .unwrap()
        .into_font()
        .unwrap();

    let png = image::open(input);
    if let Ok(image::DynamicImage::ImageRgb8(png_data)) = png {
        let width = png_data.width() as usize;
        let height = png_data.height() as usize;
        let data = png_data.into_raw();
        let mut data_new = Vec::<[u8; 3]>::with_capacity(width * height);
        data_new.resize(width * height, [0, 0, 0]);
        for y in 0..height {
            for x in 0..width {
                let offset = ((width * y) + x) as usize;
                data_new[((width * y) + x) as usize] = [
                    data[offset * 3],
                    data[(offset * 3) + 1],
                    data[(offset * 3) + 2],
                ];
            }
        }
        fastblur::gaussian_blur(&mut data_new, width as usize, height as usize, 8.0);
        let mut buf = Vec::new();
        let header = format!("P6\n{}\n{}\n{}\n", width, height, 255);
        buf.write(header.as_bytes()).unwrap();
        for px in data_new {
            buf.write(&px).unwrap();
        }

        let wu32 = width as u32;
        let hu32 = height as u32;

        let font_scaling = 10.0;
        let mut blurred = image::load_from_memory(&buf).unwrap().to_rgba();
        let mut gray = image::imageops::grayscale(&blurred);
        let center = image::imageops::crop(
            &mut gray,
            (wu32 / 2) - (wu32 / font_scaling as u32),
            (hu32 / 2) - (hu32 / font_scaling as u32),
            (wu32 / font_scaling as u32) * 2,
            (hu32 / font_scaling as u32) * 2,
        );
        // get the intensity at the 50th percentile and pick the symbol
        // color based on that
        let textclr = if imageproc::stats::percentile(&center.to_image(), 50) > 130 {
            0u8
        } else {
            255u8
        };
        let h = height as f32;
        let icon_scale = Scale {
            x: h / font_scaling,
            y: h / font_scaling,
        };

        let text_scale = Scale {
            x: h / (font_scaling * 3.0),
            y: h / (font_scaling * 3.0),
        };

        draw_text_mut(
            &mut blurred,
            Rgba([textclr, textclr, textclr, 255u8]),
            (width as u32 / 2) - (icon_scale.x as u32 / 3),
            (height as u32 / 2) - (icon_scale.y as u32 / 2),
            icon_scale,
            &icon_font,
            "",
        );

        draw_text_mut(
            &mut blurred,
            Rgba([textclr, textclr, textclr, 255u8]),
            ((width as f32 / 2.33) as u32) - (text_scale.x as u32),
            ((height as f32 / 1.64) as u32) - (text_scale.y as u32 / 2),
            text_scale,
            &text_font,
            "Type password to unlock",
        );

        blurred.save_with_format(output, ImageFormat::PNG).unwrap();
        if textclr == 0u8 {
            println!("bright")
        } else {
            println!("dark")
        }
    };
}
