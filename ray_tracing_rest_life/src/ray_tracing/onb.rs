use super::vec3::{Vec3, cross, unit_vector};

/// 正交基底 (Orthonormal Basis)
///
/// 用于在世界坐标系和局部坐标系之间进行转换
pub struct ONB {
    axis: [Vec3; 3],
}

impl ONB {
    /// 从法线向量创建一个新的正交基底
    pub fn new(n: &Vec3) -> Self {
        let mut axis = [Vec3::zeros(); 3];

        // 将法线设置为第三个基向量(w轴)
        axis[2] = unit_vector(n);

        // 选择一个不与法线平行的向量
        let a = if axis[2].x.abs() > 0.9 {
            Vec3::new(0.0, 1.0, 0.0)
        } else {
            Vec3::new(1.0, 0.0, 0.0)
        };

        // 创建第二个正交基向量(v轴)
        axis[1] = unit_vector(&cross(&axis[2], &a));

        // 创建第一个正交基向量(u轴)
        axis[0] = cross(&axis[2], &axis[1]);

        Self { axis }
    }

    /// 获取u轴 (第一个基向量)
    pub fn u(&self) -> &Vec3 {
        &self.axis[0]
    }

    /// 获取v轴 (第二个基向量)
    pub fn v(&self) -> &Vec3 {
        &self.axis[1]
    }

    /// 获取w轴 (第三个基向量，通常是法线方向)
    pub fn w(&self) -> &Vec3 {
        &self.axis[2]
    }

    /// 将局部坐标系中的向量转换到世界坐标系
    ///
    /// # 参数
    ///
    /// * `a` - 局部坐标系中的向量，其中:
    ///   * a.x 对应u轴方向的分量
    ///   * a.y 对应v轴方向的分量
    ///   * a.z 对应w轴方向的分量
    pub fn local_to_world(&self, a: &Vec3) -> Vec3 {
        a.x * self.axis[0] + a.y * self.axis[1] + a.z * self.axis[2]
    }

    /// 将世界坐标系中的向量转换到局部坐标系
    pub fn world_to_local(&self, a: &Vec3) -> Vec3 {
        Vec3::new(
            a.dot(&self.axis[0]),
            a.dot(&self.axis[1]),
            a.dot(&self.axis[2]),
        )
    }
}
