use super::hittable::{HitRecord, Hittable};
use crate::ray_tracing::materials::material::Material;
use crate::ray_tracing::math::aabb::Aabb;
use crate::ray_tracing::math::interval::Interval;
use crate::ray_tracing::math::onb::ONB;
use crate::ray_tracing::math::ray::Ray;
use crate::ray_tracing::math::vec3::*;
use crate::ray_tracing::utils::random::random_double;
use std::sync::Arc;

/// 球体几何体
pub struct Sphere {
    center: Ray, // 使用Ray表示运动轨迹：center.orig为起始位置，center.dir为位移向量
    radius: f64,
    mat: Arc<dyn Material>,
    bbox: Aabb,
}

impl Sphere {
    /// 创建静态球体
    #[inline]
    pub fn new(center: Point3, radius: f64, mat: Arc<dyn Material>) -> Self {
        let rvec = Vec3::new(radius, radius, radius);
        let bbox = Aabb::new_point(center - rvec, center + rvec);

        Self {
            center: Ray::new(center, Vec3::zeros(), 0.0),
            radius,
            mat,
            bbox,
        }
    }

    /// 创建运动球体
    #[inline]
    pub fn new_moving(
        center0: Point3,
        center1: Point3,
        radius: f64,
        mat: Arc<dyn Material>,
    ) -> Self {
        let rvec = Vec3::new(radius, radius, radius);
        let box0 = Aabb::new_point(center0 - rvec, center0 + rvec);
        let box1 = Aabb::new_point(center1 - rvec, center1 + rvec);
        let bbox = box0.merge(&box1);

        Self {
            center: Ray::new(center0, center1 - center0, 0.0),
            radius,
            mat,
            bbox,
        }
    }

    /// 获取球面UV坐标
    #[inline]
    fn get_sphere_uv(p: &Vec3) -> (f64, f64) {
        // p: 单位球体表面上的点 (球心在原点)
        let theta = (-p.y).acos();
        let phi = (-p.z).atan2(p.x) + std::f64::consts::PI;

        let u = phi / (2.0 * std::f64::consts::PI);
        let v = theta / std::f64::consts::PI;

        (u, v)
    }

    /// 生成指向球体的随机方向
    fn random_to_sphere(&self, distance_squared: f64) -> Vec3 {
        let r1 = random_double();
        let r2 = random_double();
        let z = 1.0 + r2 * ((1.0 - self.radius * self.radius / distance_squared).sqrt() - 1.0);

        let phi = 2.0 * std::f64::consts::PI * r1;
        let x = phi.cos() * (1.0 - z * z).sqrt();
        let y = phi.sin() * (1.0 - z * z).sqrt();

        Vec3::new(x, y, z)
    }
}

impl Hittable for Sphere {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let current_center = self.center.at(r.time);
        let oc = r.orig - current_center;

        let a = r.dir.norm_squared();
        let half_b = oc.dot(&r.dir);
        let c = oc.norm_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;
        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();
        let mut root = (-half_b - sqrtd) / a;

        if !ray_t.surrounds(root) {
            root = (-half_b + sqrtd) / a;
            if !ray_t.surrounds(root) {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        // 计算 UV 坐标 - 修复类型不匹配
        let outward_normal_vec = (rec.p - current_center) / self.radius;
        let (u, v) = Self::get_sphere_uv(&outward_normal_vec);
        rec.u = u;
        rec.v = v;

        rec.set_face_normal(r, &outward_normal_vec);
        rec.mat = self.mat.clone();

        true
    }

    #[inline]
    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        let mut rec = HitRecord::default();
        if !self.hit(
            &Ray::new(*origin, *direction, 0.0),
            Interval::new(0.001, f64::INFINITY),
            &mut rec,
        ) {
            return 0.0;
        }

        let current_center = self.center.at(0.0);
        let dist_squared = (current_center - *origin).norm_squared();
        let cos_theta_max = (1.0 - self.radius * self.radius / dist_squared).sqrt();
        let solid_angle = 2.0 * std::f64::consts::PI * (1.0 - cos_theta_max);

        1.0 / solid_angle
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let current_center = self.center.at(0.0);
        let direction = current_center - *origin;
        let distance_squared = direction.norm_squared();

        let onb = ONB::new(&direction);
        onb.local_to_world(&self.random_to_sphere(distance_squared))
    }
}

impl std::fmt::Debug for Sphere {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sphere")
            .field("center", &self.center)
            .field("radius", &self.radius)
            .field("mat", &"<Material>")
            .field("bbox", &self.bbox)
            .finish()
    }
}
