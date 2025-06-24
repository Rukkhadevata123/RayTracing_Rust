use super::hittable::{HitRecord, Hittable};
use super::hittable_list::HittableList;
use crate::ray_tracing::materials::material::Material;
use crate::ray_tracing::math::aabb::Aabb;
use crate::ray_tracing::math::interval::Interval;
use crate::ray_tracing::math::ray::Ray;
use crate::ray_tracing::math::vec3::*;
use crate::ray_tracing::utils::random::random_double;
use std::sync::Arc;

/// 四边形几何体
pub struct Quad {
    q: Point3,              // 四边形起始点
    u: Vec3,                // 第一条边向量
    v: Vec3,                // 第二条边向量
    mat: Arc<dyn Material>, // 材质
    bbox: Aabb,             // 包围盒
    normal: Vec3,           // 表面法线
    d: f64,                 // 平面方程常数项
    w: Vec3,                // 重心坐标计算辅助向量
    area: f64,              // 四边形面积
}

impl Quad {
    /// 创建四边形
    #[inline]
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Arc<dyn Material>) -> Self {
        let n = u.cross(&v);
        let normal = n.normalize();
        let d = normal.dot(&(q.coords)); // 使用coords访问向量坐标
        let w = n / n.dot(&n);
        let area = n.norm();

        // 计算包围盒
        let bbox_diag1 = Aabb::new_point(q, q + u + v);
        let bbox_diag2 = Aabb::new_point(q + u, q + v);
        let bbox = bbox_diag1.merge(&bbox_diag2);

        Self {
            q,
            u,
            v,
            mat,
            bbox,
            normal,
            d,
            w,
            area,
        }
    }

    /// 检查点是否在四边形内部
    #[inline]
    fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b) {
            return false;
        }

        rec.u = a;
        rec.v = b;
        true
    }
}

impl Hittable for Quad {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let denom = self.normal.dot(&r.dir);

        if denom.abs() < 1e-8 {
            return false;
        }

        let t = (self.d - self.normal.dot(&r.orig.coords)) / denom; // 使用coords
        if !ray_t.contains(t) {
            return false;
        }

        // 计算交点并检查是否在四边形内
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;
        let alpha = self.w.dot(&planar_hitpt_vector.cross(&self.v));
        let beta = self.w.dot(&self.u.cross(&planar_hitpt_vector));

        if !self.is_interior(alpha, beta, rec) {
            return false;
        }

        // 设置命中记录
        rec.t = t;
        rec.p = intersection;
        rec.mat = self.mat.clone();
        rec.set_face_normal(r, &self.normal);

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

        let distance_squared = rec.t * rec.t * direction.norm_squared();
        let cosine = (direction.dot(&rec.normal) / direction.norm()).abs();

        distance_squared / (cosine * self.area)
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        let p = self.q + (random_double() * self.u) + (random_double() * self.v);
        p - *origin
    }
}

/// 创建盒子（六个四边形面）
pub fn box_new(a: Point3, b: Point3, mat: Arc<dyn Material>) -> HittableList {
    let mut sides = HittableList::new();

    let min = Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y - min.y, 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z - min.z);

    // 六个面
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, max.z),
        dx,
        dy,
        mat.clone(),
    ))); // 前面
    sides.add(Arc::new(Quad::new(
        Point3::new(max.x, min.y, max.z),
        -dz,
        dy,
        mat.clone(),
    ))); // 右面
    sides.add(Arc::new(Quad::new(
        Point3::new(max.x, min.y, min.z),
        -dx,
        dy,
        mat.clone(),
    ))); // 后面
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, min.z),
        dz,
        dy,
        mat.clone(),
    ))); // 左面
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, max.y, max.z),
        dx,
        -dz,
        mat.clone(),
    ))); // 顶面
    sides.add(Arc::new(Quad::new(
        Point3::new(min.x, min.y, min.z),
        dx,
        dz,
        mat.clone(),
    ))); // 底面

    sides
}

impl std::fmt::Debug for Quad {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Quad")
            .field("q", &self.q)
            .field("u", &self.u)
            .field("v", &self.v)
            .field("mat", &"<Material>")
            .field("bbox", &self.bbox)
            .field("normal", &self.normal)
            .field("d", &self.d)
            .field("w", &self.w)
            .field("area", &self.area)
            .finish()
    }
}
