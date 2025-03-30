use std::sync::Arc;

use super::aabb::Aabb;
use super::interval::Interval;
use super::material::Material;
use super::ray::Ray;
use super::vec3::{Point3, Vec3, dot};

pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub mat: Arc<dyn Material + Send + Sync>, // 添加 Send + Sync
    pub t: f64,
    pub u: f64,
    pub v: f64,
    pub front_face: bool,
}

impl HitRecord {
    pub fn new(
        p: Point3,
        normal: Vec3,
        mat: Arc<dyn Material + Send + Sync>,
        t: f64,
        u: f64, // 添加 u 参数
        v: f64, // 添加 v 参数
        front_face: bool,
    ) -> Self {
        Self {
            p,
            normal,
            mat,
            t,
            u,
            v,
            front_face,
        }
    }

    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        // 设置命中记录的法线向量
        let outward_normal_unit = outward_normal.unit_vector();
        self.front_face = dot(&r.dir, &outward_normal_unit) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        };
    }
}

pub trait Hittable: Send + Sync {
    // 添加 Send + Sync
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool;
    fn bounding_box(&self) -> Option<Aabb> {
        None
    }
}

impl Clone for HitRecord {
    fn clone(&self) -> Self {
        Self {
            p: self.p,
            normal: self.normal,
            mat: self.mat.clone(),
            t: self.t,
            u: self.u, // 添加 u 字段
            v: self.v, // 添加 v 字段
            front_face: self.front_face,
        }
    }
}

impl Default for HitRecord {
    fn default() -> Self {
        Self::new(
            Point3::default(),
            Vec3::default(),
            Arc::new(super::material::NoMaterial {}),
            0.0,
            0.0, // u 默认值
            0.0, // v 默认值
            false,
        )
    }
}
