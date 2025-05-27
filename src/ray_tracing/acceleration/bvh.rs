use super::super::geometry::{
    hittable::{HitRecord, Hittable},
    hittable_list::HittableList,
};
use super::super::math::{aabb::Aabb, interval::Interval, ray::Ray, vec3::*};
use std::cmp::Ordering;
use std::sync::Arc;

/// BVH 节点，用于加速光线与场景的交点计算
pub struct BvhNode {
    left: Arc<dyn Hittable>,
    right: Arc<dyn Hittable>,
    bbox: Aabb,
}

impl BvhNode {
    /// 从可命中对象列表构造BVH
    #[inline]
    pub fn new(list: &HittableList) -> Self {
        Self::from_slice(&list.objects, 0, list.objects.len())
    }

    /// 从对象切片构造BVH
    fn from_slice(objects: &[Arc<dyn Hittable>], start: usize, end: usize) -> Self {
        // 构建包含所有对象的边界盒
        let mut bbox = Aabb::empty();
        for object in &objects[start..end] {
            if let Some(obj_bbox) = object.bounding_box() {
                bbox = bbox.merge(&obj_bbox);
            }
        }

        let object_span = end - start;
        let axis = bbox.longest_axis();

        match object_span {
            1 => {
                // 只有一个对象，左右子节点相同
                let obj = objects[start].clone();
                Self {
                    left: obj.clone(),
                    right: obj,
                    bbox,
                }
            }
            2 => {
                // 两个对象，根据轴排序
                let (left, right) = if Self::box_compare(&objects[start], &objects[start + 1], axis)
                    == Ordering::Less
                {
                    (objects[start].clone(), objects[start + 1].clone())
                } else {
                    (objects[start + 1].clone(), objects[start].clone())
                };
                Self { left, right, bbox }
            }
            _ => {
                // 多个对象，排序并递归分割
                let mut sorted_objects = objects[start..end].to_vec();
                sorted_objects.sort_by(|a, b| Self::box_compare(a, b, axis));

                let mid = object_span / 2;
                let left = Arc::new(Self::from_slice(&sorted_objects, 0, mid));
                let right = Arc::new(Self::from_slice(&sorted_objects, mid, sorted_objects.len()));

                Self { left, right, bbox }
            }
        }
    }

    /// 按指定轴比较两个可命中对象的边界盒
    #[inline]
    fn box_compare(a: &Arc<dyn Hittable>, b: &Arc<dyn Hittable>, axis: usize) -> Ordering {
        let a_box = a.bounding_box().unwrap_or_default();
        let b_box = b.bounding_box().unwrap_or_default();

        let a_min = a_box.axis_interval(axis).min;
        let b_min = b_box.axis_interval(axis).min;

        a_min.partial_cmp(&b_min).unwrap_or(Ordering::Equal)
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 首先检查光线是否与包围盒相交
        if !self.bbox.hit(r, ray_t) {
            return false;
        }

        // 检查左子树
        let hit_left = self.left.hit(r, ray_t, rec);

        // 检查右子树，如果左子树命中则限制最大距离
        let right_interval = if hit_left {
            Interval::new(ray_t.min, rec.t)
        } else {
            ray_t
        };
        let hit_right = self.right.hit(r, right_interval, rec);

        hit_left || hit_right
    }

    #[inline]
    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }

    #[inline]
    fn pdf_value(&self, origin: &Point3, direction: &Vec3) -> f64 {
        // BVH节点的PDF值是左右子树的平均
        0.5 * (self.left.pdf_value(origin, direction) + self.right.pdf_value(origin, direction))
    }

    #[inline]
    fn random(&self, origin: &Point3) -> Vec3 {
        // 随机选择左右子树
        use super::super::utils::util::random_double;
        if random_double() < 0.5 {
            self.left.random(origin)
        } else {
            self.right.random(origin)
        }
    }
}

impl std::fmt::Debug for BvhNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BvhNode")
            .field("left", &"<Hittable>")
            .field("right", &"<Hittable>")
            .field("bbox", &self.bbox)
            .finish()
    }
}
