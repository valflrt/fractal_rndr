use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use wide::f64x4;

/// A simd complex type. It holds 4 complex numbers and performs
/// calculations on them at once.
#[derive(Debug, Clone, Copy)]
pub struct Complexs {
    pub re: f64x4,
    pub im: f64x4,
}

impl Complexs {
    #[inline(always)]
    pub fn norm_sqr(self) -> f64x4 {
        self.re * self.re + self.im * self.im
    }

    pub fn powu(self, n: usize) -> Complexs {
        (0..n).fold(self, |acc, _| acc * acc)
    }

    pub fn splat(re: f64, im: f64) -> Complexs {
        Complexs {
            re: f64x4::splat(re),
            im: f64x4::splat(im),
        }
    }

    pub fn zeros() -> Complexs {
        Complexs {
            re: f64x4::splat(0.),
            im: f64x4::splat(0.),
        }
    }
}

impl Add for Complexs {
    type Output = Complexs;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Complexs {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl AddAssign for Complexs {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Mul for Complexs {
    type Output = Complexs;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let k1 = rhs.re * (self.re + self.im);
        let k2 = self.re * (rhs.im - rhs.re);
        let k3 = self.im * (rhs.re + rhs.im);

        Complexs {
            re: k1 - k3,
            im: k1 + k2,
        }
    }
}

impl Neg for Complexs {
    type Output = Complexs;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Complexs {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl Sub for Complexs {
    type Output = Complexs;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Complexs {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl SubAssign for Complexs {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}
