pub mod cosine_pdf;
pub mod hittable_pdf;
pub mod mixture_pdf;
pub mod sphere_pdf;

use super::super::math::vec3::Vec3;

/// 概率密度函数trait，用于重要性采样
#[allow(clippy::upper_case_acronyms)]
pub trait PDF: Send + Sync + std::fmt::Debug {
    /// 计算给定方向的概率密度值
    fn value(&self, direction: &Vec3) -> f64;

    /// 根据PDF生成随机方向
    fn generate(&self) -> Vec3;
}

// 重新导出所有PDF类型
pub use cosine_pdf::CosinePDF;
pub use hittable_pdf::HittablePDF;
pub use mixture_pdf::MixturePDF;
pub use sphere_pdf::SpherePDF;
