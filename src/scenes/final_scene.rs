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
    rendering::Camera,
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

/// 创建一个包含各种纹理和材质的精致测试场景
pub fn texture_showcase_scene(config: FinalSceneConfig) {
    let mut world = HittableList::new();

    // 使用精致的棋盘格地面（更小的格子）
    let fine_even_texture = Arc::new(SolidColor::new(Color::new(0.2, 0.3, 0.1)));
    let fine_odd_texture = Arc::new(SolidColor::new(Color::new(0.9, 0.9, 0.9)));
    let ground_checker = Arc::new(CheckerTexture::new(
        10.0, // 更小的格子尺寸，从100.0改为10.0
        fine_even_texture,
        fine_odd_texture,
    ));
    let ground = Arc::new(Lambertian::new_texture(ground_checker));

    world.add(Arc::new(box_new(
        Point3::new(-1000.0, -1.0, -1000.0),
        Point3::new(1000.0, 0.0, 1000.0),
        ground,
    )));

    // 精致的彩色棋盘格球体（更小的格子）
    let colored_checker = Arc::new(CheckerTexture::new_rgb(
        1.5, // 更精细的格子
        0.8, 0.2, 0.3, // 红色
        0.2, 0.8, 0.3, // 绿色
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::new_texture(colored_checker)),
    )));

    // 超精细的RGB纹理球体
    let rgb_checker = Arc::new(CheckerTexture::new_rgb(
        0.8, // 非常细的格子
        0.9, 0.1, 0.1, // 亮红色
        0.1, 0.1, 0.9, // 亮蓝色
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(-2.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::new_texture(rgb_checker)),
    )));

    // 使用SolidColor的RGB构造函数
    let solid_color = Arc::new(SolidColor::new_rgb(0.8, 0.3, 0.9));
    world.add(Arc::new(Sphere::new(
        Point3::new(2.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::new_texture(solid_color)),
    )));

    // 使用自定义Perlin噪声的大理石纹理球
    let custom_noise = Perlin::new();
    let marble_texture = Arc::new(NoiseTexture::new_with_noise(custom_noise, 2.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::new_texture(marble_texture)),
    )));

    // 各向同性材质球体（烟雾效果）- 使用自定义纹理
    let boundary_sphere = Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(boundary_sphere.clone());

    // 创建自定义噪声纹理用于烟雾
    let smoke_noise = Perlin::new();
    let smoke_texture = Arc::new(NoiseTexture::new_with_noise(smoke_noise, 0.5));
    world.add(Arc::new(ConstantMedium::new(
        boundary_sphere,
        1.5,
        smoke_texture,
    )));

    // 使用ONB坐标系统创建装饰性小球
    let up_vector = Vec3::new(0.0, 1.0, 0.0);
    let onb = ONB::new(&up_vector);

    // 创建装饰球列表，展示len和clear方法
    let mut decoration_balls = HittableList::new();

    for i in 0..8 {
        // 在局部坐标系中生成位置
        let local_pos = Vec3::new(i as f64 * 0.3 - 1.2, 0.0, 0.0);

        // 直接使用local_to_world函数转换到世界坐标系
        let world_offset = onb.local_to_world(&local_pos);
        let world_pos = Point3::new(
            world_offset.x + 0.0,
            world_offset.y + 0.2,
            world_offset.z + 3.0,
        );

        // 创建随机向量，使用Vec3Ext::random和near_zero
        let random_color = Vec3::random();
        let final_color = if random_color.near_zero() {
            Color::new(0.5, 0.5, 0.5) // 如果接近零向量，使用默认颜色
        } else {
            Color::new(
                random_color.x.abs(),
                random_color.y.abs(),
                random_color.z.abs(),
            )
        };

        decoration_balls.add(Arc::new(Sphere::new(
            world_pos,
            0.08,
            Arc::new(Metal::new(final_color, 0.0)),
        )));
    }

    // 显示装饰球数量
    eprintln!("创建装饰球数量: {}", decoration_balls.len());

    // 将装饰球添加到主世界
    for ball in &decoration_balls.objects {
        world.add(ball.clone());
    }

    // 添加一个位于原点的特殊球体，使用Point3Ext::origin
    world.add(Arc::new(Sphere::new(
        Point3::origin(), // 使用Point3Ext::origin
        0.3,
        Arc::new(Metal::new(Color::new(1.0, 0.8, 0.0), 0.1)),
    )));

    // 添加精致的小格子球体
    let micro_checker = Arc::new(CheckerTexture::new_rgb(
        0.3, // 极小的格子
        1.0, 1.0, 0.0, // 黄色
        0.0, 1.0, 1.0, // 青色
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(6.0, 1.0, 0.0),
        1.0,
        Arc::new(Lambertian::new_texture(micro_checker)),
    )));

    // 添加使用DiffuseLight::new的光源
    let light_texture = Arc::new(SolidColor::new(Color::new(4.0, 4.0, 4.0)));
    let light = Arc::new(DiffuseLight::new(light_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 7.0, 0.0),
        2.0,
        light,
    )));

    // 配置相机
    let mut camera = Camera::new();
    camera.aspect_ratio = 16.0 / 9.0;
    camera.image_width = config.image_width;
    camera.samples_per_pixel = config.samples_per_pixel;
    camera.max_depth = config.max_depth;
    camera.background = Color::new(0.7, 0.8, 1.0);

    camera.vfov = 20.0;
    camera.lookfrom = Point3::new(15.0, 3.0, 5.0); // 调整相机位置以更好地观看精致细节
    camera.lookat = Point3::new(0.0, 0.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);
    camera.defocus_angle = 0.0;
    camera.output_filename = "texture_showcase.png".to_string();

    let start = Instant::now();
    eprintln!("开始渲染精致纹理展示场景...");
    eprintln!("场景包含 {} 个物体", world.len()); // 使用len方法

    camera.render(&world, None);

    let duration = start.elapsed();
    eprintln!("精致纹理展示场景渲染完成！总耗时: {:?}", duration);
}
