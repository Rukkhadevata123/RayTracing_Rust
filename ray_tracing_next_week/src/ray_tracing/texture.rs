use std::sync::Arc;
use image::{DynamicImage, GenericImageView};

use super::vec3::{Color, Point3};
use super::noise::Perlin;

pub trait Texture: Send + Sync {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color;
}

// 纯色纹理
pub struct SolidColor {
    color_value: Color,
}

impl SolidColor {
    pub fn new(color: Color) -> Self {
        Self { color_value: color }
    }

    pub fn new_rgb(r: f64, g: f64, b: f64) -> Self {
        Self { color_value: Color::new(r, g, b) }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: &Point3) -> Color {
        self.color_value
    }
}

// 棋盘格纹理
pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture + Send + Sync>,
    odd: Arc<dyn Texture + Send + Sync>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: Arc<dyn Texture + Send + Sync>, odd: Arc<dyn Texture + Send + Sync>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }
    
    pub fn new_colors(scale: f64, c1: Color, c2: Color) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even: Arc::new(SolidColor::new(c1)),
            odd: Arc::new(SolidColor::new(c2)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, u: f64, v: f64, p: &Point3) -> Color {
        let x_integer = (self.inv_scale * p.x).floor() as i32;
        let y_integer = (self.inv_scale * p.y).floor() as i32;
        let z_integer = (self.inv_scale * p.z).floor() as i32;

        let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

        if is_even {
            self.even.value(u, v, p)
        } else {
            self.odd.value(u, v, p)
        }
    }
}

// 图片纹理
pub struct ImageTexture {
    image: Option<DynamicImage>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    pub fn new(filename: &str) -> Self {
        match image::open(filename) {
            Ok(img) => {
                let width = img.width();
                let height = img.height();
                Self {
                    image: Some(img),
                    width,
                    height,
                }
            }
            Err(e) => {
                eprintln!("Error loading image '{}': {}", filename, e);
                Self {
                    image: None,
                    width: 0,
                    height: 0,
                }
            }
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: &Point3) -> Color {
        // 如果没有纹理数据，返回青色作为调试辅助
        if self.height == 0 {
            return Color::new(0.0, 1.0, 1.0);
        }

        // 将输入纹理坐标限制在 [0,1] × [0,1]
        let u_clamped = u.clamp(0.0, 1.0);
        let v_clamped = 1.0 - v.clamp(0.0, 1.0);  // 翻转 V 为图像坐标

        let i = (u_clamped * self.width as f64) as u32;
        let j = (v_clamped * self.height as f64) as u32;

        // 防止越界
        let i = i.min(self.width - 1);
        let j = j.min(self.height - 1);

        if let Some(img) = &self.image {
            let pixel = img.get_pixel(i, j);
            let color_scale = 1.0 / 255.0;
            
            Color::new(
                color_scale * pixel[0] as f64,
                color_scale * pixel[1] as f64,
                color_scale * pixel[2] as f64,
            )
        } else {
            Color::new(0.0, 1.0, 1.0)  // 默认青色
        }
    }
}

// 噪声纹理
pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        Color::new(0.5, 0.5, 0.5) * (1.0 + (self.scale * p.z + 10.0 * self.noise.turb(p, 7)).sin())
    }
}

// 便于使用的辅助类型别名，减少反复写 Arc<dyn Texture + Send + Sync>
pub type TexturePtr = Arc<dyn Texture + Send + Sync>;