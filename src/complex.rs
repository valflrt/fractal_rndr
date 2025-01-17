use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[repr(align(16))]
#[derive(Debug, Clone, Copy)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}

impl Complex {
    pub const ZERO: Self = Self { re: 0., im: 0. };
    // pub const ONE: Self = Self { re: 1., im: 0. };
    // pub const I: Self = Self { re: 0., im: 1. };

    #[inline(always)]
    pub const fn norm_sqr(self) -> f64 {
        self.re * self.re + self.im * self.im
    }

    pub fn powu(self, n: usize) -> Complex {
        (0..n).fold(self, |acc, _| acc * acc)
    }
}

impl Add for Complex {
    type Output = Complex;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}

impl AddAssign for Complex {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}

impl Mul for Complex {
    type Output = Complex;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let k1 = rhs.re * (self.re + self.im);
        let k2 = self.re * (rhs.im - rhs.re);
        let k3 = self.im * (rhs.re + rhs.im);

        Complex {
            re: k1 - k3,
            im: k1 + k2,
        }
    }
}

impl Neg for Complex {
    type Output = Complex;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Complex {
            re: -self.re,
            im: -self.im,
        }
    }
}

impl Sub for Complex {
    type Output = Complex;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Complex {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}

impl SubAssign for Complex {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}
