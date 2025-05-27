use super::super::super::math::{aabb::Aabb, interval::Interval, ray::Ray, vec3::*};
use super::super::hittable::{HitRecord, Hittable};
use std::sync::Arc;

/// 平移变换
pub struct Translate {
    object: Arc<dyn Hittable>,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    /// 创建平移变换
    #[inline]
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
        // 将光线向相反方向偏移到对象的局部坐标系
        let offset_r = Ray::new(r.orig - self.offset, r.dir, r.time);

        // 检查偏移后的光线是否与原始物体相交
        if !self.object.hit(&offset_r, ray_t, rec) {
            return false;
        }

        // 将交点位置转换回世界坐标系
        rec.p = rec.p + self.offset;
        true
    }

    #[inline]
    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }

    #[inline]
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        // 将原点转换到对象的局部坐标系
        let local_origin = *origin - self.offset;
        self.object.pdf_value(&local_origin, direction)
    }

    #[inline]
    fn random(&self, origin: &Point3) -> Vec3 {
        // 将原点转换到对象的局部坐标系
        let local_origin = *origin - self.offset;
        self.object.random(&local_origin)
    }
}

impl std::fmt::Debug for Translate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Translate")
            .field("object", &"<Hittable>")
            .field("offset", &self.offset)
            .field("bbox", &self.bbox)
            .finish()
    }
}
