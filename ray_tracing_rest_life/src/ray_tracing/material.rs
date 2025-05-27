use std::sync::Arc;

use super::hittable::HitRecord;
use super::pdf::{CosinePDF, PDF, SpherePDF};
use super::ray::Ray;
use super::texture::{SolidColor, TexturePtr};
use super::util::random_double;
use super::vec3::{Color, Point3, Vec3, dot, reflect, refract, unit_vector};

// 新增的 scatter_record 结构体，与 C++ 类似
pub struct ScatterRecord {
    pub attenuation: Color,
    pub pdf_ptr: Option<Arc<dyn PDF>>, // PDF指针，可能为None
    pub skip_pdf: bool,                // 是否跳过PDF
    pub skip_pdf_ray: Ray,             // 不使用PDF时的散射光线
}

impl ScatterRecord {
    pub fn new() -> Self {
        Self {
            attenuation: Color::zeros(),
            pdf_ptr: None,
            skip_pdf: false,
            skip_pdf_ray: Ray::default(),
        }
    }
}

pub trait Material: Send + Sync {
    // 新版的scatter方法，使用ScatterRecord
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
        false
    }

    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0) // 默认实现返回黑色（不发光）
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        // 默认实现返回0.0
        0.0
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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        srec.pdf_ptr = Some(Arc::new(CosinePDF::new(&rec.normal)));
        srec.skip_pdf = false;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, rec: &HitRecord, scattered: &Ray) -> f64 {
        let cosine_theta = rec.normal.dot(&unit_vector(&scattered.dir));
        if cosine_theta < 0.0 {
            0.0
        } else {
            cosine_theta / std::f64::consts::PI
        }
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        let reflected = reflect(&unit_vector(&r_in.dir), &rec.normal);
        let direction = reflected + self.fuzz * Vec3::random_unit_vector();

        srec.attenuation = self.albedo;
        srec.pdf_ptr = None;
        srec.skip_pdf = true;
        srec.skip_pdf_ray = Ray::new(rec.p, direction, r_in.time);

        dot(&srec.skip_pdf_ray.dir, &rec.normal) > 0.0
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
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = Color::new(1.0, 1.0, 1.0);
        srec.pdf_ptr = None;
        srec.skip_pdf = true;

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

        srec.skip_pdf_ray = Ray::new(rec.p, direction, r_in.time);

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
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
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
    fn scatter(&self, _r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool {
        srec.attenuation = self.albedo.value(rec.u, rec.v, &rec.p);
        srec.pdf_ptr = Some(Arc::new(SpherePDF::new()));
        srec.skip_pdf = false;
        true
    }

    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 / (4.0 * std::f64::consts::PI)
    }
}

// 用于作为默认材质的简单实现
pub struct NoMaterial {}

impl Material for NoMaterial {
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
        false // 永远不散射光线
    }
}
