use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

use crate::{F, FX};

/// A simd complex type. It holds 4 complex numbers and performs
/// calculations on them at once.
#[derive(Debug, Clone, Copy)]
pub struct Complexx {
    pub re: FX,
    pub im: FX,
}

impl Complexx {
    #[inline]
    pub fn splat(re: F, im: F) -> Complexx {
        Complexx {
            re: FX::splat(re),
            im: FX::splat(im),
        }
    }

    #[inline]
    pub fn zeros() -> Complexx {
        Complexx {
            re: FX::splat(0.),
            im: FX::splat(0.),
        }
    }

    // #[inline]
    // pub fn is_zero(&self) -> FX {
    //     self.re.cmp_eq(0.) * self.im.cmp_eq(0.)
    // }

    #[inline]
    pub fn to_polar(self) -> (FX, FX) {
        (self.norm(), self.arg())
    }

    #[inline]
    pub fn from_polar(r: FX, theta: FX) -> Complexx {
        Complexx {
            re: r * theta.cos(),
            im: r * theta.sin(),
        }
    }
    #[inline]
    pub fn from_polar_splat(r: F, theta: F) -> Complexx {
        Complexx {
            re: FX::splat(r * theta.cos()),
            im: FX::splat(r * theta.sin()),
        }
    }

    #[inline(always)]
    pub fn norm_sqr(&self) -> FX {
        self.re * self.re + self.im * self.im
    }

    #[inline]
    pub fn norm(&self) -> FX {
        self.norm_sqr().sqrt()
    }

    #[inline]
    pub fn arg(&self) -> FX {
        self.im.atan2(self.re)
    }

    #[inline]
    pub fn powu(&self, n: usize) -> Complexx {
        (0..n).fold(*self, |acc, _| acc * acc)
    }

    #[inline]
    pub fn powf(&self, exp: F) -> Complexx {
        let (r, theta) = self.to_polar();
        Complexx::from_polar(r.powf(exp), theta * exp)
    }

    // #[inline]
    // pub fn powf4(&self, exp: FX) -> Complex4 {
    //     let (r, theta) = self.to_polar();
    //     Complex4::from_polar(r.pow_FX(exp), theta * exp)
    // }
}

impl Add for Complexx {
    type Output = Complexx;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Complexx {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
impl AddAssign for Complexx {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.re += rhs.re;
        self.im += rhs.im;
    }
}
impl Mul for Complexx {
    type Output = Complexx;

    #[inline(always)]
    fn mul(self, rhs: Self) -> Self::Output {
        let k1 = rhs.re * (self.re + self.im);
        let k2 = self.re * (rhs.im - rhs.re);
        let k3 = self.im * (rhs.re + rhs.im);

        Complexx {
            re: k1 - k3,
            im: k1 + k2,
        }
    }
}
impl Neg for Complexx {
    type Output = Complexx;

    #[inline(always)]
    fn neg(self) -> Self::Output {
        Complexx {
            re: -self.re,
            im: -self.im,
        }
    }
}
impl Sub for Complexx {
    type Output = Complexx;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Complexx {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}
impl SubAssign for Complexx {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.re -= rhs.re;
        self.im -= rhs.im;
    }
}

impl Mul<F> for Complexx {
    type Output = Complexx;

    #[inline(always)]
    fn mul(self, rhs: F) -> Self::Output {
        Complexx {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}

impl Mul<FX> for Complexx {
    type Output = Complexx;

    #[inline(always)]
    fn mul(self, rhs: FX) -> Self::Output {
        Complexx {
            re: self.re * rhs,
            im: self.im * rhs,
        }
    }
}
