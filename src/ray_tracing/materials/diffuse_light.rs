use super::super::geometry::hittable::HitRecord;
use super::super::math::{ray::Ray, vec3::*};
use super::material::{Material, ScatterRecord};
use super::texture::{SolidColor, TexturePtr};
use std::sync::Arc;

/// 漫射光源材质
pub struct DiffuseLight {
    emit: TexturePtr,
}

impl DiffuseLight {
    /// 从纹理创建光源
    #[inline]
    pub fn new(emit: TexturePtr) -> Self {
        Self { emit }
    }

    /// 从纯色创建光源
    #[inline]
    pub fn new_color(color: Color) -> Self {
        Self {
            emit: Arc::new(SolidColor::new(color)),
        }
    }
}

impl Material for DiffuseLight {
    #[inline]
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
        // 光源不散射光线
        false
    }

    #[inline]
    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.emit.value(u, v, p)
    }
}

impl std::fmt::Debug for DiffuseLight {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiffuseLight")
            .field("emit", &"<Texture>")
            .finish()
    }
}
