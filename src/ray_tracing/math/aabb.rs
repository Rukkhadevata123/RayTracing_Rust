use super::interval::Interval;
use super::ray::Ray;
use super::vec3::*;
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    /// 从三个区间创建 AABB
    #[inline]
    pub const fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    /// 从两个点创建 AABB
    #[inline]
    pub fn new_point(a: Point3, b: Point3) -> Self {
        let mut aabb = Self {
            x: if a.x <= b.x {
                Interval::new(a.x, b.x)
            } else {
                Interval::new(b.x, a.x)
            },
            y: if a.y <= b.y {
                Interval::new(a.y, b.y)
            } else {
                Interval::new(b.y, a.y)
            },
            z: if a.z <= b.z {
                Interval::new(a.z, b.z)
            } else {
                Interval::new(b.z, a.z)
            },
        };
        aabb.pad_to_minimums();
        aabb
    }

    /// 空的 AABB
    #[inline]
    pub const fn empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    /// 宇宙 AABB
    #[inline]
    pub const fn universe() -> Self {
        Self {
            x: Interval::universe(),
            y: Interval::universe(),
            z: Interval::universe(),
        }
    }

    /// 获取指定轴的区间
    #[inline]
    pub fn axis_interval(&self, axis: usize) -> Interval {
        match axis {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Invalid axis index: {}", axis),
        }
    }

    /// 返回最长轴的索引
    #[inline]
    pub fn longest_axis(&self) -> usize {
        let x_size = self.x.size();
        let y_size = self.y.size();
        let z_size = self.z.size();

        if x_size >= y_size && x_size >= z_size {
            0
        } else if y_size >= z_size {
            1
        } else {
            2
        }
    }

    /// 确保最小尺寸
    pub fn pad_to_minimums(&mut self) {
        const DELTA: f64 = 0.0001;
        if self.x.size() < DELTA {
            self.x = self.x.expand(DELTA);
        }
        if self.y.size() < DELTA {
            self.y = self.y.expand(DELTA);
        }
        if self.z.size() < DELTA {
            self.z = self.z.expand(DELTA);
        }
    }

    /// 检查是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.x.is_empty() || self.y.is_empty() || self.z.is_empty()
    }

    /// 合并两个 AABB
    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            x: self.x.merge(&other.x),
            y: self.y.merge(&other.y),
            z: self.z.merge(&other.z),
        }
    }

    /// 扩展 AABB
    #[inline]
    pub fn expand(&self, delta: f64) -> Self {
        Self {
            x: self.x.expand(delta),
            y: self.y.expand(delta),
            z: self.z.expand(delta),
        }
    }

    /// 光线与 AABB 相交测试
    pub fn hit(&self, ray: &Ray, mut ray_t: Interval) -> bool {
        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let ray_dir_component = match axis {
                0 => ray.dir.x,
                1 => ray.dir.y,
                2 => ray.dir.z,
                _ => unreachable!(),
            };

            if ray_dir_component.abs() < 1e-8 {
                continue;
            }

            let adinv = 1.0 / ray_dir_component;
            let ray_orig_component = match axis {
                0 => ray.orig.x,
                1 => ray.orig.y,
                2 => ray.orig.z,
                _ => unreachable!(),
            };

            let t0 = (ax.min - ray_orig_component) * adinv;
            let t1 = (ax.max - ray_orig_component) * adinv;

            let (t_min, t_max) = if adinv >= 0.0 { (t0, t1) } else { (t1, t0) };

            ray_t.min = ray_t.min.max(t_min);
            ray_t.max = ray_t.max.min(t_max);

            if ray_t.max <= ray_t.min {
                return false;
            }
        }
        true
    }
}

impl Default for Aabb {
    fn default() -> Self {
        Self::empty()
    }
}

// 平移操作
impl Add<Vec3> for Aabb {
    type Output = Self;

    #[inline]
    fn add(self, offset: Vec3) -> Self::Output {
        Self {
            x: Interval::new(self.x.min + offset.x, self.x.max + offset.x),
            y: Interval::new(self.y.min + offset.y, self.y.max + offset.y),
            z: Interval::new(self.z.min + offset.z, self.z.max + offset.z),
        }
    }
}

impl Add<Aabb> for Vec3 {
    type Output = Aabb;

    #[inline]
    fn add(self, aabb: Aabb) -> Self::Output {
        aabb + self
    }
}
