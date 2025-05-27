use super::super::super::math::vec3::*;
use super::PDF;

/// 均匀球面分布PDF，用于各向同性散射
#[derive(Debug, Default)]
pub struct SpherePDF;

impl SpherePDF {
    /// 创建均匀球面分布PDF
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl PDF for SpherePDF {
    #[inline]
    fn value(&self, _direction: &Vec3) -> f64 {
        // 单位球面的均匀分布PDF值
        1.0 / (4.0 * std::f64::consts::PI)
    }

    #[inline]
    fn generate(&self) -> Vec3 {
        Vec3::random_unit_vector()
    }
}
