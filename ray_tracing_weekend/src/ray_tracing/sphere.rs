use std::sync::Arc;

use super::hittable::{HitRecord, Hittable};
use super::interval::Interval;
use super::material::Material;
use super::ray::Ray;
use super::vec3::{Point3, dot};

pub struct Sphere {
    pub center: Point3,
    pub radius: f64,
    pub mat: Arc<dyn Material + Send + Sync>, // 添加 Send + Sync
}

impl Sphere {
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Material + Send + Sync>) -> Self {
        Self {
            center,
            radius,
            mat,
        }
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let oc = r.orig - self.center;
        let a = r.dir.length_squared();
        let half_b = dot(&oc, &r.dir);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        // 寻找射线与球体交点的最近t值
        let mut root = (-half_b - sqrtd) / a;
        if !ray_t.surrounds(root) {
            // 改用 surrounds 而不是 contains，与C++版本一致
            root = (-half_b + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, &outward_normal);
        rec.mat = self.mat.clone();

        true
    }
}
