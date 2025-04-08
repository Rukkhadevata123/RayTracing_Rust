use std::cmp::Ordering;
use std::sync::Arc;

use super::aabb::Aabb;
use super::hittable::{HitRecord, Hittable};
use super::hittable_list::HittableList;
use super::interval::Interval;
use super::ray::Ray;

pub struct BvhNode {
    left: Arc<dyn Hittable + Send + Sync>,
    right: Arc<dyn Hittable + Send + Sync>,
    bbox: Aabb,
}

impl BvhNode {
    // 从可命中对象列表构造BVH
    pub fn new(list: &HittableList) -> Self {
        // 调用with_slice版本，传递对象的引用
        Self::with_slice(&list.objects, 0, list.objects.len())
    }

    // 从对象切片的一个子集构造BVH
    fn with_slice(objects: &[Arc<dyn Hittable + Send + Sync>], start: usize, end: usize) -> Self {
        let mut bbox = Aabb::empty();

        // 构建包含所有源对象的边界盒
        for object_index in start..end {
            let object_bbox = objects[object_index]
                .bounding_box()
                .unwrap_or(Aabb::empty());
            bbox = bbox.merge(&object_bbox);
        }

        // 确定对象排序的轴
        let axis = bbox.longest_axis(); // 使用Aabb的longest_axis方法

        // 选择合适的比较函数
        let comparator = match axis {
            0 => box_x_compare,
            1 => box_y_compare,
            _ => box_z_compare,
        };

        let object_span = end - start;

        // 根据对象数量决定如何构建树
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
                // 有两个对象，根据轴进行排序
                if comparator(&objects[start], &objects[start + 1]) == Ordering::Less {
                    Self {
                        left: objects[start].clone(),
                        right: objects[start + 1].clone(),
                        bbox,
                    }
                } else {
                    Self {
                        left: objects[start + 1].clone(),
                        right: objects[start].clone(),
                        bbox,
                    }
                }
            }
            _ => {
                // 有多个对象，排序并递归创建左右子树
                let mut sorted_objects = objects[start..end].to_vec();
                sorted_objects.sort_by(comparator);

                let mid = object_span / 2;

                let left = Arc::new(BvhNode::with_slice(&sorted_objects, 0, mid));
                let right = Arc::new(BvhNode::with_slice(
                    &sorted_objects,
                    mid,
                    sorted_objects.len(),
                ));

                Self { left, right, bbox }
            }
        }
    }
}

impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 如果光线不与边界盒相交，则直接返回false
        if !self.bbox.hit(r, ray_t) {
            return false;
        }

        // 检查左子树是否被击中
        let hit_left = self.left.hit(r, ray_t, rec);

        // 检查右子树是否被击中，如果左子树被击中，则只考虑比左子树更近的交点
        let hit_right = self.right.hit(
            r,
            Interval::new(ray_t.min, if hit_left { rec.t } else { ray_t.max }),
            rec,
        );

        hit_left || hit_right
    }

    fn bounding_box(&self) -> Option<Aabb> {
        Some(self.bbox)
    }
}

// 按x轴比较两个可命中对象的边界盒
fn box_x_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
) -> Ordering {
    box_compare(a, b, 0)
}

// 按y轴比较两个可命中对象的边界盒
fn box_y_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
) -> Ordering {
    box_compare(a, b, 1)
}

// 按z轴比较两个可命中对象的边界盒
fn box_z_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
) -> Ordering {
    box_compare(a, b, 2)
}

// 按指定轴比较两个可命中对象的边界盒
fn box_compare(
    a: &Arc<dyn Hittable + Send + Sync>,
    b: &Arc<dyn Hittable + Send + Sync>,
    axis: usize,
) -> Ordering {
    let a_box = a.bounding_box().unwrap_or(Aabb::default());
    let b_box = b.bounding_box().unwrap_or(Aabb::default());

    let a_min = a_box.axis_interval(axis).min;
    let b_min = b_box.axis_interval(axis).min;

    a_min.partial_cmp(&b_min).unwrap_or(Ordering::Equal)
}
