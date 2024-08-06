use image::{RgbImage, Rgb};
use rusttype::{Font, Scale};
use srtparse::Item;
use chrono::Duration;
use std::fs::File;
use std::io::{Read, Write};

fn main() {
    // Load the font
    let font_data = include_bytes!("../font.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error loading font");

    // Define image properties
    let image_width = 1280;
    let image_height = 720;
    let scale = Scale::uniform(100.0);

    // Parse the SRT file
    let srt = srtparse::from_file("lyrics.srt").unwrap();
    // Assume 30 frames per second
    let fps = 30.0;
    
    let mut frame_index = 0;

    for Item { text, start_time, end_time, .. } in srt {
        let start_seconds = start_time.seconds;
        let end_seconds = end_time.seconds;
        let duration_frames = ((end_seconds - start_seconds) as f64 * fps) as usize;

        for _ in 0..duration_frames {
            // Create a new image with black background
            let mut img = RgbImage::new(image_width, image_height);
            for pixel in img.pixels_mut() {
                *pixel = Rgb([0, 0, 0]);
            }

            // Draw the text
            let v_metrics = font.v_metrics(scale);
            let glyphs: Vec<_> = font.layout(&text, scale, rusttype::point(0.0, v_metrics.ascent)).collect();
            let width = glyphs.iter().rev().next().unwrap().position().x as u32;

            let x = (image_width - width) / 2;
            let y = (image_height + v_metrics.ascent as u32) / 2;

            for glyph in glyphs {
                if let Some(bounding_box) = glyph.pixel_bounding_box() {
                    glyph.draw(|x_offset, y_offset, v| {
                        let x = x + x_offset as u32 + bounding_box.min.x as u32;
                        let y = y + y_offset as u32 + bounding_box.min.y as u32;
                        let pixel = img.get_pixel_mut(x, y);
                        let intensity = (v * 255.0) as u8;
                        *pixel = Rgb([intensity, intensity, intensity]);
                    });
                }
            }

            // Save the image
            img.save(format!("frame_{:06}.png", frame_index)).expect("Unable to save image");
            frame_index += 1;
        }
    }
}

