pub mod checker;
pub mod image;
pub mod noise;
pub mod solid_color;

use super::super::math::vec3::{Color, Point3};
use std::sync::Arc;

/// 纹理trait - 定义纹理的基本接口
pub trait Texture: Send + Sync + std::fmt::Debug {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

/// 纹理指针类型别名
pub type TexturePtr = Arc<dyn Texture>;

// 重新导出所有纹理类型
pub use checker::CheckerTexture;
pub use image::ImageTexture;
pub use noise::NoiseTexture;
pub use solid_color::SolidColor;
