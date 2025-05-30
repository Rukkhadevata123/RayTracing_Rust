use nalgebra::{Point3 as NalgebraPoint3, Vector3 as NalgebraVector3};

/// 3D向量类型
pub type Vec3 = NalgebraVector3<f64>;
/// 3D点类型
pub type Point3 = NalgebraPoint3<f64>;
/// 颜色类型
pub type Color = Vec3;

/// Vec3扩展trait，添加光线追踪特定功能
pub trait Vec3Ext {
    fn random() -> Self;
    fn random_range(min: f64, max: f64) -> Self;
    fn random_unit_vector() -> Self;
    fn random_in_unit_sphere() -> Self;
    fn random_in_unit_disk() -> Self;
    fn random_cosine_direction() -> Self;
    fn near_zero(&self) -> bool;
    fn reflect(&self, n: &Vec3) -> Vec3;
    fn refract(&self, n: &Vec3, etai_over_etat: f64) -> Vec3;
}

impl Vec3Ext for Vec3 {
    #[inline]
    fn random() -> Self {
        use super::super::utils::util::random_double;
        Self::new(random_double(), random_double(), random_double())
    }

    #[inline]
    fn random_range(min: f64, max: f64) -> Self {
        use super::super::utils::util::random_double_range;
        Self::new(
            random_double_range(min, max),
            random_double_range(min, max),
            random_double_range(min, max),
        )
    }

    fn random_unit_vector() -> Self {
        loop {
            let p = Self::random_range(-1.0, 1.0);
            let len_squared = p.norm_squared();
            if 1e-160 < len_squared && len_squared <= 1.0 {
                return p / len_squared.sqrt();
            }
        }
    }

    fn random_in_unit_sphere() -> Self {
        loop {
            let p = Self::random_range(-1.0, 1.0);
            if p.norm_squared() < 1.0 {
                return p;
            }
        }
    }

    fn random_in_unit_disk() -> Self {
        use super::super::utils::util::random_double_range;
        loop {
            let p = Self::new(
                random_double_range(-1.0, 1.0),
                random_double_range(-1.0, 1.0),
                0.0,
            );
            if p.norm_squared() < 1.0 {
                return p;
            }
        }
    }

    fn random_cosine_direction() -> Self {
        use super::super::utils::util::random_double;
        let r1 = random_double();
        let r2 = random_double();
        let phi = 2.0 * std::f64::consts::PI * r1;

        let x = phi.cos() * r2.sqrt();
        let y = phi.sin() * r2.sqrt();
        let z = (1.0 - r2).sqrt();

        Self::new(x, y, z)
    }

    #[inline]
    fn near_zero(&self) -> bool {
        const S: f64 = 1e-8;
        self.x.abs() < S && self.y.abs() < S && self.z.abs() < S
    }

    #[inline]
    fn reflect(&self, n: &Vec3) -> Vec3 {
        *self - *n * 2.0 * self.dot(n)
    }

    fn refract(&self, n: &Vec3, etai_over_etat: f64) -> Vec3 {
        let cos_theta = f64::min((-*self).dot(n), 1.0);
        let r_out_perp = (*self + *n * cos_theta) * etai_over_etat;
        let r_out_parallel = *n * -(1.0 - r_out_perp.norm_squared()).abs().sqrt();
        r_out_perp + r_out_parallel
    }
}
