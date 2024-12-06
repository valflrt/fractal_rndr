use astro_float::BigFloat;

use crate::{P, RM};

pub fn norm_sqr(z: &(BigFloat, BigFloat)) -> BigFloat {
    let (re, im) = z;

    re.powi(2, P, RM).add(&im.powi(2, P, RM), P, RM)
}

pub fn add(z1: &(BigFloat, BigFloat), z2: &(BigFloat, BigFloat)) -> (BigFloat, BigFloat) {
    let (re1, im1) = z1;
    let (re2, im2) = z2;

    (re1.add(&re2, P, RM), im1.add(&im2, P, RM))
}

pub fn mul(z1: &(BigFloat, BigFloat), z2: &(BigFloat, BigFloat)) -> (BigFloat, BigFloat) {
    let (re1, im1) = z1;
    let (re2, im2) = z2;

    let p = re1.mul(&re2, P, RM);
    let q = im1.mul(&im2, P, RM);
    let r = re1.add(&im1, P, RM).mul(&re2.add(im2, P, RM), P, RM);

    let re = p.sub(&q, P, RM);
    let im = r.sub(&p, P, RM).sub(&q, P, RM);

    (re, im)
}

pub fn pow(z: &(BigFloat, BigFloat), n: usize) -> (BigFloat, BigFloat) {
    let mut z = z.clone();

    for _ in 0..n - 1 {
        z = mul(&z, &z);
    }

    z
}
