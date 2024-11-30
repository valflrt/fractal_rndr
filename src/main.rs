use std::time::Instant;

use image::{ImageBuffer, Rgb};
use num::complex::Complex;
use rayon::iter::{ParallelBridge, ParallelIterator};

const IMG_WIDTH: u32 = 1920 * 2;
const IMG_HEIGHT: u32 = 1080 * 2;
const ASPECT_RATIO: f64 = IMG_WIDTH as f64 / IMG_HEIGHT as f64;

const MAX_ITER: u32 = 2000;

// const ZOOM: f64 = 1.;
// const CENTER_X: f64 = 0.;
// const CENTER_Y: f64 = 0.;

const ZOOM: f64 = 0.01;
const CENTER_X: f64 = 0.;
const CENTER_Y: f64 = 0.045;

const WIDTH: f64 = ZOOM;
const HEIGHT: f64 = WIDTH / ASPECT_RATIO;
const X_MIN: f64 = CENTER_X - WIDTH / 2.0;
const X_MAX: f64 = CENTER_X + WIDTH / 2.0;
const Y_MIN: f64 = CENTER_Y - HEIGHT / 2.0;
const Y_MAX: f64 = CENTER_Y + HEIGHT / 2.0;

fn main() {
    let mut img = ImageBuffer::new(IMG_WIDTH, IMG_HEIGHT);

    let start = Instant::now();

    let pixels = img
        .enumerate_pixels()
        .par_bridge()
        .map(|(x, y, _)| {
            let real = X_MIN + (x as f64 / IMG_WIDTH as f64) * (X_MAX - X_MIN);
            let imag = Y_MIN + (y as f64 / IMG_HEIGHT as f64) * (Y_MAX - Y_MIN);
            let c = Complex::new(real, imag);

            let iterations = mandelbrot(c, MAX_ITER);

            (x, y, fancy_color_mapping(iterations, MAX_ITER))
        })
        .collect::<Vec<_>>();

    for (x, y, pixel) in pixels {
        img.put_pixel(x, y, pixel);
    }

    println!("{:?} elapsed", start.elapsed());

    img.save("fractal.png").expect("Failed to save image");
}

fn mandelbrot(c: Complex<f64>, max_iter: u32) -> u32 {
    let mut z0 = Complex::new(0.0, 0.0);
    let mut z1 = Complex::new(0.0, 0.0);

    for i in 0..max_iter {
        if z1.norm_sqr() > 4.0 {
            return i;
        }
        let new_z = z1 * z1 + z0 + c;
        z0 = z1;
        z1 = new_z;
    }
    max_iter
}

fn fancy_color_mapping(iterations: u32, max_iter: u32) -> Rgb<u8> {
    if iterations == max_iter {
        Rgb([0, 0, 0])
    } else {
        let t = iterations as f64 / max_iter as f64;

        let r = (9.0 * (1.0 - t) * t.powi(3) * 255.0).min(255.0).max(0.0) as u8;
        let g = (15.0 * (1.0 - t).powi(2) * t.powi(2) * 255.0)
            .min(255.0)
            .max(0.0) as u8;
        let b = (8.5 * (1.0 - t).powi(3) * t * 255.0).min(255.0).max(0.0) as u8;

        Rgb([r, g, b])
    }
}
