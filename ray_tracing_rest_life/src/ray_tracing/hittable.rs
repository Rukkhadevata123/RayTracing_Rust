use std::sync::Arc;

use super::aabb::Aabb;
use super::interval::Interval;
use super::material::Material;
use super::ray::Ray;
use super::util::degrees_to_radians;
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

// 平移变换
pub struct Translate {
    object: Arc<dyn Hittable>,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    pub fn new(object: Arc<dyn Hittable>, offset: Vec3) -> Self {
        // 计算平移后的包围盒
        let bbox = if let Some(obj_bbox) = object.bounding_box() {
            obj_bbox + offset
        } else {
            Aabb::empty()
        };

        Self {
            object,
            offset,
            bbox,
        }
    }
}

impl Hittable for Translate {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 将光线向相反方向偏移
        let offset_r = Ray::new(r.orig - self.offset, r.dir, r.time);

        // 检查偏移后的光线是否与原始物体相交
        if !self.object.hit(&offset_r, ray_t, rec) {
            return false;
        }

        // 将交点向偏移方向移动回来
        rec.p = rec.p + self.offset;

        true
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}

// Y轴旋转变换
pub struct RotateY {
    object: Arc<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}

impl RotateY {
    pub fn new(object: Arc<dyn Hittable>, angle: f64) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        // 计算旋转后的包围盒
        let bbox = if let Some(obj_bbox) = object.bounding_box() {
            let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

            // 检查包围盒的所有顶点
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x = i as f64 * obj_bbox.x.max + (1 - i) as f64 * obj_bbox.x.min;
                        let y = j as f64 * obj_bbox.y.max + (1 - j) as f64 * obj_bbox.y.min;
                        let z = k as f64 * obj_bbox.z.max + (1 - k) as f64 * obj_bbox.z.min;

                        // 应用旋转变换
                        let newx = cos_theta * x + sin_theta * z;
                        let newz = -sin_theta * x + cos_theta * z;

                        let tester = Vec3::new(newx, y, newz);

                        // 更新最小和最大点
                        min.x = min.x.min(tester.x);
                        min.y = min.y.min(tester.y);
                        min.z = min.z.min(tester.z);

                        max.x = max.x.max(tester.x);
                        max.y = max.y.max(tester.y);
                        max.z = max.z.max(tester.z);
                    }
                }
            }

            Aabb::new(
                Interval::new(min.x, max.x),
                Interval::new(min.y, max.y),
                Interval::new(min.z, max.z),
            )
        } else {
            Aabb::empty()
        };

        Self {
            object,
            sin_theta,
            cos_theta,
            bbox,
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 将光线从世界空间转换到对象空间
        let origin = Point3::new(
            self.cos_theta * r.orig.x - self.sin_theta * r.orig.z,
            r.orig.y,
            self.sin_theta * r.orig.x + self.cos_theta * r.orig.z,
        );

        let direction = Vec3::new(
            self.cos_theta * r.dir.x - self.sin_theta * r.dir.z,
            r.dir.y,
            self.sin_theta * r.dir.x + self.cos_theta * r.dir.z,
        );

        let rotated_r = Ray::new(origin, direction, r.time);

        // 在对象空间中检测相交
        if !self.object.hit(&rotated_r, ray_t, rec) {
            return false;
        }

        // 将交点和法线从对象空间转换回世界空间
        let p = rec.p;
        rec.p = Point3::new(
            self.cos_theta * p.x + self.sin_theta * p.z,
            p.y,
            -self.sin_theta * p.x + self.cos_theta * p.z,
        );

        let normal = rec.normal;
        rec.normal = Vec3::new(
            self.cos_theta * normal.x + self.sin_theta * normal.z,
            normal.y,
            -self.sin_theta * normal.x + self.cos_theta * normal.z,
        );

        true
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}

pub trait Hittable: Send + Sync {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool;

    fn bounding_box(&self) -> Option<Aabb> {
        None
    }

    /// 计算给定方向的PDF值
    fn pdf_value(&self, _origin: &Point3, _direction: &Vec3) -> f64 {
        0.0 // 默认实现返回0
    }

    /// 从origin点生成指向该物体的随机方向
    fn random(&self, _origin: &Point3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0) // 默认实现返回某个方向
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
