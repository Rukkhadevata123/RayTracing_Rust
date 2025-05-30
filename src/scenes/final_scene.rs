use super::super::ray_tracing::{
    acceleration::BvhNode,
    geometry::{HittableList, Quad, RotateY, Sphere, Translate, box_new},
    materials::texture::{CheckerTexture, ImageTexture, NoiseTexture, SolidColor},
    materials::{Dielectric, DiffuseLight, Lambertian, Metal, NoMaterial},
    math::{
        onb::ONB,
        vec3::{Color, Point3, Vec3, Vec3Ext},
    },
    procedural::noise::Perlin,
    rendering::{Camera, color::hsv_to_rgb},
    utils::util::random_double_range,
    volumes::ConstantMedium,
};
use std::sync::Arc;
use std::time::Instant;

/// 最终场景配置
pub struct FinalSceneConfig {
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub output_filename: String,
}

impl Default for FinalSceneConfig {
    fn default() -> Self {
        Self {
            image_width: 800,
            samples_per_pixel: 5000,
            max_depth: 75,
            output_filename: "final_scene.png".to_string(),
        }
    }
}

/// 构建最终复杂场景
pub fn final_scene_next_week(config: FinalSceneConfig) {
    let mut world = HittableList::new();

    // 地面材质
    let ground = Arc::new(Lambertian::new(Color::new(0.48, 0.83, 0.53)));

    // 创建地面的随机盒子
    let mut boxes1 = HittableList::new();
    const BOXES_PER_SIDE: i32 = 20;

    for i in 0..BOXES_PER_SIDE {
        for j in 0..BOXES_PER_SIDE {
            let w = 100.0;
            let x0 = -1000.0 + i as f64 * w;
            let z0 = -1000.0 + j as f64 * w;
            let y0 = 0.0;
            let x1 = x0 + w;
            let y1 = random_double_range(1.0, 101.0);
            let z1 = z0 + w;

            boxes1.add(Arc::new(box_new(
                Point3::new(x0, y0, z0),
                Point3::new(x1, y1, z1),
                ground.clone(),
            )));
        }
    }

    // 使用BVH加速地面盒子
    world.add(Arc::new(BvhNode::new(&boxes1)));

    // 添加光源
    let light = Arc::new(DiffuseLight::new_color(Color::new(7.0, 7.0, 7.0)));
    world.add(Arc::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        light.clone(),
    )));

    // 添加移动球体
    let center1 = Point3::new(400.0, 400.0, 200.0);
    let center2 = center1 + Vec3::new(30.0, 0.0, 0.0);
    let sphere_material = Arc::new(Lambertian::new(Color::new(0.7, 0.3, 0.1)));
    world.add(Arc::new(Sphere::new_moving(
        center1,
        center2,
        50.0,
        sphere_material,
    )));

    // 玻璃球
    world.add(Arc::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::new(Dielectric::new(1.5)),
    )));

    // 金属球
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 150.0, 145.0),
        50.0,
        Arc::new(Metal::new(Color::new(0.8, 0.8, 0.9), 1.0)),
    )));

    // 蓝色烟雾球
    let boundary = Arc::new(Sphere::new(
        Point3::new(360.0, 150.0, 145.0),
        70.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(boundary.clone());
    world.add(Arc::new(ConstantMedium::new_color(
        boundary,
        0.2,
        Color::new(0.2, 0.4, 0.9),
    )));

    // 环境雾
    let boundary = Arc::new(Sphere::new(
        Point3::origin(),
        5000.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(Arc::new(ConstantMedium::new_color(
        boundary,
        0.0001,
        Color::new(1.0, 1.0, 1.0),
    )));

    // 地球纹理球
    let earth_texture = Arc::new(ImageTexture::new("textures/earthmap.jpg"));
    let earth_material = Arc::new(Lambertian::new_texture(earth_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        earth_material,
    )));

    // 噪声纹理球
    let noise_texture = Arc::new(NoiseTexture::new(0.2));
    world.add(Arc::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Arc::new(Lambertian::new_texture(noise_texture)),
    )));

    // 创建小球群
    let mut boxes2 = HittableList::new();
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    const NS: i32 = 1000;

    for _ in 0..NS {
        boxes2.add(Arc::new(Sphere::new(
            Point3::new(
                Vec3::random_range(0.0, 165.0).x,
                Vec3::random_range(0.0, 165.0).y,
                Vec3::random_range(0.0, 165.0).z,
            ),
            10.0,
            white.clone(),
        )));
    }

    // 小球群的BVH，然后旋转和平移
    let boxes2_node = Arc::new(BvhNode::new(&boxes2));
    let boxes2_rotated = Arc::new(RotateY::new(boxes2_node, 15.0));
    let boxes2_translated = Arc::new(Translate::new(
        boxes2_rotated,
        Vec3::new(-100.0, 270.0, 395.0),
    ));
    world.add(boxes2_translated);

    // 光源列表（用于重要性采样）
    let mut lights = HittableList::new();
    lights.add(Arc::new(Quad::new(
        Point3::new(123.0, 554.0, 147.0),
        Vec3::new(300.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 265.0),
        Arc::new(NoMaterial),
    )));

    // 配置相机
    let mut camera = Camera::new();
    camera.aspect_ratio = 1.0;
    camera.image_width = config.image_width;
    camera.samples_per_pixel = config.samples_per_pixel;
    camera.max_depth = config.max_depth;
    camera.background = Color::zeros(); // 黑色背景

    camera.vfov = 40.0;
    camera.lookfrom = Point3::new(478.0, 278.0, -600.0);
    camera.lookat = Point3::new(278.0, 278.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);
    camera.defocus_angle = 0.0;
    camera.output_filename = config.output_filename;

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染最终场景...");
    eprintln!(
        "图像大小: {}x{}, 采样数: {}, 反射深度: {}",
        config.image_width, config.image_width, config.samples_per_pixel, config.max_depth
    );

    camera.render(&world, Some(Arc::new(lights)));

    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

/// 展示各种纹理效果
pub fn texture_showcase_scene(config: FinalSceneConfig) {
    let mut world = HittableList::new();

    // 使用更精致的棋盘格地面
    let fine_even_texture = Arc::new(SolidColor::new(Color::new(0.1, 0.25, 0.05)));
    let fine_odd_texture = Arc::new(SolidColor::new(Color::new(0.95, 0.95, 0.9)));
    let ground_checker = Arc::new(CheckerTexture::new(
        3.0, // 更小的格子尺寸，提高格子密度
        fine_even_texture,
        fine_odd_texture,
    ));
    let ground = Arc::new(Lambertian::new_texture(ground_checker));

    world.add(Arc::new(box_new(
        Point3::new(-1000.0, -1.0, -1000.0),
        Point3::new(1000.0, 0.0, 1000.0),
        ground,
    )));

    // 增加多个光源，提升照明效果
    // 主光源（白色）
    let main_light_texture = Arc::new(SolidColor::new(Color::new(7.0, 7.0, 7.0)));
    let main_light = Arc::new(DiffuseLight::new(main_light_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 10.0, 0.0),
        2.0,
        main_light,
    )));

    // 辅助光源（暖色调）
    let warm_light_texture = Arc::new(SolidColor::new(Color::new(5.0, 3.0, 1.0)));
    let warm_light = Arc::new(DiffuseLight::new(warm_light_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(-8.0, 6.0, -5.0),
        1.0,
        warm_light,
    )));

    // 辅助光源（冷色调）
    let cool_light_texture = Arc::new(SolidColor::new(Color::new(1.0, 3.0, 5.0)));
    let cool_light = Arc::new(DiffuseLight::new(cool_light_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(8.0, 5.0, -5.0),
        1.0,
        cool_light,
    )));

    // 中央棋盘格球体阵列 - 环形布局
    let sphere_positions = [
        (0.0, 1.0, 0.0),   // 中心
        (-3.5, 1.0, -3.5), // 左后
        (-5.0, 1.0, 0.0),  // 左
        (-3.5, 1.0, 3.5),  // 左前
        (0.0, 1.0, 5.0),   // 前
        (3.5, 1.0, 3.5),   // 右前
        (5.0, 1.0, 0.0),   // 右
        (3.5, 1.0, -3.5),  // 右后
        (0.0, 1.0, -5.0),  // 后
    ];

    // 中央特殊球体 - 超精细棋盘格
    let micro_checker = Arc::new(CheckerTexture::new_rgb(
        0.15, // 极小的格子
        0.9, 0.1, 0.1, // 红色
        0.1, 0.1, 0.9, // 蓝色
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(
            sphere_positions[0].0,
            sphere_positions[0].1,
            sphere_positions[0].2,
        ),
        1.5, // 中心球体更大
        Arc::new(Lambertian::new_texture(micro_checker)),
    )));

    // 环绕的彩色棋盘格球体
    for i in 1..sphere_positions.len() {
        let checker_scale = 0.3 + (i as f64 * 0.15); // 逐渐变化的格子尺寸
        let hue = i as f64 / sphere_positions.len() as f64; // 色调变化

        // 为每个球体创建独特的颜色组合
        let color1 = hsv_to_rgb(hue, 0.9, 0.9);
        let color2 = hsv_to_rgb(hue + 0.5, 0.9, 0.9);

        let checker = Arc::new(CheckerTexture::new_rgb(
            checker_scale,
            color1.0,
            color1.1,
            color1.2,
            color2.0,
            color2.1,
            color2.2,
        ));

        world.add(Arc::new(Sphere::new(
            Point3::new(
                sphere_positions[i].0,
                sphere_positions[i].1,
                sphere_positions[i].2,
            ),
            1.0,
            Arc::new(Lambertian::new_texture(checker)),
        )));
    }

    // 大理石纹理球 - 位置更佳
    let custom_noise = Perlin::new();
    let marble_texture = Arc::new(NoiseTexture::new_with_noise(custom_noise, 2.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 3.5, -4.0), // 放置在高点，更加突出
        2.0,                         // 更大的尺寸，更加醒目
        Arc::new(Lambertian::new_texture(marble_texture)),
    )));

    // 玻璃球与烟雾效果 - 位置更佳
    let boundary_sphere = Arc::new(Sphere::new(
        Point3::new(-4.0, 2.5, -4.0), // 对称放置
        2.0,                          // 更大尺寸
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(boundary_sphere.clone());

    let smoke_noise = Perlin::new();
    let smoke_texture = Arc::new(NoiseTexture::new_with_noise(smoke_noise, 0.5));
    world.add(Arc::new(ConstantMedium::new(
        boundary_sphere,
        0.5, // 更合适的密度
        smoke_texture,
    )));

    // 原点金属球 - 增加反光度
    world.add(Arc::new(Sphere::new(
        Point3::origin(),
        0.5,                                                  // 稍大些
        Arc::new(Metal::new(Color::new(0.9, 0.8, 0.2), 0.0)), // 更亮的金色，完全镜面
    )));

    // ======= 使用ONB创建各种几何结构 =======
    let up_vector = Vec3::new(0.0, 1.0, 0.0);
    let onb = ONB::new(&up_vector);

    // 1. 螺旋上升布局 - 使用local_to_world方法
    let mut decoration_balls = HittableList::new();
    for i in 0..24 {
        // 螺旋参数
        let angle = i as f64 * 0.5; // 螺旋角度
        let radius = 4.0; // 螺旋半径
        let height = i as f64 * 0.2; // 高度增量

        // 螺旋路径
        let local_pos = Vec3::new(radius * angle.cos(), height, radius * angle.sin());

        // 转换到世界坐标
        let world_offset = onb.local_to_world(&local_pos);
        let world_pos = Point3::new(
            world_offset.x + 0.0,
            world_offset.y + 1.0,
            world_offset.z + 0.0,
        );

        // 彩虹色渐变
        let hue = i as f64 / 24.0;
        let (r, g, b) = hsv_to_rgb(hue, 0.9, 0.9);
        let color = Color::new(r, g, b);

        let size = 0.15 + (i % 3) as f64 * 0.05; // 变化的大小
        decoration_balls.add(Arc::new(Sphere::new(
            world_pos,
            size,
            Arc::new(Metal::new(color, 0.0)),
        )));
    }

    // 2. 使用u和v创建网格状阵列 - 展示坐标系方向
    for i in -4..5 {
        for j in -4..5 {
            // 只选取外围部分，形成矩形框架
            if (i > -4 && i < 4) && (j > -4 && j < 4) {
                continue;
            }

            // 使用u和v方法显式获取基向量
            let u_offset = onb.u() * (i as f64 * 0.5); // u方向间距
            let v_offset = onb.v() * (j as f64 * 0.5); // v方向间距

            // 计算位置，固定在地面上方
            let world_pos = Point3::new(
                u_offset.x + 0.0,
                0.5, // 高度固定
                v_offset.z + 0.0,
            );

            // 材质根据u/v方向的梯度变化
            let u_factor = (i as f64 + 4.0) / 8.0;
            let v_factor = (j as f64 + 4.0) / 8.0;
            let color = Color::new(u_factor, 0.5, v_factor);

            decoration_balls.add(Arc::new(Sphere::new(
                world_pos,
                0.2,
                Arc::new(Lambertian::new(color)),
            )));
        }
    }

    // 3. 使用world_to_local创建放射状物体
    let center_point = Point3::new(0.0, 4.0, 0.0);
    for i in 0..16 {
        let angle = i as f64 * std::f64::consts::PI / 8.0;

        // 创建放射状的点
        let ray_end = center_point + Vec3::new(angle.cos() * 2.0, 0.0, angle.sin() * 2.0);

        // 将世界坐标转换为ONB局部坐标
        let local_coords = onb.world_to_local(&(ray_end - center_point));

        // 使用局部坐标计算材质属性 - 从局部坐标的角度决定材质特性
        let roughness = local_coords.y.abs() * 0.5;
        let metal_tint = Color::new(
            (local_coords.x + 1.0) * 0.5,
            (local_coords.y + 1.0) * 0.5,
            (local_coords.z + 1.0) * 0.5,
        );

        decoration_balls.add(Arc::new(Sphere::new(
            ray_end,
            0.15,
            Arc::new(Metal::new(metal_tint, roughness)),
        )));
    }

    eprintln!("创建装饰球数量: {}", decoration_balls.len());
    for ball in &decoration_balls.objects {
        world.add(ball.clone());
    }

    // 配置相机
    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = config.image_width;
    camera.samples_per_pixel = config.samples_per_pixel;
    camera.max_depth = config.max_depth;
    camera.background = Color::new(0.05, 0.05, 0.1); // 更深沉的背景色

    // 更好的相机角度
    camera.vfov = 30.0;
    camera.lookfrom = Point3::new(10.0, 6.0, 10.0);
    camera.lookat = Point3::new(0.0, 1.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);
    camera.defocus_angle = 0.1; // 轻微景深效果
    camera.focus_dist = 12.0;
    camera.output_filename = "texture_showcase.png".to_string();

    let start = Instant::now();
    eprintln!("开始渲染精致纹理展示场景...");
    eprintln!("场景包含 {} 个物体", world.len());

    camera.render(&world, None);

    let duration = start.elapsed();
    eprintln!("精致纹理展示场景渲染完成！总耗时: {:?}", duration);
}
