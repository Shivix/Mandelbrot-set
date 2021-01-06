use num_complex::Complex64;
use sdl2::render::Canvas;
use sdl2::{pixels, rect};
use sdl2::video::Window;

/// calculates the quadratic map for the Mandelbrot set
fn mandelbrot(z: Complex64, c: Complex64) -> Complex64{
    z * z + c
}

/// checks if the orbit remains bounded
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
            let iterations = compute_point(Complex64::new(pixel_to_mandelbrot(i, x_offset, zoom, width_correction),
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
                Event::KeyDown{keycode: Some(Keycode::Space), .. } => println!("Zoom: {}, x_offset: {}, y_offset: {}", zoom, x_offset, y_offset),
                Event::KeyDown{..} => break 'main,
                _ => continue,
            }
        }
    }
}
 