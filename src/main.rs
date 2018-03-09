extern crate image;

use image::{GenericImage, Pixel, Rgba};

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
                        let mut nearest_opaque = &image::Rgba([0, 0, 0, 0]);
                        let mut nearest_dist: f64 = std::f64::INFINITY;

                        for (other_x, other_y, other_pixel) in reference_rgba.enumerate_pixels() {
                            let other_alpha = other_pixel.channels4().3;

                            if other_alpha > 0 {
                                let dist = ((x as f64 - other_x as f64).powi(2) + (y as f64 - other_y as f64).powi(2)).sqrt();

                                if dist < nearest_dist {
                                    nearest_opaque = other_pixel;
                                    nearest_dist = dist;
                                }
                            }
                        }

                        *pixel = image::Rgba([nearest_opaque.channels4().0, nearest_opaque.channels4().1, nearest_opaque.channels4().2, alpha]);
                    }
                }
            }
            
            editable_rgba.save(&args[2]).unwrap();
        },
        Err(msg) => panic!("{:?}", msg),
    }
}
