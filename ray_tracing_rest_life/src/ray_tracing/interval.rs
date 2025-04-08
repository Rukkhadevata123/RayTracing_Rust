use super::util::clamp;

/// 表示一个浮点数区间 [min, max]
#[derive(Debug, Clone, Copy)]
pub struct Interval {
    pub min: f64,
    pub max: f64,
}

impl Interval {
    /// 创建一个新的区间
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// 返回区间的大小
    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    /// 检查值是否在区间内（包括边界）
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    /// 检查值是否在区间内（不包括边界）
    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    /// 将值限制在区间内
    pub fn clamp(&self, x: f64) -> f64 {
        clamp(x, self)
    }

    pub fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Self {
            min: self.min - padding,
            max: self.max + padding,
        }
    }

    /// 空区间（无效区间）
    pub fn empty() -> Self {
        Self {
            min: f64::INFINITY,
            max: f64::NEG_INFINITY,
        }
    }

    /// 宇宙区间（包含所有值）
    pub fn universe() -> Self {
        Self {
            min: f64::NEG_INFINITY,
            max: f64::INFINITY,
        }
    }
}

/// 预定义的空区间（全局常量）
pub const EMPTY: Interval = Interval {
    min: f64::INFINITY,
    max: f64::NEG_INFINITY,
};

/// 预定义的宇宙区间（全局常量）
pub const UNIVERSE: Interval = Interval {
    min: f64::NEG_INFINITY,
    max: f64::INFINITY,
};
