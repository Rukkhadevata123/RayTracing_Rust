use super::super::geometry::hittable::HitRecord;
use super::super::math::{ray::Ray, vec3::*};
use super::super::utils::util::random_double;
use super::material::{Material, ScatterRecord};

/// 电介质材质（玻璃等透明材质）
#[derive(Debug)]
pub struct Dielectric {
    refraction_index: f64, // 折射率
}

impl Dielectric {
    /// 创建电介质材质
    #[inline]
    pub const fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    /// Schlick近似计算反射率
    #[inline]
    fn reflectance(cosine: f64, refraction_ratio: f64) -> f64 {
        let r0 = (1.0 - refraction_ratio) / (1.0 + refraction_ratio);
        let r0_squared = r0 * r0;
        r0_squared + (1.0 - r0_squared) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = r_in.dir.normalize();
        let cos_theta = (-unit_direction).dot(&rec.normal).min(1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;

        let direction = if cannot_refract || Self::reflectance(cos_theta, ri) > random_double() {
            // 全内反射或随机反射
            unit_direction.reflect(&rec.normal)
        } else {
            // 折射
            unit_direction.refract(&rec.normal, ri)
        };

        let scattered_ray = Ray::new(rec.p, direction, r_in.time);
        srec.set_specular(Color::new(1.0, 1.0, 1.0), scattered_ray);
        true
    }
}
