use std::sync::Arc;

use super::aabb::Aabb;
use super::hittable::{HitRecord, Hittable};
use super::interval::Interval;
use super::material::Material;
use super::material::NoMaterial;
use super::ray::Ray;
use super::vec3::{Point3, Vec3, dot};

pub struct Sphere {
    pub center: Ray,
    pub radius: f64,
    pub mat: Arc<dyn Material + Send + Sync>, // 添加 Send + Sync
    pub bbox: Aabb,
}

impl Sphere {
    pub fn new(static_center: Point3, radius: f64, mat: Arc<dyn Material + Send + Sync>) -> Self {
        let revc = Vec3::new(radius, radius, radius);
        let bbox = Aabb::new_point(static_center - revc, static_center + revc);
        Self {
            center: Ray::new(static_center, Vec3::zeros(), 0.0),
            radius,
            mat,
            bbox: bbox,
        }
    }
    pub fn new_moving(
        center0: Point3,
        center1: Point3,
        radius: f64,
        mat: Arc<dyn Material + Send + Sync>,
    ) -> Self {
        // 创建半径向量
        let revc = Vec3::new(radius, radius, radius);

        // 计算起始位置的包围盒
        let box1 = Aabb::new_point(center0 - revc, center0 + revc);

        // 计算结束位置的包围盒
        let box2 = Aabb::new_point(center1 - revc, center1 + revc);

        // 合并两个包围盒，覆盖整个运动轨迹
        Self {
            center: Ray::new(center0, center1 - center0, 0.0),
            radius,
            mat,
            bbox: box1.merge(&box2),
        }
    }

    // 添加获取球面 UV 坐标的方法
    fn get_sphere_uv(p: &Point3) -> (f64, f64) {
        // p: 单位球体表面上的点 (球心在原点)
        // u: 返回 [0,1] 范围内的值，表示经度角
        // v: 返回 [0,1] 范围内的值，表示纬度角

        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + std::f64::consts::PI;

        let u = phi / (2.0 * std::f64::consts::PI);
        let v = theta / std::f64::consts::PI;

        (u, v)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let current_center = self.center.at(r.time);
        let oc = -current_center + r.orig;
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
        let outward_normal = (rec.p - current_center) / self.radius;

        // 计算 UV 坐标
        let (u, v) = Self::get_sphere_uv(&outward_normal);
        rec.u = u;
        rec.v = v;

        rec.set_face_normal(r, &outward_normal);
        rec.mat = self.mat.clone();

        true
    }
    fn bounding_box(&self) -> Option<Aabb> {
        // 返回包围盒
        Some(self.bbox.clone())
    }
}

impl Default for Sphere {
    fn default() -> Self {
        Self {
            center: Ray::default(),
            radius: 0.0,
            mat: Arc::new(NoMaterial {}),
            bbox: Aabb::default(), // 使用空 AABB
        }
    }
}
