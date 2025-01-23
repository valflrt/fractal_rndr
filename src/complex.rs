use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use wide::f64x4;

/// A simd complex type. It holds 4 complex numbers and performs
/// calculations on them at once.
#[derive(Debug, Clone, Copy)]
pub struct Complex4 {
    pub re: f64x4,
    pub im: f64x4,
}

impl Complex4 {
    #[inline]
    pub fn splat(re: f64, im: f64) -> Complex4 {
        Complex4 {
            re: f64x4::splat(re),
            im: f64x4::splat(im),
        }
    }

    #[inline]
    pub fn zeros() -> Complex4 {
        Complex4 {
            re: f64x4::splat(0.),
            im: f64x4::splat(0.),
        }
    }

    // #[inline]
    // pub fn is_zero(&self) -> f64x4 {
    //     self.re.cmp_eq(0.) * self.im.cmp_eq(0.)
    // }

    #[inline]
    pub fn to_polar(self) -> (f64x4, f64x4) {
        (self.norm(), self.arg())
    }

    #[inline]
    pub fn from_polar(r: f64x4, theta: f64x4) -> Complex4 {
        Complex4 {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }

    #[inline(always)]
    pub fn norm_sqr(&self) -> f64x4 {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn norm(&self) -> f64x4 {
        self.norm_sqr().sqrt()
    }

    #[inline]
    pub fn arg(&self) -> f64x4 {
        self.im.atan2(self.re)
    }

    #[inline]
    pub fn powu(&self, n: usize) -> Complex4 {
        (0..n).fold(*self, |acc, _| acc * acc)
    }

    #[inline]
    pub fn powf(&self, exp: f64) -> Complex4 {
        let (r, theta) = self.to_polar();
        Complex4::from_polar(r.powf(exp), theta * exp)
    }

    // #[inline]
    // pub fn powf4(&self, exp: f64x4) -> Complex4 {
    //     let (r, theta) = self.to_polar();
    //     Complex4::from_polar(r.pow_f64x4(exp), theta * exp)
    // }
}

impl Add for Complex4 {
    type Output = Complex4;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Complex4 {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
impl AddAssign for Complex4 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}
impl Mul for Complex4 {
    type Output = Complex4;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let k1 = rhs.re * (self.re + self.im);
        let k2 = self.re * (rhs.im - rhs.re);
        let k3 = self.im * (rhs.re + rhs.im);

        Complex4 {
            re: k1 - k3,
            im: k1 + k2,
        }
    }
}
impl Neg for Complex4 {
    type Output = Complex4;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Complex4 {
            re: -self.re,
            im: -self.im,
        }
    }
}
impl Sub for Complex4 {
    type Output = Complex4;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Complex4 {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}
impl SubAssign for Complex4 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Mul<f64> for Complex4 {
    type Output = Complex4;

    #[inline(always)]
    fn mul(self, rhs: f64) -> Self::Output {
        Complex4 {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl Mul<f64x4> for Complex4 {
    type Output = Complex4;

    #[inline(always)]
    fn mul(self, rhs: f64x4) -> Self::Output {
        Complex4 {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}
