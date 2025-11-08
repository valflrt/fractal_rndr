use crate::F;

// Texture simplified from https://github.com/Calinou/free-blue-noise-textures
const BLUE_NOISE: &[u8] = include_bytes!("../assets/triang_pdf_bluenoise.bin");
const BLUE_NOISE_SIZE: usize = 256;

/// Returns triangular pdf blue noise as a float between 0 and 1.
pub fn blue_noise(x: usize, y: usize) -> F {
    let i = x % 256 + BLUE_NOISE_SIZE * (y % 256);
    BLUE_NOISE[i] as F / 255. // in range (0, 1)
}
