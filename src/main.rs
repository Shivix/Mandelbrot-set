#![feature(portable_simd)]

use std::simd::*;
use image::{Rgb, RgbImage};

#[derive(Clone, Copy)]
struct ComplexSIMD {
    real: f64x4,
    imag: f64x4,
}

impl ComplexSIMD {
    pub fn new(real: f64x4, imag: f64x4) -> Self {
        Self { real, imag }
    }
}

//#[cfg(target_feature = "avx2")]
fn mandelbrot(z: ComplexSIMD, c: ComplexSIMD) -> ComplexSIMD {
    let nr = z.real * z.real - z.imag * z.imag + c.real;
    let ni = z.real * z.imag * f64x4::splat(2.0) + c.imag;
    ComplexSIMD::new(nr, ni)
}

//#[cfg(target_feature = "avx2")]
fn escape_check(z: ComplexSIMD) -> mask64x4 {
    (z.real * z.real + z.imag * z.imag).simd_gt(f64x4::splat(4.0))
}

/// Converts the x y coordinates of the pixels to the mandelbrot coordinates
fn pixel_to_mandelbrot(pixel: f64x4, offset: f64, zoom: f64, zoom_correction: u32) -> f64x4 {
    (pixel - f64x4::splat(zoom_correction as f64)) * f64x4::splat(zoom + offset)
}

/// renders the mandelbrot set to a SDL2 canvas
fn render_mandelbrot(image: &mut RgbImage, x_offset: f64, y_offset: f64, zoom: f64) {
    const MAX_ITER: i32 = 255 * 3;
    // divide by 2 makes the zoom center on the middle of the camera
    let width_correction = image.width() / 2;
    let height_correction = image.height() / 2;
    for i in 0..image.height() {
        for j in (0..image.width()).step_by(4) {
            let c = ComplexSIMD::new(
                pixel_to_mandelbrot(
                    f64x4::from_array([j as f64, (j + 1) as f64, (j + 2) as f64, (j + 3) as f64]),
                    x_offset,
                    zoom,
                    width_correction,
                ),
                pixel_to_mandelbrot(f64x4::splat(i as f64), y_offset, zoom, height_correction),
            );
            let mut num_of_iters = i64x4::splat(0);
            let mut z = ComplexSIMD::new(f64x4::splat(0.0), f64x4::splat(0.0));
            let mut escaped = Mask::splat(false);
            for _ in 0..=MAX_ITER {
                z = mandelbrot(z, c);
                escaped |= escape_check(z);
                if escaped.all() {
                    break;
                }
                // true is set to -1
                num_of_iters -= !escaped.to_int();
            }
            for k in 0..4 {
                let colour = (num_of_iters[k] % 255) as u8;
                image.put_pixel(j + k as u32, i, Rgb([colour, colour, colour]));
            }
        }
    }
}

// TODO: add command line args for offset and zoom
fn main() {
    const WIDTH: u32 = 2560;
    const HEIGHT: u32 = 1440;
    let mut image = RgbImage::new(WIDTH, HEIGHT);
    let zoom = 0.003;
    let x_offset = 0.0;
    let y_offset = 0.0;
    // FIXME: offsets are shrinking image
    //let zoom = 0.000002;
    //let x_offset = -0.355;
    //let y_offset = 0.096;
    render_mandelbrot(&mut image, x_offset, y_offset, zoom);
    image.save("mandelbrot.jpg").expect("cannot save image");
}
