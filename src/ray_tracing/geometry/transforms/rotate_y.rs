use super::super::hittable::{HitRecord, Hittable};
use crate::ray_tracing::math::aabb::Aabb;
use crate::ray_tracing::math::interval::Interval;
use crate::ray_tracing::math::ray::Ray;
use crate::ray_tracing::math::vec3::*;
use crate::ray_tracing::utils::random::degrees_to_radians;
use std::sync::Arc;

/// Y轴旋转变换
pub struct RotateY {
    object: Arc<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}

impl RotateY {
    /// 创建Y轴旋转变换
    #[inline]
    pub fn new(object: Arc<dyn Hittable>, angle: f64) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        // 计算旋转后的包围盒
        let bbox = if let Some(obj_bbox) = object.bounding_box() {
            let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
            let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

            // 检查原包围盒的所有8个顶点，找到旋转后的新包围盒
            for i in 0..2 {
                for j in 0..2 {
                    for k in 0..2 {
                        let x = i as f64 * obj_bbox.x.max + (1 - i) as f64 * obj_bbox.x.min;
                        let y = j as f64 * obj_bbox.y.max + (1 - j) as f64 * obj_bbox.y.min;
                        let z = k as f64 * obj_bbox.z.max + (1 - k) as f64 * obj_bbox.z.min;

                        // 应用Y轴旋转变换
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

            Aabb::new_point(min, max)
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

    /// 将点从世界坐标系转换到对象的局部坐标系
    #[inline]
    fn world_to_local(&self, world_point: &Point3) -> Point3 {
        Point3::new(
            self.cos_theta * world_point.x - self.sin_theta * world_point.z,
            world_point.y,
            self.sin_theta * world_point.x + self.cos_theta * world_point.z,
        )
    }

    /// 将点从对象的局部坐标系转换到世界坐标系
    #[inline]
    fn local_to_world(&self, local_point: &Point3) -> Point3 {
        Point3::new(
            self.cos_theta * local_point.x + self.sin_theta * local_point.z,
            local_point.y,
            -self.sin_theta * local_point.x + self.cos_theta * local_point.z,
        )
    }

    /// 将向量从世界坐标系转换到对象的局部坐标系
    #[inline]
    fn world_to_local_vec(&self, world_vec: &Vec3) -> Vec3 {
        Vec3::new(
            self.cos_theta * world_vec.x - self.sin_theta * world_vec.z,
            world_vec.y,
            self.sin_theta * world_vec.x + self.cos_theta * world_vec.z,
        )
    }

    /// 将向量从对象的局部坐标系转换到世界坐标系
    #[inline]
    fn local_to_world_vec(&self, local_vec: &Vec3) -> Vec3 {
        Vec3::new(
            self.cos_theta * local_vec.x + self.sin_theta * local_vec.z,
            local_vec.y,
            -self.sin_theta * local_vec.x + self.cos_theta * local_vec.z,
        )
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 将光线从世界坐标系转换到对象的局部坐标系
        let origin = self.world_to_local(&r.orig);
        let direction = self.world_to_local_vec(&r.dir);
        let rotated_r = Ray::new(origin, direction, r.time);

        // 在对象的局部坐标系中检测相交
        if !self.object.hit(&rotated_r, ray_t, rec) {
            return false;
        }

        // 将交点和法线从对象的局部坐标系转换回世界坐标系
        rec.p = self.local_to_world(&rec.p);
        rec.normal = self.local_to_world_vec(&rec.normal);

        true
    }

    #[inline]
    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }

    #[inline]
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        // 将原点和方向转换到对象的局部坐标系
        let local_origin = self.world_to_local(origin);
        let local_direction = self.world_to_local_vec(direction);
        self.object.pdf_value(&local_origin, &local_direction)
    }

    #[inline]
    fn random(&self, origin: &Point3) -> Vec3 {
        // 将原点转换到对象的局部坐标系
        let local_origin = self.world_to_local(origin);
        let local_direction = self.object.random(&local_origin);
        // 将生成的方向转换回世界坐标系
        self.local_to_world_vec(&local_direction)
    }
}

impl std::fmt::Debug for RotateY {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RotateY")
            .field("object", &"<Hittable>")
            .field("sin_theta", &self.sin_theta)
            .field("cos_theta", &self.cos_theta)
            .field("bbox", &self.bbox)
            .finish()
    }
}
