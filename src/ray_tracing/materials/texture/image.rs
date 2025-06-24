use super::Texture;
use crate::ray_tracing::math::vec3::{Color, Point3};
use image::{DynamicImage, GenericImageView};
/// 图像纹理
#[derive(Debug)]
pub struct ImageTexture {
    image: Option<DynamicImage>,
    width: u32,
    height: u32,
}

impl ImageTexture {
    /// 从文件加载图像纹理
    #[inline]
    pub fn new(image_filename: &str) -> Self {
        // 尝试多个可能的路径
        let paths = [
            image_filename,
            &format!("textures/{}", image_filename),
            &format!("../textures/{}", image_filename),
        ];

        for path in &paths {
            if let Ok(img) = image::open(path) {
                return Self::from_image(img);
            }
        }

        // 检查环境变量
        if let Ok(image_dir) = std::env::var("RTW_IMAGES") {
            let full_path = format!("{}/{}", image_dir, image_filename);
            if let Ok(img) = image::open(&full_path) {
                return Self::from_image(img);
            }
        }

        eprintln!("ERROR: Could not load image file '{}'.", image_filename);
        Self {
            image: None,
            width: 0,
            height: 0,
        }
    }

    /// 从图像对象创建纹理
    #[inline]
    fn from_image(img: DynamicImage) -> Self {
        let width = img.width();
        let height = img.height();
        Self {
            image: Some(img),
            width,
            height,
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
        let v_clamped = 1.0 - v.clamp(0.0, 1.0); // 翻转 V 为图像坐标

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
            Color::new(0.0, 1.0, 1.0) // 默认青色
        }
    }
}
