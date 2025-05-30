use super::vec3::Vec3;

/// 正交基底 (Orthonormal Basis)
#[derive(Debug, Clone, Copy)]
#[allow(clippy::upper_case_acronyms)]
pub struct ONB {
    axis: [Vec3; 3],
}

impl ONB {
    /// 从法线向量创建正交基底
    pub fn new(n: &Vec3) -> Self {
        let mut axis = [Vec3::zeros(); 3];

        // w 轴 = 法线方向
        axis[2] = n.normalize();

        // 选择一个不与法线平行的向量
        let a = if axis[2].x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        // v 轴 = w × a
        axis[1] = axis[2].cross(&a).normalize();
        // u 轴 = v × w
        axis[0] = axis[1].cross(&axis[2]);

        Self { axis }
    }

    /// 获取 u 轴（第一个基向量）
    #[inline]
    pub fn u(&self) -> Vec3 {
        self.axis[0]
    }

    /// 获取 v 轴（第二个基向量）
    #[inline]
    pub fn v(&self) -> Vec3 {
        self.axis[1]
    }

    /// 获取 w 轴（第三个基向量，法线方向）
    #[inline]
    pub fn w(&self) -> Vec3 {
        self.axis[2]
    }

    /// 将局部坐标转换到世界坐标
    #[inline]
    pub fn local_to_world(&self, local: &Vec3) -> Vec3 {
        self.axis[0] * local.x + self.axis[1] * local.y + self.axis[2] * local.z
    }

    /// 将世界坐标转换到局部坐标
    #[inline]
    pub fn world_to_local(&self, world: &Vec3) -> Vec3 {
        Vec3::new(
            world.dot(&self.axis[0]),
            world.dot(&self.axis[1]),
            world.dot(&self.axis[2]),
        )
    }
}
