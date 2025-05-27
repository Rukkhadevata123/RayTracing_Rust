use std::sync::Arc;

use super::aabb::Aabb;
use super::hittable::{HitRecord, Hittable};
use super::hittable_list::HittableList;
use super::interval::Interval;
use super::material::Material;
use super::ray::Ray;
use super::util::random_double;
use super::vec3::{Point3, Vec3, cross, dot, unit_vector};

pub struct Quad {
    q: Point3,                            // 四边形的起始点
    u: Vec3,                              // 第一条边的向量
    v: Vec3,                              // 第二条边的向量
    mat: Arc<dyn Material + Send + Sync>, // 材质
    bbox: Aabb,                           // 包围盒
    normal: Vec3,                         // 表面法线
    d: f64,                               // 平面方程常数项
    w: Vec3,                              // 计算重心坐标的辅助向量
    area: f64,                            // 四边形面积
}

impl Quad {
    pub fn new(q: Point3, u: Vec3, v: Vec3, mat: Arc<dyn Material + Send + Sync>) -> Self {
        // 计算法线向量 n = u × v (叉乘)
        let n = cross(&u, &v);
        let normal = unit_vector(&n);
        let d = dot(&normal, &q);

        // 计算重心坐标的辅助向量
        let w = n / dot(&n, &n);

        // 计算包围盒
        let bbox_diagonal1 = Aabb::new_point(q, q + u + v);
        let bbox_diagonal2 = Aabb::new_point(q + u, q + v);
        let bbox = bbox_diagonal1.merge(&bbox_diagonal2);

        // 计算面积
        let area = n.length();

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

    // 检查点是否在四边形内部
    fn is_interior(&self, a: f64, b: f64, rec: &mut HitRecord) -> bool {
        // 使用单位区间 [0,1] 判断是否在四边形内
        let unit_interval = Interval::new(0.0, 1.0);

        // 如果超出单位区间，即不在四边形内
        if !unit_interval.contains(a) || !unit_interval.contains(b) {
            return false;
        }

        // 设置纹理坐标
        rec.u = a;
        rec.v = b;

        true
    }
}

impl Hittable for Quad {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 计算光线与平面的交点
        let denom = dot(&self.normal, &r.dir);

        // 如果光线与平面平行，则不相交
        if denom.abs() < 1e-8 {
            return false;
        }

        // 计算参数 t 并检查是否在有效区间内
        let t = (self.d - dot(&self.normal, &r.orig)) / denom;
        if !ray_t.contains(t) {
            return false;
        }

        // 计算交点
        let intersection = r.at(t);
        let planar_hitpt_vector = intersection - self.q;

        // 计算参数坐标 (alpha, beta)
        let alpha = dot(&self.w, &cross(&planar_hitpt_vector, &self.v));
        let beta = dot(&self.w, &cross(&self.u, &planar_hitpt_vector));

        // 检查是否在四边形内部
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

    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox.clone())
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

        let distance_squared = rec.t * rec.t * direction.length_squared();
        let cosine = (dot(direction, &rec.normal) / direction.length()).abs();

        distance_squared / (cosine * self.area)
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        // 生成四边形上的随机点
        let p = self.q + (random_double() * self.u) + (random_double() * self.v);
        // 返回从原点到该点的方向
        p - *origin
    }
}

// 创建一个盒子 (六个面)
pub fn box_new(a: Point3, b: Point3, mat: Arc<dyn Material + Send + Sync>) -> HittableList {
    let mut sides = HittableList::new();

    // 计算最小和最大顶点
    let min = Point3::new(a.x.min(b.x), a.y.min(b.y), a.z.min(b.z));
    let max = Point3::new(a.x.max(b.x), a.y.max(b.y), a.z.max(b.z));

    // 计算沿各轴的边向量
    let dx = Vec3::new(max.x - min.x, 0.0, 0.0);
    let dy = Vec3::new(0.0, max.y - min.y, 0.0);
    let dz = Vec3::new(0.0, 0.0, max.z - min.z);

    // 添加六个面
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
