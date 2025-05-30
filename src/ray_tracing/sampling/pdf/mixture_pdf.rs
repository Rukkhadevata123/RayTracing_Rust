use super::super::super::math::vec3::Vec3;
use super::super::super::utils::util::random_double;
use super::PDF;
use std::sync::Arc;

/// 混合PDF，组合两种不同的采样策略
pub struct MixturePDF {
    pdf1: Arc<dyn PDF>,
    pdf2: Arc<dyn PDF>,
    weight1: f64,
    weight2: f64,
}

impl MixturePDF {
    /// 创建等权重的混合PDF
    #[inline]
    pub fn new(pdf1: Arc<dyn PDF>, pdf2: Arc<dyn PDF>) -> Self {
        Self::new_weighted(pdf1, pdf2, 0.5)
    }

    /// 创建带权重的混合PDF
    #[inline]
    pub fn new_weighted(pdf1: Arc<dyn PDF>, pdf2: Arc<dyn PDF>, weight1: f64) -> Self {
        let weight1 = weight1.clamp(0.0, 1.0);
        Self {
            pdf1,
            pdf2,
            weight1,
            weight2: 1.0 - weight1,
        }
    }
}

impl PDF for MixturePDF {
    #[inline]
    fn value(&self, direction: &Vec3) -> f64 {
        let value =
            self.weight1 * self.pdf1.value(direction) + self.weight2 * self.pdf2.value(direction);

        // 确保返回值始终为正，避免数值问题
        if value < 1e-8 { 1e-8 } else { value }
    }

    #[inline]
    fn generate(&self) -> Vec3 {
        // 根据权重随机选择PDF
        if random_double() < self.weight1 {
            self.pdf1.generate()
        } else {
            self.pdf2.generate()
        }
    }
}

impl std::fmt::Debug for MixturePDF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MixturePDF")
            .field("pdf1", &"<PDF>")
            .field("pdf2", &"<PDF>")
            .field("weight1", &self.weight1)
            .field("weight2", &self.weight2)
            .finish()
    }
}
