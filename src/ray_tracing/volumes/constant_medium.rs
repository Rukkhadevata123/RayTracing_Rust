use super::super::geometry::hittable::{HitRecord, Hittable};
use super::super::materials::{isotropic::Isotropic, texture::TexturePtr};
use super::super::math::{aabb::Aabb, interval::Interval, ray::Ray, vec3::*};
use super::super::utils::util::random_double;
use std::sync::Arc;

/// 常密度介质，用于体积散射效果（如烟雾、云朵等）
pub struct ConstantMedium {
    boundary: Arc<dyn Hittable>,
    phase_function: Arc<dyn super::super::materials::Material>,
    neg_inv_density: f64,
}

impl ConstantMedium {
    /// 从纹理创建常密度介质
    #[inline]
    pub fn new(boundary: Arc<dyn Hittable>, density: f64, texture: TexturePtr) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: Arc::new(Isotropic::new(texture)),
        }
    }

    /// 从颜色创建常密度介质
    #[inline]
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
        // 调试标志，在实际使用中应该设为 false
        const ENABLE_DEBUG: bool = false;
        let debugging = ENABLE_DEBUG && random_double() < 0.00001;

        let mut rec1 = HitRecord::default();
        let mut rec2 = HitRecord::default();

        // 寻找光线进入介质的点
        if !self.boundary.hit(r, Interval::universe(), &mut rec1) {
            return false;
        }

        // 寻找光线离开介质的点
        if !self
            .boundary
            .hit(r, Interval::new(rec1.t + 0.0001, f64::INFINITY), &mut rec2)
        {
            return false;
        }

        if debugging {
            eprintln!("t_min={}, t_max={}", rec1.t, rec2.t);
        }

        // 限制交点在有效区间内
        rec1.t = rec1.t.max(ray_t.min);
        rec2.t = rec2.t.min(ray_t.max);

        if rec1.t >= rec2.t {
            return false;
        }

        rec1.t = rec1.t.max(0.0);

        // 计算光线在介质中的传播距离
        let ray_length = r.dir.norm();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;

        // 根据介质密度随机确定散射点
        let hit_distance = self.neg_inv_density * random_double().ln();

        if hit_distance > distance_inside_boundary {
            return false;
        }

        // 设置散射点信息
        rec.t = rec1.t + hit_distance / ray_length;
        rec.p = r.at(rec.t);

        if debugging {
            eprintln!("hit_distance = {}", hit_distance);
            eprintln!("rec.t = {}", rec.t);
            eprintln!("rec.p = {:?}", rec.p);
        }

        // 设置法线（对体积散射来说法线是任意的）
        rec.normal = Vec3::new(1.0, 0.0, 0.0);
        rec.front_face = true;
        rec.mat = self.phase_function.clone();

        true
    }

    #[inline]
    fn bounding_box(&self) -> Option<Aabb> {
        self.boundary.bounding_box()
    }
}

impl std::fmt::Debug for ConstantMedium {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ConstantMedium")
            .field("boundary", &"<Hittable>")
            .field("phase_function", &"<Material>")
            .field("neg_inv_density", &self.neg_inv_density)
            .finish()
    }
}
