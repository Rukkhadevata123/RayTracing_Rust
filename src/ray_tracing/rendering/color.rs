use crate::ray_tracing::math::interval::Interval;
use crate::ray_tracing::math::vec3::Color;
use image::Rgb;

/// 线性颜色值转换为伽马校正值（伽马值为2.0）
#[inline]
pub fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        linear_component.sqrt()
    } else {
        0.0
    }
}

/// 将HDR颜色转换为LDR像素值
pub fn color_to_rgb_with_samples(pixel_color: &Color, samples_per_pixel: i32) -> Rgb<u8> {
    // 处理NaN值
    let mut r = if pixel_color.x.is_nan() {
        0.0
    } else {
        pixel_color.x
    };
    let mut g = if pixel_color.y.is_nan() {
        0.0
    } else {
        pixel_color.y
    };
    let mut b = if pixel_color.z.is_nan() {
        0.0
    } else {
        pixel_color.z
    };

    // 平均化样本
    let scale = 1.0 / samples_per_pixel as f64;
    r *= scale;
    g *= scale;
    b *= scale;

    // 伽马校正
    r = linear_to_gamma(r);
    g = linear_to_gamma(g);
    b = linear_to_gamma(b);

    // 色调映射和量化
    let intensity = Interval::new(0.000, 0.999);
    let r_byte = (256.0 * intensity.clamp(r)) as u8;
    let g_byte = (256.0 * intensity.clamp(g)) as u8;
    let b_byte = (256.0 * intensity.clamp(b)) as u8;

    Rgb([r_byte, g_byte, b_byte])
}

// HSV到RGB转换辅助函数
pub fn hsv_to_rgb(h: f64, s: f64, v: f64) -> (f64, f64, f64) {
    let h = h - h.floor(); // 归一化到[0,1]
    let s = s.clamp(0.0, 1.0);
    let v = v.clamp(0.0, 1.0);

    let hi = (h * 6.0).floor() as i32 % 6;
    let f = h * 6.0 - hi as f64;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);

    match hi {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    }
}
