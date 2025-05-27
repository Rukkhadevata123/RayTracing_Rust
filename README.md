# Ray Tracing: The Rest of Your Life - Rust实现

一个基于《Ray Tracing: The Rest of Your Life》的现代Rust光线追踪渲染器实现，专注于高级渲染技术和性能优化。

## 🚀 特性

- **蒙特卡洛路径追踪**：高质量的全局光照模拟
- **重要性采样**：智能的光源采样策略提升收敛速度
- **概率密度函数（PDF）系统**：支持多种采样分布
- **体积渲染**：烟雾、云朵等体积散射效果
- **BVH加速结构**：高效的光线-场景求交加速
- **程序化纹理**：Perlin噪声和棋盘格纹理
- **并行渲染**：基于Rayon的高性能并行计算
- **现代Rust特性**：类型安全、内存安全、零成本抽象

## 🏗️ 项目结构

```
src/
├── main.rs                                    # 主程序入口
├── lib.rs                                     # 库入口和文档
├── scenes/                                    # 场景定义
│   ├── mod.rs
│   ├── cornell_box.rs                         # 康奈尔盒场景
│   └── final_scene.rs                         # 复杂最终场景
└── ray_tracing/                              # 核心光线追踪库
    ├── mod.rs                                # 根模块
    ├── math/                                 # 数学基础模块
    │   ├── mod.rs
    │   ├── vec3.rs                          # nalgebra向量包装和扩展
    │   ├── ray.rs                           # 光线表示
    │   ├── interval.rs                      # 数值区间
    │   ├── aabb.rs                          # 轴对齐包围盒
    │   └── onb.rs                           # 正交基
    ├── materials/                            # 材质系统
    │   ├── mod.rs
    │   ├── material.rs                      # 材质trait和散射记录
    │   ├── lambertian.rs                    # 漫反射材质
    │   ├── metal.rs                         # 金属材质
    │   ├── dielectric.rs                    # 电介质材质（玻璃）
    │   ├── diffuse_light.rs                 # 发光材质
    │   ├── isotropic.rs                     # 各向同性散射材质
    │   └── texture/                         # 纹理子系统
    │       ├── mod.rs                       # 纹理trait定义
    │       ├── solid_color.rs               # 纯色纹理
    │       ├── checker.rs                   # 棋盘格纹理
    │       ├── image.rs                     # 图像纹理
    │       └── noise.rs                     # 噪声纹理
    ├── geometry/                             # 几何体系统
    │   ├── mod.rs
    │   ├── hittable.rs                      # 可命中对象trait
    │   ├── sphere.rs                        # 球体（支持运动模糊）
    │   ├── quad.rs                          # 四边形和盒子
    │   ├── hittable_list.rs                 # 几何体列表
    │   └── transforms/                      # 几何变换
    │       ├── mod.rs
    │       ├── translate.rs                 # 平移变换
    │       └── rotate_y.rs                  # Y轴旋转变换
    ├── acceleration/                         # 加速结构
    │   ├── mod.rs
    │   └── bvh.rs                           # 包围体层次结构
    ├── volumes/                              # 体积渲染
    │   ├── mod.rs
    │   └── constant_medium.rs               # 常密度介质
    ├── sampling/                             # 采样系统
    │   ├── mod.rs
    │   └── pdf/                             # 概率密度函数
    │       ├── mod.rs                       # PDF trait定义
    │       ├── cosine_pdf.rs                # 余弦分布
    │       ├── sphere_pdf.rs                # 球面均匀分布
    │       ├── hittable_pdf.rs              # 几何体采样
    │       └── mixture_pdf.rs               # 混合分布
    ├── rendering/                            # 渲染管线
    │   ├── mod.rs
    │   ├── camera.rs                        # 相机系统和渲染器
    │   └── color.rs                         # 颜色处理和伽马校正
    ├── procedural/                           # 程序化生成
    │   ├── mod.rs
    │   └── noise.rs                         # Perlin噪声生成器
    └── utils/                                # 工具函数
        ├── mod.rs
        ├── util.rs                          # 通用工具函数
        └── random.rs                        # 随机数生成
```

## 🛠️ 技术栈

- **Rust 1.70+**：现代系统编程语言
- **nalgebra**：高性能线性代数库
- **image**：图像处理和I/O
- **rayon**：数据并行框架
- **indicatif**：进度条显示

## 🚀 快速开始

### 环境要求

- Rust 1.70 或更高版本
- Cargo 包管理器

### 安装运行

```bash
# 克隆仓库
git clone https://github.com/Rukkhadevata123/RayTracing_Rust.git
cd RayTracing_Rust

# 构建项目
cargo build --release

# 运行不同场景
cargo run --release cornell    # 康奈尔盒场景
cargo run --release final      # 最终复杂场景
cargo run --release quick      # 快速测试版本
```

### 自定义参数

```rust
// 在 scenes/cornell_box.rs 中修改配置
let config = CornellBoxConfig {
    image_width: 800,
    samples_per_pixel: 5000,
    max_depth: 50,
    output_filename: "my_render.png".to_string(),
};
```

## 🎨 渲染示例

### 康奈尔盒 + 玻璃球

![康奈尔盒](./images/output_cornell_box_glass.png)

高质量的康奈尔盒场景，展示了：

- 漫反射表面的柔和阴影
- 玻璃球的折射和反射效果
- 面光源的软阴影
- 颜色溢出效果

### 复杂最终场景

![最终场景](./images/output_final_scene_800x800_5000spp.png)

包含多种材质和效果的复杂场景：

- 程序化噪声纹理
- 体积散射（烟雾效果）
- 运动模糊
- 多种材质的混合

## 🔬 核心技术实现

### 蒙特卡洛路径追踪

```rust
// 路径追踪核心循环
fn ray_color(&self, r: &Ray, depth: i32, world: &dyn Hittable, lights: Option<&Arc<dyn Hittable>>) -> Color {
    if depth <= 0 { return Color::zeros(); }
    
    // 光线与场景求交
    if !world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
        return self.background;
    }
    
    // 材质散射 + 重要性采样
    let scattered_direction = importance_sample(&rec, lights);
    let pdf_value = calculate_pdf(&scattered_direction);
    
    // 递归追踪 + 俄罗斯轮盘赌优化
    emission + (attenuation * scattering_pdf * self.ray_color(&scattered, depth - 1, world, lights)) / pdf_value
}
```

### 重要性采样系统

```rust
// 混合PDF：光源采样 + BRDF采样
let light_pdf = Arc::new(HittablePDF::new(light_objects, &rec.p));
let mixture_pdf = MixturePDF::new(light_pdf, material_pdf);

let direction = mixture_pdf.generate();  // 智能采样方向
let pdf_value = mixture_pdf.value(&direction);  // PDF值
```

### BVH加速结构

```rust
// 层次包围盒加速光线求交
impl Hittable for BvhNode {
    fn hit(&self, r: &Ray, ray_t: Interval, rec: &mut HitRecord) -> bool {
        if !self.bbox.hit(r, ray_t) { return false; }
        
        let hit_left = self.left.hit(r, ray_t, rec);
        let right_interval = if hit_left { 
            Interval::new(ray_t.min, rec.t) 
        } else { 
            ray_t 
        };
        let hit_right = self.right.hit(r, right_interval, rec);
        
        hit_left || hit_right
    }
}
```

## 📊 性能优化

### 并行渲染

- **Rayon数据并行**：每个像素行并行处理
- **SIMD向量化**：nalgebra自动向量化
- **内存局部性**：优化的数据结构布局

### 算法优化

- **俄罗斯轮盘赌**：自适应路径终止
- **重要性采样**：减少方差，提升收敛速度
- **BVH加速**：O(log n)的求交复杂度
- **分层采样**：减少像素内的采样方差

### 典型性能数据

| 场景 | 分辨率 | 采样数 | 渲染时间 | 配置 |
|------|--------|--------|----------|------|
| 康奈尔盒 | 600×600 | 1000 | ~5分钟 | 8核CPU |
| 最终场景 | 800×800 | 5000 | ~45分钟 | 8核CPU |
| 快速测试 | 400×400 | 100 | ~30秒 | 8核CPU |

## 🧮 数学基础

### 渲染方程

```
L_o(p,ω_o) = L_e(p,ω_o) + ∫_Ω f_r(p,ω_i,ω_o) L_i(p,ω_i) (n·ω_i) dω_i
```

其中：

- `L_o`：出射辐射度
- `L_e`：自发光
- `f_r`：BRDF
- `L_i`：入射辐射度

### 蒙特卡洛估计

```
⟨F⟩ ≈ (1/N) Σ f(X_i)/p(X_i)
```

通过重要性采样减少估计方差。

## 🔧 代码架构特点

### Rust特有设计

- **零成本抽象**：trait对象的高效分发
- **所有权系统**：内存安全的并行计算
- **类型安全**：编译时错误检查
- **函数式特性**：map/reduce并行模式

### 设计模式

- **策略模式**：Material和PDF trait
- **装饰器模式**：几何变换系统
- **建造者模式**：场景构建
- **工厂模式**：纹理和材质创建

## 📚 学习路径

### 初学者

1. 理解Vec3和Ray的基本操作
2. 学习球体求交算法
3. 实现简单的漫反射材质

### 进阶

1. 掌握蒙特卡洛方法
2. 理解PDF和重要性采样
3. 实现自定义材质和几何体

### 高级

1. 优化BVH构建算法
2. 实现双向路径追踪
3. 添加GPU计算支持

## 🤝 贡献指南

欢迎提交Issue和Pull Request！

### 开发环境

```bash
# 格式化代码
cargo fmt

# 检查代码
cargo clippy

# 运行测试
cargo test

# 生成文档
cargo doc --open
```

### 代码风格

- 遵循Rust官方代码规范
- 使用有意义的变量名
- 添加适当的文档注释
- 保持函数简洁

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- Peter Shirley的《Ray Tracing》系列书籍
- Rust社区的优秀库生态
- 所有贡献者的支持

## 📚 参考资料

- [Ray Tracing in One Weekend](https://raytracing.github.io/books/RayTracingInOneWeekend.html)
- [Ray Tracing: The Next Week](https://raytracing.github.io/books/RayTracingTheNextWeek.html)
- [Ray Tracing: The Rest of Your Life](https://raytracing.github.io/books/RayTracingTheRestOfYourLife.html)
- [Physically Based Rendering](http://www.pbr-book.org/)
- [nalgebra文档](https://docs.rs/nalgebra/)
- [Rayon文档](https://docs.rs/rayon/)

---

**享受光线追踪的魅力！** ✨
