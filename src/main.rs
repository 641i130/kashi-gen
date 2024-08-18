use image::{RgbImage, Rgb};
use rusttype::{Font, Scale};
use srtparse::Item;
use std::collections::HashMap;
use std::path::Path;

// Pseudocode plan
// Get all subtitles from SRT file
// Using the frame number and the text with the given frame, get all the text and put it into a
// special data type that allows for multiple lines of text 
//
// This should allow for multi lines of text when it matters

// Constants
const WIDTH: u32 = 1280;
const HEIGHT: u32 = 720;
const SCALE: f32 = 75.0;
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

fn text_frame(text:&str) -> RgbImage {
    let mut img = RgbImage::new(WIDTH, HEIGHT);
    for pixel in img.pixels_mut() {
        *pixel = Rgb([0, 0, 0]);
    }

    let v_metrics = load_font().v_metrics(Scale::uniform(SCALE));
    let glyphs: Vec<_> = load_font().layout(&text, Scale::uniform(SCALE), rusttype::point(0.0, v_metrics.ascent)).collect();
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
    return img;
}

fn multi_text_frame(text1: &str, text2: &str) -> RgbImage {
    let mut img = RgbImage::new(WIDTH, HEIGHT);
    for pixel in img.pixels_mut() {
        *pixel = Rgb([0, 0, 0]);
    }

    let scale = Scale::uniform(SCALE);
    let v_metrics = load_font().v_metrics(scale);

    // Draw text1
    let glyphs1: Vec<_> = load_font().layout(&text1, scale, rusttype::point(0.0, v_metrics.ascent)).collect();
    let max_width1 = glyphs1.iter().map(|g| g.position().x as u32).max().unwrap_or(0);
    let x1 = (WIDTH - max_width1) / 2;
    let y1 = (HEIGHT / 2) - ((v_metrics.ascent as u32 + v_metrics.descent as u32) / 2);

    for glyph in glyphs1 {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x_offset, y_offset, v| {
                let x = x1 + x_offset as u32 + bounding_box.min.x as u32;
                let y = y1 + y_offset as u32 + bounding_box.min.y as u32;
                if x < WIDTH && y < HEIGHT {
                    let pixel = img.get_pixel_mut(x, y);
                    let intensity = (v * 255.0) as u8;
                    *pixel = Rgb([intensity, intensity, intensity]);
                }
            });
        }
    }

    // Draw text2
    let v_metrics2 = load_font().v_metrics(scale);
    let glyphs2: Vec<_> = load_font().layout(&text2, scale, rusttype::point(0.0, v_metrics2.ascent)).collect();
    let max_width2 = glyphs2.iter().map(|g| g.position().x as u32).max().unwrap_or(0);
    let x2 = (WIDTH - max_width2) / 2;
    let y2 = (HEIGHT / 2) + ((v_metrics2.ascent as u32 + v_metrics2.descent as u32) / 2);

    for glyph in glyphs2 {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x_offset, y_offset, v| {
                let x = x2 + x_offset as u32 + bounding_box.min.x as u32;
                let y = y2 + y_offset as u32 + bounding_box.min.y as u32;
                if x < WIDTH && y < HEIGHT {
                    let pixel = img.get_pixel_mut(x, y);
                    let intensity = (v * 255.0) as u8;
                    *pixel = Rgb([intensity, intensity, intensity]);
                }
            });
        }
    }

    img
}

fn generate_frames(frames:HashMap<u64, Frame>) {
    for i in 0..frames.len() {
        // If the frame has text generate text
        match frames.get(&(i as u64)) {
            // Has text
            Some(frame) => {
                match &frame.text {
                    Some(text) => {
                        // iterate over all text entries and create the image needed
                        if text.len() > 1 {
                            let img = multi_text_frame(&text[0],&text[1]);
                            let path = format!("output/frame_{:06}.png", i);
                            img.save(&path).expect("Unable to save image");
                        } else {
                            let img = text_frame(&text[0]);
                            let path = format!("output/frame_{:06}.png", i);
                            img.save(&path).expect("Unable to save image");
                        }
                    },
                    // No text, black frame
                    None => {
                        let mut img = RgbImage::new(WIDTH, HEIGHT);
                        for pixel in img.pixels_mut() {
                            *pixel = Rgb([0, 0, 0]);
                        }
                        let path = format!("output/frame_{:06}.png", i);
                        let p = Path::new(&path);
                        if !p.exists() {
                            let mut img = RgbImage::new(WIDTH, HEIGHT);
                            for pixel in img.pixels_mut() {
                                *pixel = Rgb([0, 0, 0]);
                            }
                            img.save(&path).expect("Unable to save image");
                        }
                    },
                }
            },
            None => todo!(),
        }
    }
}

struct Frame {
    i: u64,
    text: Option<Vec<String>>,
}

fn main() {
    let srt = parse_srt();
    let frame_count: u64 = (60 * 4 + 41) * FRAME_RATE as u64;
    let mut frames: HashMap<u64, Frame> = HashMap::new();
    for i in 0..frame_count {
        frames.insert(i,Frame{i,text:None});
    }
    for item in srt {
        // calculate frame range of given text and the frame amount
        let start_frame = ((item.start_time.into_duration().as_millis() as f64 / 1000.0) * FRAME_RATE).round() as u64;
        let end_frame = ((item.end_time.into_duration().as_millis() as f64 / 1000.0) * FRAME_RATE).round() as u64;
        // how to check if given frame is in a frame range of a given thing
        // get a mutable reference to the Frame
        for ind in start_frame..end_frame {
            if let Some(frame) = frames.get_mut(&ind) {
                match &mut frame.text {
                    Some(frr) => {
                        // Append to the existing vector
                        frr.push(item.text.clone());
                    },
                    None => {
                        // No text yet, create it!
                        println!("Adding {} for frame {}",item.text,frame.i);
                        frame.text = Some(vec![item.text.clone()]);
                    }
                }
            } else {
                todo!();  // handle the case where the frame doesn't exist
            }
        }
    }
    generate_frames(frames);
}
