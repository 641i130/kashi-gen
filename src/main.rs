use image::{RgbImage, Rgb};
use rusttype::{Font, Scale};
use srtparse::Item;
use std::collections::HashMap;
use std::path::Path;

fn main() {
    // Load the font
    let font_data = include_bytes!("../font.ttf");
    let font = Font::try_from_bytes(font_data as &[u8]).expect("Error loading font");
    // Define image properties
    let image_width = 1280;
    let image_height = 720;
    let scale = Scale::uniform(50.0);
    // Parse the SRT file
    let srt = srtparse::from_file("lyrics.srt").unwrap();
    // song is 4:41 = 60*4+41 = 281*30 = 8430 frames total
    let frame_count: u64 = 8430;
    dbg!(srt.len()); // 132 sections of these
    // get all srt start times in an object
    //let mut times: Vec<u64> = Vec::new(); // Create an empty Vec to store the times
    let mut times: HashMap<u64, Item> = HashMap::new();
    for item in srt {
        times.insert(((item.start_time.into_duration().as_millis() as f64 / 1000.0) * 30.0).round() as u64,item);
    }
    for current_frame in 0..frame_count {
        // check if frame is in the next SRT
        if times.contains_key(&current_frame) {
            // generate text if its in this list
            println!("FOUND FRAME at {}",&current_frame);
            // given the start time, i need to get the text of that section and when it ends
            // guess i gotta make a custom struct hash table to look these up or something
            match times.get(&current_frame) {
                Some(value) => {
                    // calculate how many frames to make of this text
                    let start_time = value.start_time.into_duration().as_millis();
                    let end_time = value.end_time.into_duration().as_millis();
                    let frames_srt = (((end_time-start_time) as f64 / 1000.0) * 30.0).round() as u64; // calculate frame count
                    println!("frames_srt count for word {}",&frames_srt);
                    let mut img = RgbImage::new(image_width, image_height);
                    for pixel in img.pixels_mut() {
                        *pixel = Rgb([0, 0, 0]);
                    }
                    println!("Found: {}", value);
                    // Draw the text
                    let v_metrics = font.v_metrics(scale);
                    let glyphs: Vec<_> = font.layout(&value.text, scale, rusttype::point(0.0, v_metrics.ascent)).collect();
                    let width = glyphs.iter().rev().next().unwrap().position().x as u32;
                    let mut x = 0;
                    let y = (image_height + v_metrics.ascent as u32) / 2;
                    if image_width > width {
                        x = (image_width - width) / 2;
                    }
                    for glyph in glyphs {
                        if let Some(bounding_box) = glyph.pixel_bounding_box() {
                            glyph.draw(|x_offset, y_offset, v| {
                                let x = x + x_offset as u32 + bounding_box.min.x as u32;
                                let y = y + y_offset as u32 + bounding_box.min.y as u32;
                                if x < image_width && y < image_height {
                                    let pixel = img.get_pixel_mut(x, y);
                                    let intensity = (v * 255.0) as u8;
                                    *pixel = Rgb([intensity, intensity, intensity]);
                                }
                            });
                        }
                    }
                    let mut c = 0;
                    for _ in 0..frames_srt {
                        let original_frame_path = format!("output/frame_{:06}.png", current_frame+c);
                        img.save(&original_frame_path).expect("Unable to save wtf!?");
                        c+=1;
                    }
                },
                None => panic!("WTF"),
            }
        } else {
            // no text; make black frames
            let mut img = RgbImage::new(image_width, image_height);
            for pixel in img.pixels_mut() {
                *pixel = Rgb([0, 0, 0]);
            }
            let original_frame_path = format!("output/frame_{:06}.png", current_frame);
            let p = Path::new(&original_frame_path);
            if !p.exists() {
                img.save(&original_frame_path).expect("Unable to save wtf!?");
            }
        }
    }
}

