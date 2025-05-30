/// 表示一个浮点数区间 [min, max]
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    /// 创建一个新的区间
    #[inline]
    pub const fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// 空区间（无效区间）
    #[inline]
    pub const fn empty() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// 宇宙区间（包含所有值）
    #[inline]
    pub const fn universe() -> Self {
        Self {
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
        }
    }

    /// 返回区间的大小
    #[inline]
    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    /// 检查值是否在区间内（包括边界）
    #[inline]
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    /// 检查值是否在区间内（不包括边界）
    #[inline]
    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    /// 将值限制在区间内
    #[inline]
    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            self.min
        } else if x > self.max {
            self.max
        } else {
            x
        }
    }

    /// 扩展区间
    #[inline]
    pub fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self {
            min: self.min - padding,
            max: self.max + padding,
        }
    }

    /// 检查区间是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.min >= self.max
    }

    /// 合并两个区间
    #[inline]
    pub fn merge(&self, other: &Self) -> Self {
        Self {
            min: self.min.min(other.min),
            max: self.max.max(other.max),
        }
    }
}

impl Default for Interval {
    fn default() -> Self {
        Self::empty()
    }
}
