use super::Texture;
use crate::ray_tracing::math::vec3::{Color, Point3};

/// 纯色纹理
#[derive(Debug, Clone)]
pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    /// 从颜色创建纯色纹理
    #[inline]
    pub const fn new(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Texture for SolidColor {
    #[inline]
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.albedo
    }
}
