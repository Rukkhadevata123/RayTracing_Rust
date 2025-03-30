use std::sync::Arc;

use super::aabb::Aabb;
use super::hittable::{HitRecord, Hittable};
use super::interval::Interval;
use super::ray::Ray;

pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable + Send + Sync>>, 
    bbox: Aabb,
}

impl HittableList {
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::default(),
        }
    }

    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        // 先获取边界盒
        let object_bbox = object.bounding_box().unwrap_or(Aabb::default());

        // 计算新的合并边界盒
        self.bbox = if self.bbox.is_empty() {
            object_bbox
        } else {
            self.bbox.merge(&object_bbox)
        };

        // 然后将对象添加到列表中
        self.objects.push(object);
    }

    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let mut temp_rec = rec.clone();
        let mut hit_anything = false;
        let mut closest_so_far = ray_t.max;

        for object in &self.objects {
            if object.hit(r, Interval::new(ray_t.min, closest_so_far), &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }

        hit_anything
    }
    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox.clone())
    }
}
