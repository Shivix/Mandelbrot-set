use num_complex::Complex64;
use sdl2::render::Canvas;
use sdl2::{pixels, rect};
use sdl2::video::Window;

use std::arch::x86_64::*;

#[derive(Clone)]
struct ComplexSIMD{
    real: Vec4d,
    imag: Vec4d,
}

impl ComplexSIMD{
    pub fn new(real: Vec4d, imag: Vec4d) -> Self{
        Self{
            real,
            imag,
        }
    }
}

#[derive(Clone)]
struct Vec4d{
    val: __m256d,
}

impl Vec4d{
    pub unsafe fn new(val: f64) -> Self{
        Self{
            val: _mm256_set1_pd(val),
        }
    }
    pub unsafe fn from(val1: f64, val2: f64, val3: f64, val4: f64) -> Self{
        Self{
            val: _mm256_setr_pd(val1, val2, val3, val4),
        }
    }
    pub unsafe fn take(val: __m256d) -> Self{
        Self{
            val,
        }
    }
}

impl std::ops::Add for Vec4d{
    type Output = Self;
    fn add(self, other: Self) -> Self::Output{
        Self{
            val: unsafe { _mm256_add_pd(self.val, other.val) },
        }
    }
}
impl std::ops::Mul for Vec4d{
    type Output = Self;
    fn mul(self, other: Self) -> Self::Output{
        Self{
            val: unsafe { _mm256_mul_pd(self.val, other.val) },
        }
    }
}
impl std::ops::Sub for Vec4d{
    type Output = Self;
    fn sub(self, other: Self) -> Self::Output{
        Self{
            val: unsafe { _mm256_sub_pd(self.val, other.val) },
        }
    }
}

#[cfg(target_feature = "avx2")]
unsafe fn mandelbrot(z: ComplexSIMD, c: ComplexSIMD) -> ComplexSIMD{
    let nr = z.real.clone() * z.real.clone() - z.imag.clone() * z.imag.clone() + c.real.clone();
    let ni = z.real.clone() * z.imag.clone() * Vec4d::new(2.0) + c.imag.clone();
    ComplexSIMD::new(nr, ni)
}

#[cfg(target_feature = "avx2")]
unsafe fn escape_check(z: ComplexSIMD) -> Vec4d{
    Vec4d::take(_mm256_cmp_pd((z.real.clone() * z.real.clone() + z.imag.clone() * z.imag.clone()).val,
                  _mm256_set1_pd(4.0), _CMP_GT_OQ))
}

#[cfg(target_feature = "avx2")]
unsafe fn compute_point(c: ComplexSIMD) -> Vec4d{// TODO: simd check if changed, simd check if higher?
    const MAX_ITER: i32 = 255 * 3;
    let mut z = ComplexSIMD::new(Vec4d::new(0.0), Vec4d::new(0.0));
    let mut result: [i32; 4] = [MAX_ITER, MAX_ITER, MAX_ITER, MAX_ITER];
    let mut r = [false, false, false, false];
    for i in 1..=MAX_ITER {
        z = mandelbrot(z, c.clone());
        let iters = escape_check(z.clone());
        simd_extract(iters.clone(), 0);
        if iters[0] && !r[0] {
            result[0] = i;
            r[0] = true;
        }
        if iters[1] && !r[1] {
            result[1] = i;
            r[1] = true;
        }
        if iters[2] && !r[2] {
            result[2] = i;
            r[2] = true;
        }
        if iters[3] && !r[3] {
            result[3] = i;
            r[3] = true;
        }
        if r[0] && r[1] && r[2] && r[3] {
            Vec4d::from(result[0] as f64, result[1] as f64,
                               result[2] as f64, result[3] as f64)
        }
    }
    Vec4d::from(result[0] as f64, result[1] as f64, result[2] as f64, result[3] as f64)
}


/// calculates the quadratic map for the Mandelbrot set
fn mandelbrot(z: Complex64, c: Complex64) -> Complex64{
    z * z + c
}

fn escape_check(z: Complex64) -> bool{
    (z.re * z.re + z.im * z.im) > 4.0
}

/// calculates the number of iterations until it takes to escape
fn compute_point(c: Complex64) -> i32{
    const MAX_ITER: i32 = 255 * 3;
    let mut z = Complex64::new(0.0, 0.0);
    for i in 1..MAX_ITER{
        z = mandelbrot(z, c);
        if escape_check(z) {
            return i
        }
    }
    MAX_ITER
}

/// Converts the x y coordinates of the pixels to the mandelbrot coordinates
fn pixel_to_mandelbrot(pixel: i32, offset: f64, zoom: f64, zoom_correction: i32) -> f64{
    (pixel - zoom_correction) as f64 * zoom + offset
}

/// renders the mandelbrot set to a SDL2 canvas
fn render_mandelbrot(canvas: &mut Canvas<Window>, x_offset: f64, y_offset: f64, zoom: f64, width: i32, height: i32){
    canvas.set_draw_color(pixels::Color::RGB(30, 30, 30));
    canvas.clear();
    let width_correction = width / 2; // makes the zoom center on the middle of the camera
    let height_correction = height / 2;
    for i in 0..width{
        for j in 0..height{
            let iterations = compute_point(Complex64::new(
                pixel_to_mandelbrot(i, x_offset, zoom, width_correction),
                pixel_to_mandelbrot(j, y_offset, zoom, height_correction)));
            let colour = (iterations % 255) as u8;
            canvas.set_draw_color(pixels::Color::RGB(colour, colour, colour));
            canvas.fill_rect(rect::Rect::new(i, j, 1, 1)).expect("Failed to create rectangle");
        }
    }
}

fn main() {
    const WIDTH: i32 = 1920;
    const HEIGHT: i32 = 1080;
    
    let sdl_context = sdl2::init().expect("Failed to create sdl context");
    let video_subsystem = sdl_context.video().expect("Failed to create video subsystem");
    let window = video_subsystem.window("Mandelbrot set", WIDTH as u32, HEIGHT as u32)
        .fullscreen()
        .vulkan()
        .build()
        .expect("Failed to create window");
    let mut canvas: Canvas<Window> = window.into_canvas()
        .build()
        .expect("Failed to create canvas");
    let mut events = sdl_context.event_pump().expect("Cannot create event pump");
    
    let mut zoom = 0.003;
    let mut y_offset = 0.0;
    let mut x_offset = 0.0;
    'main: loop{
        render_mandelbrot(&mut canvas, x_offset, y_offset, zoom, WIDTH, HEIGHT);
        canvas.present();
        for event in events.poll_iter(){
            use sdl2::event::*;
            use sdl2::keyboard::Keycode;
            match event {
                Event::KeyDown{keycode: Some(Keycode::W), .. } => y_offset -= zoom * 20.0,
                Event::KeyDown{keycode: Some(Keycode::A), .. } => x_offset -= zoom * 20.0,
                Event::KeyDown{keycode: Some(Keycode::S), .. } => y_offset += zoom * 20.0,
                Event::KeyDown{keycode: Some(Keycode::D), .. } => x_offset += zoom * 20.0,
                Event::KeyDown{keycode: Some(Keycode::Q), .. } => zoom *= 0.5,
                Event::KeyDown{keycode: Some(Keycode::E), .. } => zoom *= 0.5,
                Event::KeyDown{keycode: Some(Keycode::Space), .. } => println!("Zoom: {}, x_offset: {}, y_offset: {}", 
                                                                               zoom, x_offset, y_offset),
                Event::KeyDown{..} => break 'main,
                _ => continue,
            }
        }
    }
}
 