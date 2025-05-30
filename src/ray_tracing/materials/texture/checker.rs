use super::super::super::math::vec3::{Color, Point3};
use super::Texture;
use super::solid_color::SolidColor;
use std::sync::Arc;

/// 棋盘格纹理
#[derive(Debug)]
pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    /// 从两个纹理创建棋盘格纹理
    #[inline]
    pub fn new(scale: f64, even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }

    /// 从两个颜色创建棋盘格纹理
    #[inline]
    pub fn new_colors(scale: f64, c1: Color, c2: Color) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even: Arc::new(SolidColor::new(c1)),
            odd: Arc::new(SolidColor::new(c2)),
        }
    }

    /// 从RGB值创建棋盘格纹理
    #[inline]
    pub fn new_rgb(scale: f64, r1: f64, g1: f64, b1: f64, r2: f64, g2: f64, b2: f64) -> Self {
        Self::new_colors(scale, Color::new(r1, g1, b1), Color::new(r2, g2, b2))
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        // 计算3D空间中的整数坐标
        let x_integer = (self.inv_scale * p.x).floor() as i32;
        let y_integer = (self.inv_scale * p.y).floor() as i32;
        let z_integer = (self.inv_scale * p.z).floor() as i32;

        // 通过坐标和的奇偶性决定使用哪种纹理
        let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}
