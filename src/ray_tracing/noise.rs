use super::util::random_int_range;
use super::vec3::{Point3, Vec3, dot};

pub struct Perlin {
    ranvec: Vec<Vec3>,
    perm_x: Vec<i32>,
    perm_y: Vec<i32>,
    perm_z: Vec<i32>,
}

impl Perlin {
    pub fn new() -> Self {
        let point_count = 256;

        let mut ranvec = Vec::with_capacity(point_count);
        for _ in 0..point_count {
            ranvec.push(Vec3::random_range(-1.0, 1.0).unit_vector());
        }

        Self {
            ranvec,
            perm_x: Self::perlin_generate_perm(point_count),
            perm_y: Self::perlin_generate_perm(point_count),
            perm_z: Self::perlin_generate_perm(point_count),
        }
    }

    pub fn noise(&self, p: &Point3) -> f64 {
        let u = p.x - p.x.floor();
        let v = p.y - p.y.floor();
        let w = p.z - p.z.floor();

        let i = p.x.floor() as i32;
        let j = p.y.floor() as i32;
        let k = p.z.floor() as i32;

        let mut c = [[[Vec3::zeros(); 2]; 2]; 2];

        for di in 0..2 {
            for dj in 0..2 {
                for dk in 0..2 {
                    let idx_x = (i + di as i32) & 255;
                    let idx_y = (j + dj as i32) & 255;
                    let idx_z = (k + dk as i32) & 255;

                    let idx = self.perm_x[idx_x as usize]
                        ^ self.perm_y[idx_y as usize]
                        ^ self.perm_z[idx_z as usize];

                    c[di][dj][dk] = self.ranvec[idx as usize];
                }
            }
        }

        Self::perlin_interp(&c, u, v, w)
    }

    pub fn turb(&self, p: &Point3, depth: i32) -> f64 {
        let mut accum = 0.0;
        let mut temp_p = *p;
        let mut weight = 1.0;

        for _ in 0..depth {
            accum += weight * self.noise(&temp_p);
            weight *= 0.5;
            temp_p = temp_p * 2.0;
        }

        accum.abs()
    }

    fn perlin_generate_perm(point_count: usize) -> Vec<i32> {
        let mut p = Vec::with_capacity(point_count);

        for i in 0..point_count {
            p.push(i as i32);
        }

        Self::permute(&mut p, point_count);

        p
    }

    fn permute(p: &mut Vec<i32>, n: usize) {
        for i in (1..n).rev() {
            let target = random_int_range(0, i as i32) as usize;
            p.swap(i, target);
        }
    }

    fn perlin_interp(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        // Hermitian 平滑
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);

        let mut accum = 0.0;

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let i_f = i as f64;
                    let j_f = j as f64;
                    let k_f = k as f64;

                    let weight_v = Vec3::new(u - i_f, v - j_f, w - k_f);

                    accum += (i_f * uu + (1.0 - i_f) * (1.0 - uu))
                        * (j_f * vv + (1.0 - j_f) * (1.0 - vv))
                        * (k_f * ww + (1.0 - k_f) * (1.0 - ww))
                        * dot(&c[i][j][k], &weight_v);
                }
            }
        }

        accum
    }
}
