use super::super::super::geometry::hittable::Hittable;
use super::super::super::math::vec3::*;
use super::PDF;
use std::sync::Arc;

/// 基于几何体的PDF，用于光源采样
pub struct HittablePDF {
    objects: Arc<dyn Hittable>,
    origin: Point3,
}

impl HittablePDF {
    /// 创建基于几何体的PDF
    #[inline]
    pub fn new(objects: Arc<dyn Hittable>, origin: &Point3) -> Self {
        Self {
            objects,
            origin: *origin,
        }
    }
}

impl PDF for HittablePDF {
    #[inline]
    fn value(&self, direction: &Vec3) -> f64 {
        self.objects.pdf_value(&self.origin, direction)
    }

    #[inline]
    fn generate(&self) -> Vec3 {
        self.objects.random(&self.origin)
    }
}

impl std::fmt::Debug for HittablePDF {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HittablePDF")
            .field("objects", &"<Hittable>")
            .field("origin", &self.origin)
            .finish()
    }
}
