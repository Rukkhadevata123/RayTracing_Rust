use super::super::geometry::hittable::HitRecord;
use super::super::math::{ray::Ray, vec3::*};
use super::super::sampling::pdf::CosinePDF;
use super::material::{Material, ScatterRecord};
use super::texture::TexturePtr;
use std::sync::Arc;

/// 朗伯材质（理想漫反射）
pub struct Lambertian {
    albedo: TexturePtr,
}

impl Lambertian {
    /// 从纯色创建朗伯材质
    #[inline]
    pub fn new(albedo: Color) -> Self {
        Self {
            albedo: Arc::new(super::texture::SolidColor::new(albedo)),
        }
    }

    /// 从纹理创建朗伯材质
    #[inline]
    pub fn new_texture(albedo: TexturePtr) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        let pdf = Arc::new(CosinePDF::new(&rec.normal));

        srec.set_diffuse(attenuation, pdf);
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cos_theta = rec.normal.dot(&scattered.dir.normalize());
        if cos_theta < 0.0 {
            0.0
        } else {
            cos_theta / std::f64::consts::PI
        }
    }
}

impl std::fmt::Debug for Lambertian {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Lambertian")
            .field("albedo", &"<Texture>")
            .finish()
    }
}
