use super::super::geometry::hittable::HitRecord;
use super::super::math::{ray::Ray, vec3::*};
use super::super::sampling::pdf::PDF;
use std::sync::Arc;

/// 散射记录，包含材质散射的所有信息
pub struct ScatterRecord {
    pub attenuation: Color,
    pub pdf_ptr: Option<Arc<dyn PDF>>,
    pub skip_pdf: bool,
    pub skip_pdf_ray: Ray,
}

impl ScatterRecord {
    #[inline]
    pub fn new() -> Self {
        Self {
            attenuation: Color::new(0.0, 0.0, 0.0),
            pdf_ptr: None,
            skip_pdf: false,
            skip_pdf_ray: Ray::default(),
        }
    }

    /// 设置为跳过PDF的散射（如镜面反射）
    #[inline]
    pub fn set_specular(&mut self, attenuation: Color, ray: Ray) {
        self.attenuation = attenuation;
        self.skip_pdf = true;
        self.skip_pdf_ray = ray;
        self.pdf_ptr = None;
    }

    /// 设置为使用PDF的散射（如漫反射）
    #[inline]
    pub fn set_diffuse(&mut self, attenuation: Color, pdf: Arc<dyn PDF>) {
        self.attenuation = attenuation;
        self.skip_pdf = false;
        self.pdf_ptr = Some(pdf);
    }
}

impl Default for ScatterRecord {
    fn default() -> Self {
        Self::new()
    }
}

/// 材质trait，定义光线与表面的交互行为
pub trait Material: Send + Sync + std::fmt::Debug {
    /// 主要的散射方法
    fn scatter(&self, r_in: &Ray, rec: &HitRecord, srec: &mut ScatterRecord) -> bool;

    /// 材质发射的光（仅用于光源）
    #[inline]
    fn emitted(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        Color::new(0.0, 0.0, 0.0)
    }

    /// 散射PDF值（用于重要性采样）
    #[inline]
    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        0.0
    }
}

/// 空材质，用作默认值或虚拟光源
#[derive(Debug, Default)]
pub struct NoMaterial;

impl Material for NoMaterial {
    #[inline]
    fn scatter(&self, _r_in: &Ray, _rec: &HitRecord, _srec: &mut ScatterRecord) -> bool {
        false
    }

    #[inline]
    fn scattering_pdf(&self, _r_in: &Ray, _rec: &HitRecord, _scattered: &Ray) -> f64 {
        1.0 // 返回合理的非零值，避免除零错误
    }
}
