use super::super::math::vec3::{Point3, Vec3, Vec3Ext};
use super::super::utils::util::random_int_range;

/// Perlin噪声生成器，用于程序化纹理
#[derive(Debug)]
pub struct Perlin {
    ranvec: Vec<Vec3>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

impl Perlin {
    /// 创建新的Perlin噪声生成器
    #[inline]
    pub fn new() -> Self {
        const POINT_COUNT: usize = 256;

        // 生成随机梯度向量
        let mut ranvec = Vec::with_capacity(POINT_COUNT);
        for _ in 0..POINT_COUNT {
            ranvec.push(Vec3::random_range(-1.0, 1.0).normalize());
        }

        Self {
            ranvec,
            perm_x: Self::generate_perm(POINT_COUNT),
            perm_y: Self::generate_perm(POINT_COUNT),
            perm_z: Self::generate_perm(POINT_COUNT),
        }
    }

    /// 计算点p处的噪声值
    #[inline]
    pub fn noise(&self, p: &Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        // 获取8个相邻格点的梯度向量
        let mut c = [[[Vec3::zeros(); 2]; 2]; 2];
        #[allow(clippy::needless_range_loop)]
        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let idx = self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize];

                    c[di][dj][dk] = self.ranvec[idx as usize];
                }
            }
        }

        Self::perlin_interp(&c, u, v, w)
    }

    /// 湍流函数，多个频率的噪声叠加
    #[inline]
    pub fn turb(&self, p: &Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p *= 2.0;
        }

        accum.abs()
    }

    /// 生成置换表
    #[inline]
    fn generate_perm(point_count: usize) -> Vec<i32> {
        let mut p: Vec<i32> = (0..point_count as i32).collect();
        Self::permute(&mut p);
        p
    }

    /// 随机置换数组
    fn permute(p: &mut [i32]) {
        let n = p.len();
        for i in (1..n).rev() {
            let target = random_int_range(0, i as i32) as usize;
            p.swap(i, target);
        }
    }

    /// Perlin插值，使用三线性插值和Hermite平滑
    fn perlin_interp(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        // Hermite平滑函数，提供更好的过渡
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;
        #[allow(clippy::needless_range_loop)]
        // 三线性插值
        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let i_f = i as f64;
                    let j_f = j as f64;
                    let k_f = k as f64;

                    // 权重向量（从当前点到格点的向量）
                    let weight_v = Vec3::new(u - i_f, v - j_f, w - k_f);

                    // 插值权重
                    let weight = (i_f * uu + (1.0 - i_f) * (1.0 - uu))
                        * (j_f * vv + (1.0 - j_f) * (1.0 - vv))
                        * (k_f * ww + (1.0 - k_f) * (1.0 - ww));

                    // 梯度噪声：权重向量与梯度向量的点积
                    accum += weight * c[i][j][k].dot(&weight_v);
                }
            }
        }

        accum
    }
}

impl Default for Perlin {
    fn default() -> Self {
        Self::new()
    }
}
