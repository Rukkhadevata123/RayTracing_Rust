use super::interval::Interval;
use super::vec3::Color;
use image::{Rgb, RgbImage};

/// 线性值转换为伽马空间（伽马值为2）
pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

/// 将颜色转换为Rgb像素值，可选采样次数
pub fn color_to_rgb(pixel_color: &Color, samples_per_pixel: Option<i32>) -> Rgb<u8> {
    let mut r = pixel_color.x;
    let mut g = pixel_color.y;
    let mut b = pixel_color.z;

    // 根据采样次数计算实际颜色（如果提供）
    if let Some(samples) = samples_per_pixel {
        let scale = 1.0 / f64::from(samples);
        r *= scale;
        g *= scale;
        b *= scale;
    }

    // 应用线性到伽马空间的转换
    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    // 将[0,1]的颜色值映射到[0,255]范围
    let intensity = Interval::new(0.000, 0.999);
    let r_byte = (256.0 * intensity.clamp(r)) as u8;
    let g_byte = (256.0 * intensity.clamp(g)) as u8;
    let b_byte = (256.0 * intensity.clamp(b)) as u8;

    Rgb([r_byte, g_byte, b_byte])
}

/// 将颜色转换为Rgb像素值，考虑采样次数
pub fn color_to_rgb_with_samples(pixel_color: &Color, samples_per_pixel: i32) -> Rgb<u8> {
    color_to_rgb(pixel_color, Some(samples_per_pixel))
}

/// 保存图像为PNG文件
pub fn save_image(img: RgbImage, filename: &str) -> Result<(), image::ImageError> {
    img.save(filename)
}
