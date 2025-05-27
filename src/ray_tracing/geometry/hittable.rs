use super::super::materials::material::Material;
use super::super::math::{aabb::Aabb, interval::Interval, ray::Ray, vec3::*};
use std::sync::Arc;

/// 命中记录，包含光线与物体交点的所有信息
pub struct HitRecord {
    pub p: Point3,              // 交点位置
    pub normal: Vec3,           // 表面法线
    pub mat: Arc<dyn Material>, // 材质
    pub t: f64,                 // 光线参数t
    pub u: f64,                 // 纹理坐标u
    pub v: f64,                 // 纹理坐标v
    pub front_face: bool,       // 是否为正面
}

impl HitRecord {
    /// 创建新的命中记录
    #[inline]
    pub fn new(
        p: Point3,
        normal: Vec3,
        mat: Arc<dyn Material>,
        t: f64,
        u: f64,
        v: f64,
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

    /// 根据光线方向设置正确的法线方向
    #[inline]
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: &Vec3) {
        self.front_face = r.dir.dot(outward_normal) < 0.0;
        self.normal = if self.front_face {
            *outward_normal
        } else {
            -*outward_normal
        };
    }
}

impl std::fmt::Debug for HitRecord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HitRecord")
            .field("p", &self.p)
            .field("normal", &self.normal)
            .field("mat", &"<Material>")
            .field("t", &self.t)
            .field("u", &self.u)
            .field("v", &self.v)
            .field("front_face", &self.front_face)
            .finish()
    }
}

impl Clone for HitRecord {
    fn clone(&self) -> Self {
        Self {
            p: self.p,
            normal: self.normal,
            mat: self.mat.clone(),
            t: self.t,
            u: self.u,
            v: self.v,
            front_face: self.front_face,
        }
    }
}

impl Default for HitRecord {
    fn default() -> Self {
        Self::new(
            Point3::origin(),
            Vec3::new(0.0, 0.0, 0.0),
            Arc::new(super::super::materials::material::NoMaterial),
            0.0,
            0.0,
            0.0,
            false,
        )
    }
}

/// 可被光线击中的物体trait
pub trait Hittable: Send + Sync + std::fmt::Debug {
    /// 检测光线与物体的交点
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool;

    /// 返回物体的包围盒
    fn bounding_box(&self) -> Option<Aabb> {
        None
    }

    /// 计算从给定点向物体随机采样的概率密度
    fn pdf_value(&self, _origin: &Point3, _direction: &Vec3) -> f64 {
        0.0
    }

    /// 从给定点向物体生成随机方向
    fn random(&self, _origin: &Point3) -> Vec3 {
        Vec3::new(1.0, 0.0, 0.0) // 默认方向
    }
}
