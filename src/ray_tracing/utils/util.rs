use rand::Rng;
use std::f64::consts::PI;

#[inline]
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

#[inline]
pub fn random_double() -> f64 {
    let mut rng = rand::rng();
    rng.random()
}

#[inline]
pub fn random_double_range(min: f64, max: f64) -> f64 {
    let mut rng = rand::rng();
    rng.random_range(min..max)
}

#[inline]
pub fn random_int_range(min: i32, max: i32) -> i32 {
    let mut rng = rand::rng();
    rng.random_range(min..=max)
}
