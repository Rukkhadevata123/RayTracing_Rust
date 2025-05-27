use super::super::super::math::vec3::{Color, Point3};
use super::super::super::procedural::noise::Perlin;
use super::Texture;

/// 噪声纹理，基于Perlin噪声生成程序化纹理
#[derive(Debug)]
pub struct NoiseTexture {
    noise: Perlin,
    scale: f64,
}

impl NoiseTexture {
    /// 创建新的噪声纹理
    #[inline]
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }

    /// 创建带自定义Perlin噪声的纹理
    #[inline]
    pub fn new_with_noise(noise: Perlin, scale: f64) -> Self {
        Self { noise, scale }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, p: &Point3) -> Color {
        // 使用正弦函数创建大理石纹理效果
        // turb函数添加湍流细节
        let noise_value = 1.0 + (self.scale * p.z + 10.0 * self.noise.turb(p, 7)).sin();
        Color::new(0.5, 0.5, 0.5) * noise_value
    }
}
