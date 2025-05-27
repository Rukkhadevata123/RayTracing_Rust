use super::interval::Interval;
use super::ray::Ray;
use super::vec3::{Point3, Vec3};
use std::ops::Add;

#[derive(Debug, Clone, Copy)]
pub struct Aabb {
    pub x: Interval,
    pub y: Interval,
    pub z: Interval,
}

impl Aabb {
    /// 创建一个新的 AABB
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        let mut aabb = Self { x, y, z };
        aabb.pad_to_minimums();
        aabb
    }

    /// 创建一个包含两个点的 AABB
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

    /// 创建一个空的 AABB
    pub fn empty() -> Self {
        Self {
            x: Interval::empty(),
            y: Interval::empty(),
            z: Interval::empty(),
        }
    }

    /// 创建一个包含整个宇宙的 AABB
    pub fn universe() -> Self {
        Self {
            x: Interval::universe(),
            y: Interval::universe(),
            z: Interval::universe(),
        }
    }

    /// 返回当前轴的区间
    pub fn axis_interval(&self, axis: usize) -> Interval {
        match axis {
            0 => self.x,
            1 => self.y,
            2 => self.z,
            _ => panic!("Invalid axis index"),
        }
    }

    /// 返回AABB的最长轴的索引
    pub fn longest_axis(&self) -> usize {
        let x_size = self.x.size();
        let y_size = self.y.size();
        let z_size = self.z.size();

        if x_size > y_size && x_size > z_size {
            0 // x轴最长
        } else if y_size > z_size {
            1 // y轴最长
        } else {
            2 // z轴最长
        }
    }

    /// 确保所有轴的大小不小于最小值
    pub fn pad_to_minimums(&mut self) {
        let delta = 0.0001;
        if self.x.size() < delta {
            self.x = self.x.expand(delta);
        }
        if self.y.size() < delta {
            self.y = self.y.expand(delta);
        }
        if self.z.size() < delta {
            self.z = self.z.expand(delta);
        }
    }

    /// 检查 AABB 是否为空
    pub fn is_empty(&self) -> bool {
        self.x.size() <= 0.0 || self.y.size() <= 0.0 || self.z.size() <= 0.0
    }

    /// 返回包含两个 AABB 的最小 AABB
    pub fn merge(&self, other: &Aabb) -> Self {
        Self {
            x: Interval::new(self.x.min.min(other.x.min), self.x.max.max(other.x.max)),
            y: Interval::new(self.y.min.min(other.y.min), self.y.max.max(other.y.max)),
            z: Interval::new(self.z.min.min(other.z.min), self.z.max.max(other.z.max)),
        }
    }

    /// 对 AABB 进行膨胀
    pub fn expand(&self, delta: f64) -> Self {
        Self {
            x: self.x.expand(delta),
            y: self.y.expand(delta),
            z: self.z.expand(delta),
        }
    }

    /// 检测光线是否与 AABB 相交
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

// 为 Aabb 实现 Default trait，默认为空盒子
impl Default for Aabb {
    fn default() -> Self {
        Self::empty()
    }
}

// 实现盒子平移操作
impl Add<Vec3> for Aabb {
    type Output = Self;

    fn add(self, rhs: Vec3) -> Self::Output {
        Self {
            x: Interval::new(self.x.min + rhs.x, self.x.max + rhs.x),
            y: Interval::new(self.y.min + rhs.y, self.y.max + rhs.y),
            z: Interval::new(self.z.min + rhs.z, self.z.max + rhs.z),
        }
    }
}

impl Add<Aabb> for Vec3 {
    type Output = Aabb;

    fn add(self, rhs: Aabb) -> Self::Output {
        rhs + self
    }
}
