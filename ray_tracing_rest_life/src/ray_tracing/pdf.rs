use std::sync::Arc;

use super::hittable::Hittable;
use super::onb::ONB;
use super::util::random_double;
use super::vec3::{Point3, Vec3, dot, random_cosine_direction, random_unit_vector, unit_vector};

/// 概率密度函数特质，用于重要性采样
pub trait PDF {
    /// 计算给定方向的概率密度
    fn value(&self, direction: &Vec3) -> f64;

    /// 生成一个符合该PDF的随机方向
    fn generate(&self) -> Vec3;
}

/// 均匀球面PDF
pub struct SpherePDF;

impl SpherePDF {
    pub fn new() -> Self {
        Self
    }
}

impl PDF for SpherePDF {
    fn value(&self, _direction: &Vec3) -> f64 {
        // 均匀分布在单位球面上的PDF值为常量 1/(4π)
        1.0 / (4.0 * std::f64::consts::PI)
    }

    fn generate(&self) -> Vec3 {
        // 生成均匀分布在单位球面上的随机方向
        random_unit_vector()
    }
}

/// 余弦加权PDF，用于漫反射材质
pub struct CosinePDF {
    uvw: ONB,
}

impl CosinePDF {
    pub fn new(w: &Vec3) -> Self {
        Self { uvw: ONB::new(w) }
    }
}

impl PDF for CosinePDF {
    fn value(&self, direction: &Vec3) -> f64 {
        // 计算方向与法线的余弦值
        let cosine_theta = dot(&unit_vector(direction), self.uvw.w());

        // 确保值非负，并且除以π归一化
        f64::max(0.0, cosine_theta / std::f64::consts::PI)
    }

    fn generate(&self) -> Vec3 {
        // 生成余弦加权的随机方向
        // 先在局部坐标系生成，再转换到世界坐标系
        self.uvw.local_to_world(&random_cosine_direction())
    }
}

/// 可命中物体的PDF，用于直接光源采样
pub struct HittablePDF {
    objects: Arc<dyn Hittable>,
    origin: Point3,
}

impl HittablePDF {
    pub fn new(objects: Arc<dyn Hittable>, origin: &Point3) -> Self {
        Self {
            objects,
            origin: *origin,
        }
    }
}

impl PDF for HittablePDF {
    fn value(&self, direction: &Vec3) -> f64 {
        // 委托给可命中对象计算PDF值
        self.objects.pdf_value(&self.origin, direction)
    }

    fn generate(&self) -> Vec3 {
        // 委托给可命中对象生成随机方向
        self.objects.random(&self.origin)
    }
}

/// 混合PDF，组合多个PDF
pub struct MixturePDF {
    pdfs: [Arc<dyn PDF>; 2],
}

impl MixturePDF {
    pub fn new(p0: Arc<dyn PDF>, p1: Arc<dyn PDF>) -> Self {
        Self { pdfs: [p0, p1] }
    }
}

impl PDF for MixturePDF {
    fn value(&self, direction: &Vec3) -> f64 {
        // 对各个PDF的值进行等权重平均
        0.5 * self.pdfs[0].value(direction) + 0.5 * self.pdfs[1].value(direction)
    }

    fn generate(&self) -> Vec3 {
        // 随机选择一个PDF生成方向
        if random_double() < 0.5 {
            self.pdfs[0].generate()
        } else {
            self.pdfs[1].generate()
        }
    }
}
