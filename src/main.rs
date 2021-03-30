use sdl2::render::Canvas;
use sdl2::{pixels, rect};
use sdl2::video::Window;

use packed_simd_2::{f64x4, m64x4};

#[derive(Clone, Copy)]
struct ComplexSIMD{
    real: f64x4,
    imag: f64x4,
}

impl ComplexSIMD{
    pub fn new(real: f64x4, imag: f64x4) -> Self{
        Self{
            real,
            imag,
        }
    }
}

/*impl std::ops::Add for Vec4d{
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
}*/

//#[cfg(target_feature = "avx2")]
fn mandelbrot(z: ComplexSIMD, c: ComplexSIMD) -> ComplexSIMD{
    let nr = z.real * z.real - z.imag * z.imag + c.real;
    let ni = z.real * z.imag * 2.0 + c.imag;
    ComplexSIMD::new(nr, ni)
}

//#[cfg(target_feature = "avx2")]
fn escape_check(z: ComplexSIMD) -> m64x4{
    (z.real * z.real + z.imag * z.imag).gt(f64x4::splat(4.0))
}

/// Converts the x y coordinates of the pixels to the mandelbrot coordinates
fn pixel_to_mandelbrot(pixel: f64x4, offset: f64, zoom: f64, zoom_correction: i32) -> f64x4{
    (pixel - zoom_correction as f64) * zoom + offset
}

/// renders the mandelbrot set to a SDL2 canvas
fn render_mandelbrot(canvas: &mut Canvas<Window>, x_offset: f64, y_offset: f64, zoom: f64, width: i32, height: i32){
    canvas.set_draw_color(pixels::Color::RGB(30, 30, 30));
    canvas.clear();
    const MAX_ITER: i32 = 255 * 3;
    let width_correction = width / 2; // makes the zoom center on the middle of the camera
    let height_correction = height / 2;
    for i in 0..height{
        for j in (0..width).step_by(4){
            let c= ComplexSIMD::new(pixel_to_mandelbrot(f64x4::new(j as f64, j as f64 + 1.0, j as f64 + 2.0,
                                                                   j as f64 + 3.0), x_offset, zoom, width_correction),
                pixel_to_mandelbrot(f64x4::splat(i as f64), y_offset, zoom, height_correction));
            let mut num_of_iters = f64x4::splat(0.0);
            let mut z = ComplexSIMD::new(f64x4::splat(0.0), f64x4::splat(0.0));
            
            'esc: for _ in 0..=MAX_ITER {
                z = mandelbrot(z, c);
                let escape_mask = escape_check(z);
                if escape_mask.all() {
                    break 'esc;
                }
                let a = f64x4::new(if escape_mask.extract(0) {0.0} else {1.0}, // todo: scuffed, change
                                   if escape_mask.extract(1) {0.0} else {1.0},
                                   if escape_mask.extract(2) {0.0} else {1.0},
                                   if escape_mask.extract(3) {0.0} else {1.0});
                num_of_iters += a;
            }
            let colour = (num_of_iters % 255.0);
            for k in 0..4 {
                canvas.set_draw_color(pixels::Color::RGB(colour.extract(k) as u8, colour.extract(k) as u8, 
                                                         colour.extract(k) as u8));
                canvas.fill_rect(rect::Rect::new(j + k as i32, i, 1, 1))
                    .expect("Failed to create rectangle");
            }
        }
    }
}

fn main() {
    const WIDTH: i32 = 1920;
    const HEIGHT: i32 = 1080;
    
    let sdl_context = sdl2::init().expect("Failed to create sdl context");
    let video_subsystem = sdl_context.video().expect("Failed to create video subsystem");
    let window = video_subsystem.window("Mandelbrot set", WIDTH as u32, HEIGHT as u32)
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
 