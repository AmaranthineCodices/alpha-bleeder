extern crate image;
extern crate clap;
extern crate glob;
extern crate threadpool;

use std::collections::{VecDeque, HashSet};
use image::{GenericImage, ImageBuffer, Pixel, Rgba};
use clap::{Arg, App};

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
                    // stop underflow errors
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
    let matches = App::new("alpha-bleeder")
        .version("0.3.0")
        .author("AmaranthineCodices <me@amaranthinecodices.me>")
        .about("Bleeds color into fully-transparent pixels to eliminate artifacts while resizing.")
        .arg(Arg::with_name("FILES")
            .help("The input files. This can be a glob pattern, e.g. *.png")
            .required(true)
            .index(1))
        .get_matches();

    let pool = threadpool::ThreadPool::new(4);
    let mut count = 0;

    // FILES is required, so unwrapping is safe.
    let files_glob = matches.value_of("FILES").unwrap();
    for file in glob::glob(&files_glob).expect("Invalid glob pattern").filter_map(Result::ok) {
        count += 1;
        pool.execute(move|| {
            match image::open(&file) {
                Ok(mut img) => {
                    println!("bleeding image {}", file.display());
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
                    
                    editable_rgba.save(&file).unwrap();
                    println!("bled {} successfully", file.display());
                },
                Err(msg) => panic!("{:?}", msg),
            }
        });
    }

    pool.join();
    println!("bled {} images", count);
}
