extern crate image;

use std::collections::{VecDeque, HashSet};
use image::{GenericImage, ImageBuffer, Pixel, Rgba};

type ImageCoordinates = (u32, u32);

fn get_closest_opaque_pixel<'a>(img: &'a ImageBuffer<Rgba<u8>, std::vec::Vec<u8>>, x: u32, y: u32) -> Option<&'a Rgba<u8>> {
    let mut queue: VecDeque<ImageCoordinates> = VecDeque::new();
    let mut seen: HashSet<ImageCoordinates> = HashSet::new();
    queue.push_front((x, y));

    while queue.len() > 0 {
        let current = queue.pop_front().unwrap();

        if !seen.contains(&current) {
            seen.insert(current);

            let current_x = current.0;
            let current_y = current.1;

            if img.in_bounds(current_x, current_y) {
                let pixel_at = img.get_pixel(current.0, current.1);
            
                if pixel_at.channels4().3 != 0 {
                    return Some(&pixel_at);
                }
                else {
                    // stop overflow
                    if current_x > 0 {
                        queue.push_back((current_x - 1, current_y));
                    }

                    if current_y > 0 {
                        queue.push_back((current_x, current_y - 1));
                    }

                    queue.push_back((current_x + 1, current_y));
                    queue.push_back((current_x, current_y + 1));
                }
            }
        }
    }

    None
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let file = &args[1];

    match image::open(file) {
        Ok(mut img) => {
            println!("dimensions: {:?}", img.dimensions());

            let reference = img.clone();
            let reference_rgba = reference.as_rgba8().unwrap();
            let editable_rgba = img.as_mut_rgba8().unwrap();

            // Block to encapsulate the mutable borrow so that the image can be saved
            {
                for (x, y, pixel) in editable_rgba.enumerate_pixels_mut() {
                    let alpha = pixel.channels4().3;

                    if alpha == 0 {
                        let nearest_opaque = get_closest_opaque_pixel(reference_rgba, x, y).unwrap();
                        let channels = nearest_opaque.channels4();

                        *pixel = Rgba([channels.0, channels.1, channels.2, alpha]);
                    }
                }
            }
            
            editable_rgba.save(&args[2]).unwrap();
        },
        Err(msg) => panic!("{:?}", msg),
    }
}
