use super::PDF;
use crate::ray_tracing::math::onb::ONB;
use crate::ray_tracing::math::vec3::*;

/// 余弦分布PDF，用于漫反射材质
#[derive(Debug)]
pub struct CosinePDF {
    uvw: ONB,
}

impl CosinePDF {
    /// 从法线方向创建余弦分布PDF
    #[inline]
    pub fn new(normal: &Vec3) -> Self {
        Self {
            uvw: ONB::new(normal),
        }
    }
}

impl PDF for CosinePDF {
    #[inline]
    fn value(&self, direction: &Vec3) -> f64 {
        let cosine_theta = direction.normalize().dot(&self.uvw.w());
        f64::max(0.0, cosine_theta / std::f64::consts::PI)
    }

    #[inline]
    fn generate(&self) -> Vec3 {
        self.uvw.local_to_world(&Vec3::random_cosine_direction())
    }
}
