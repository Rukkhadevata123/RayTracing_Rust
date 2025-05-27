use super::super::math::{aabb::Aabb, interval::Interval, ray::Ray, vec3::*};
use super::super::utils::util::random_int_range;
use super::hittable::{HitRecord, Hittable};
use std::sync::Arc;

/// 可命中物体列表
#[derive(Default)]
pub struct HittableList {
    pub objects: Vec<Arc<dyn Hittable>>,
    bbox: Aabb,
}

impl HittableList {
    /// 创建空列表
    #[inline]
    pub const fn new() -> Self {
        Self {
            objects: Vec::new(),
            bbox: Aabb::empty(),
        }
    }

    /// 添加物体到列表
    pub fn add(&mut self, object: Arc<dyn Hittable>) {
        if let Some(obj_bbox) = object.bounding_box() {
            self.bbox = if self.objects.is_empty() {
                obj_bbox
            } else {
                self.bbox.merge(&obj_bbox)
            };
        }

        self.objects.push(object);
    }

    /// 清空列表
    pub fn clear(&mut self) {
        self.objects.clear();
        self.bbox = Aabb::empty();
    }

    /// 获取物体数量
    #[inline]
    pub fn len(&self) -> usize {
        self.objects.len()
    }

    /// 检查是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.objects.is_empty()
    }
}

impl Hittable for HittableList {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        let mut temp_rec = HitRecord::default();
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

    #[inline]
    fn bounding_box(&self) -> Option<Aabb> {
        if self.is_empty() {
            None
        } else {
            Some(self.bbox)
        }
    }

    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        if self.is_empty() {
            return 0.0;
        }

        let weight = 1.0 / self.objects.len() as f64;
        self.objects
            .iter()
            .map(|obj| weight * obj.pdf_value(origin, direction))
            .sum()
    }

    fn random(&self, origin: &Point3) -> Vec3 {
        if self.is_empty() {
            return Vec3::new(1.0, 0.0, 0.0);
        }

        let random_index = random_int_range(0, self.objects.len() as i32 - 1) as usize;
        self.objects[random_index].random(origin)
    }
}

impl FromIterator<Arc<dyn Hittable>> for HittableList {
    fn from_iter<T: IntoIterator<Item = Arc<dyn Hittable>>>(iter: T) -> Self {
        let mut list = HittableList::new();
        for item in iter {
            list.add(item);
        }
        list
    }
}

impl std::fmt::Debug for HittableList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HittableList")
            .field("objects", &format!("{} objects", self.objects.len()))
            .field("bbox", &self.bbox)
            .finish()
    }
}
