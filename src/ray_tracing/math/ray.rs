use super::vec3::{Point3, Vec3};

#[derive(Clone, Copy, Debug, Default)]
pub struct Ray {
    pub orig: Point3,
    pub dir: Vec3,
    pub time: f64,
}

impl Ray {
    #[inline]
    pub const fn new(orig: Point3, dir: Vec3, time: f64) -> Self {
        Self { orig, dir, time }
    }

    #[inline]
    pub fn at(&self, t: f64) -> Point3 {
        self.orig + self.dir * t
    }

    #[inline]
    pub fn origin(&self) -> Point3 {
        self.orig
    }

    #[inline]
    pub fn direction(&self) -> Vec3 {
        self.dir
    }

    #[inline]
    pub fn time(&self) -> f64 {
        self.time
    }
}
