use super::super::geometry::hittable::HitRecord;
use super::super::math::{ray::Ray, vec3::*};
use super::super::sampling::pdf::SpherePDF;
use super::material::{Material, ScatterRecord};
use super::texture::{SolidColor, TexturePtr};
use std::sync::Arc;

/// 各向同性散射材质，用于体积介质
pub struct Isotropic {
    albedo: TexturePtr,
}

impl Isotropic {
    /// 从纹理创建各向同性材质
    #[inline]
    pub fn new(texture: TexturePtr) -> Self {
        Self { albedo: texture }
    }

    /// 从颜色创建各向同性材质
    #[inline]
    pub fn new_color(color: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(color)),
        }
    }
}

impl Material for Isotropic {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        let pdf = Arc::new(SpherePDF::new());

        srec.set_diffuse(attenuation, pdf);
        true
    }

    #[inline]
    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        // 各向同性散射在所有方向的概率相等
        1.0 / (4.0 * std::f64::consts::PI)
    }
}

impl std::fmt::Debug for Isotropic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Isotropic")
            .field("albedo", &"<Texture>")
            .finish()
    }
}
