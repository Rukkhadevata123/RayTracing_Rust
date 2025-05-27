use std::sync::Arc;

use super::aabb::Aabb;
use super::hittable::{HitRecord, Hittable};
use super::interval::Interval;
use super::ray::Ray;
use super::util::random_int_range;
use super::vec3::{Point3, Vec3};

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
        let object_bbox = object.bounding_box().unwrap_or_default();

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
        Some(self.bbox)
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        if self.objects.is_empty() {
            return 0.0;
        }

        // 对所有对象的PDF值求平均
        let weight = 1.0 / self.objects.len() as f64;
        let mut sum = 0.0;

        for object in &self.objects {
            sum += weight * object.pdf_value(origin, direction);
        }

        sum
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        if self.objects.is_empty() {
            // 如果列表为空，返回默认方向
            return Vec3::new(1.0, 0.0, 0.0);
        }

        // 随机选择一个对象进行采样
        let int_size = self.objects.len();
        let random_index = random_int_range(0, int_size as i32 - 1) as usize;
        self.objects[random_index].random(origin)
    }
}
