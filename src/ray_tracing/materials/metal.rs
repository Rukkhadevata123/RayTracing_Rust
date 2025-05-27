use super::super::geometry::hittable::HitRecord;
use super::super::math::{ray::Ray, vec3::*};
use super::material::{Material, ScatterRecord};

/// 金属材质
#[derive(Debug)]
pub struct Metal {
    albedo: Color,
    fuzz: f64, // 模糊度，0为完美镜面，1为完全模糊
}

impl Metal {
    /// 创建金属材质
    #[inline]
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: fuzz.clamp(0.0, 1.0), // 限制模糊度在合理范围内
        }
    }
}

impl Material for Metal {
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let reflected = r_in.dir.normalize().reflect(&rec.normal);
        let scattered_dir = reflected + self.fuzz * Vec3::random_in_unit_sphere();

        // 检查散射方向是否在表面上方
        if scattered_dir.dot(&rec.normal) <= 0.0 {
            return false;
        }

        let scattered_ray = Ray::new(rec.p, scattered_dir, r_in.time);
        srec.set_specular(self.albedo, scattered_ray);
        true
    }
}
