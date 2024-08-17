use image::{RgbImage, Rgb};
use rusttype::{Font, Scale};
use srtparse::Item;
use std::collections::HashMap;
use std::path::Path;

// Constants
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SCALE: f32 = 50.0;
const FRAME_RATE: f64 = 30.0;

// Function to load the font
fn load_font() -> Font<'static> {
    let font_data = include_bytes!("../font.ttf");
    Font::try_from_bytes(font_data as &[u8]).expect("Error loading font")
}

// Function to parse the SRT file
fn parse_srt() -> Vec<Item> {
    srtparse::from_file("lyrics.srt").unwrap()
}

fn generate_frames(times: &mut HashMap<u64, Item>, frame_count: u64) {
    for current_frame in 0..frame_count {
        if times.contains_key(&current_frame) {
            println!("FOUND FRAME at {}", &current_frame);
            match times.get(&current_frame) {
                Some(value) => {
                    let start_time = value.start_time.into_duration().as_millis();
                    let end_time = value.end_time.into_duration().as_millis();
                    let frames_srt = (((end_time - start_time) as f64 / 1000.0) * FRAME_RATE).round() as u64;
                    println!("frames_srt count for word {}", &frames_srt);

                    let mut img = RgbImage::new(WIDTH, HEIGHT);
                    for pixel in img.pixels_mut() {
                        *pixel = Rgb([0, 0, 0]);
                    }

                    let v_metrics = load_font().v_metrics(Scale::uniform(SCALE));
                    let glyphs: Vec<_> = load_font().layout(&value.text, Scale::uniform(SCALE), rusttype::point(0.0, v_metrics.ascent)).collect();
                    let max_width = glyphs.iter().map(|g| g.position().x as u32).max().unwrap_or(0);
                    let x = (WIDTH - max_width) / 2;
                    let y = (HEIGHT + v_metrics.ascent as u32) / 2;

                    for glyph in glyphs {
                        if let Some(bounding_box) = glyph.pixel_bounding_box() {
                            glyph.draw(|x_offset, y_offset, v| {
                                let x = x + x_offset as u32 + bounding_box.min.x as u32;
                                let y = y + y_offset as u32 + bounding_box.min.y as u32;
                                if x < WIDTH && y < HEIGHT {
                                    let pixel = img.get_pixel_mut(x, y);
                                    let intensity = (v * 255.0) as u8;
                                    *pixel = Rgb([intensity, intensity, intensity]);
                                }
                            });
                        }
                    }
                    let mut c = 0; 
                    for _ in 0..frames_srt {
                        let path = format!("output/frame_{:06}.png", current_frame+c);
                        img.save(&path).expect("Unable to save image");
                        c+=1;
                    }
                },
                None => continue,
            }
        } else {
            let mut img = RgbImage::new(WIDTH, HEIGHT);
            for pixel in img.pixels_mut() {
                *pixel = Rgb([0, 0, 0]);
            }
            let path = format!("output/frame_{:06}.png", current_frame);
            let p = Path::new(&path);
            if !p.exists() {
                let mut img = RgbImage::new(WIDTH, HEIGHT);
                for pixel in img.pixels_mut() {
                    *pixel = Rgb([0, 0, 0]);
                }
                img.save(&path).expect("Unable to save image");
            }
        }
    }
}

fn main() {
    let srt = parse_srt();
    let frame_count: u64 = (60 * 4 + 41) * FRAME_RATE as u64;
    let mut times: HashMap<u64, Item> = HashMap::new();

    for item in srt {
        times.insert((((item.start_time.into_duration().as_millis() as f64 / 1000.0) * FRAME_RATE).round()) as u64, item);
    }
    generate_frames(&mut times, frame_count);
}
