use std::sync::Arc;

use super::hittable::HitRecord;
use super::ray::Ray;
use super::texture::{SolidColor, TexturePtr};
use super::util::random_double;
use super::vec3::{Color, Point3, Vec3, dot, reflect, refract, unit_vector};

pub trait Material: Send + Sync {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool;

    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0) // 默认实现返回黑色（不发光）
    }
}

pub struct Lambertian {
    pub albedo: TexturePtr,
}

impl Lambertian {
    pub fn new(albedo: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(albedo)),
        }
    }

    pub fn new_texture(albedo: TexturePtr) -> Self {
        Self { albedo }
    }
}

impl Material for Lambertian {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let mut scatter_direction = rec.normal + Vec3::random_unit_vector();

        // 捕获接近零的散射方向
        if scatter_direction.near_zero() {
            scatter_direction = rec.normal;
        }

        *scattered = Ray::new(rec.p, scatter_direction, r_in.time);

        // 从纹理中获取颜色，使用命中点的 u,v 坐标和位置
        *attenuation = self.albedo.value(rec.u, rec.v, &rec.p);

        true
    }
}

pub struct Metal {
    pub albedo: Color,
    pub fuzz: f64,
}

impl Metal {
    pub fn new(albedo: Color, fuzz: f64) -> Self {
        Self {
            albedo,
            fuzz: if fuzz < 1.0 { fuzz } else { 1.0 },
        }
    }
}

impl Material for Metal {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        let reflected = reflect(&unit_vector(&r_in.dir), &rec.normal);
        let direction = reflected + self.fuzz * Vec3::random_unit_vector();
        *scattered = Ray::new(rec.p, direction, r_in.time);
        *attenuation = self.albedo;
        dot(&scattered.dir, &rec.normal) > 0.0
    }
}

pub struct Dielectric {
    pub refraction_index: f64,
}

impl Dielectric {
    pub fn new(refraction_index: f64) -> Self {
        Self { refraction_index }
    }

    fn reflectance(cosine: f64, ref_idx: f64) -> f64 {
        // 使用Schlick近似计算反射率
        let mut r0 = (1.0 - ref_idx) / (1.0 + ref_idx);
        r0 = r0 * r0;
        r0 + (1.0 - r0) * (1.0 - cosine).powi(5)
    }
}

impl Material for Dielectric {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        *attenuation = Color::new(1.0, 1.0, 1.0);
        let ri = if rec.front_face {
            1.0 / self.refraction_index
        } else {
            self.refraction_index
        };

        let unit_direction = unit_vector(&r_in.dir);
        let cos_theta = f64::min(dot(&-unit_direction, &rec.normal), 1.0);
        let sin_theta = (1.0 - cos_theta * cos_theta).sqrt();

        let cannot_refract = ri * sin_theta > 1.0;

        // 使用 if 表达式直接初始化 direction
        let direction = if cannot_refract || Self::reflectance(cos_theta, ri) > random_double() {
            reflect(&unit_direction, &rec.normal)
        } else {
            refract(&unit_direction, &rec.normal, ri)
        };

        *scattered = Ray::new(rec.p, direction, r_in.time);
        true
    }
}

pub struct DiffuseLight {
    emit: TexturePtr,
}

impl DiffuseLight {
    pub fn new(texture: TexturePtr) -> Self {
        Self { emit: texture }
    }

    pub fn new_color(color: Color) -> Self {
        Self {
            emit: Arc::new(SolidColor::new(color)),
        }
    }
}

impl Material for DiffuseLight {
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Color,
        _scattered: &mut Ray,
    ) -> bool {
        // 光源不散射光线，只发射
        false
    }

    fn emitted(&self, u: f64, v: f64, p: &Point3) -> Color {
        self.emit.value(u, v, p)
    }
}

pub struct Isotropic {
    albedo: TexturePtr,
}

impl Isotropic {
    pub fn new(texture: TexturePtr) -> Self {
        Self { albedo: texture }
    }

    pub fn new_color(color: Color) -> Self {
        Self {
            albedo: Arc::new(SolidColor::new(color)),
        }
    }
}

impl Material for Isotropic {
    fn scatter(
        &self,
        r_in: &Ray,
        rec: &HitRecord,
        attenuation: &mut Color,
        scattered: &mut Ray,
    ) -> bool {
        // 各向同性散射 - 向随机方向散射光线
        *scattered = Ray::new(rec.p, Vec3::random_unit_vector(), r_in.time);
        *attenuation = self.albedo.value(rec.u, rec.v, &rec.p);

        true
    }
}

// 用于作为默认材质的简单实现
pub struct NoMaterial {}

impl Material for NoMaterial {
    fn scatter(
        &self,
        _r_in: &Ray,
        _rec: &HitRecord,
        _attenuation: &mut Color,
        _scattered: &mut Ray,
    ) -> bool {
        false // 永远不散射光线
    }
}
