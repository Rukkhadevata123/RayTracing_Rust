use std::sync::Arc;

use super::aabb::Aabb;
use super::hittable::{HitRecord, Hittable};
use super::interval::{Interval, UNIVERSE};
use super::material::{Isotropic, Material};
use super::ray::Ray;
use super::texture::TexturePtr;
use super::util::random_double;
use super::vec3::{Color, Vec3};

pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    phase_function: Arc<dyn Material + Send + Sync>,
    neg_inv_density: f64,
}

impl ConstantMedium {
    pub fn new(boundary: Arc<dyn Hittable>, density: f64, texture: TexturePtr) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::new(texture)),
        }
    }

    pub fn new_color(boundary: Arc<dyn Hittable>, density: f64, color: Color) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::new_color(color)),
        }
    }
}

impl Hittable for ConstantMedium {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        // 打印调试信息，但非常少见
        let enable_debug = false;
        let debugging = enable_debug && random_double() < 0.00001;

        let mut rec1 = HitRecord::default();
        let mut rec2 = HitRecord::default();

        // 检查光线是否击中边界
        if !self.boundary.hit(r, UNIVERSE, &mut rec1) {
            return false;
        }

        // 从第一个交点稍远处继续检查，查找第二个交点（光线离开边界的点）
        if !self
            .boundary
            .hit(r, Interval::new(rec1.t + 0.0001, f64::INFINITY), &mut rec2)
        {
            return false;
        }

        if debugging {
            println!("\nt_min={}, t_max={}", rec1.t, rec2.t);
        }

        // 确保交点在有效区间内
        if rec1.t < ray_t.min {
            rec1.t = ray_t.min;
        }

        if rec2.t > ray_t.max {
            rec2.t = ray_t.max;
        }

        // 如果交点无效，返回失败
        if rec1.t >= rec2.t {
            return false;
        }

        // 确保交点在光线前方
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        // 计算光线在介质中传播的距离
        let ray_length = r.dir.length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;

        // 基于介质密度随机确定散射点
        let hit_distance = self.neg_inv_density * random_double().ln();

        // 如果散射距离超出介质范围，则没有交点
        if hit_distance > distance_inside_boundary {
            return false;
        }

        // 计算散射点位置和时间
        rec.t = rec1.t + hit_distance / ray_length;
        rec.p = r.at(rec.t);

        if debugging {
            println!("hit_distance = {}", hit_distance);
            println!("rec.t = {}", rec.t);
            println!("rec.p = {:?}", rec.p);
        }

        // 设置法线为任意方向（因为介质是体积散射，法线不重要）
        rec.normal = Vec3::new(1.0, 0.0, 0.0); // 任意方向
        rec.front_face = true; // 同样是任意设置

        // 设置材质为相位函数（各向同性散射）
        rec.mat = self.phase_function.clone();

        true
    }

    fn bounding_box(&self) -> Option<Aabb> {
        self.boundary.bounding_box()
    }
}
