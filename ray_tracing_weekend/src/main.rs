mod ray_tracing;

use ray_tracing::camera::Camera;
use ray_tracing::hittable_list::HittableList;
use ray_tracing::material::{Dielectric, Lambertian, Metal};
use ray_tracing::sphere::Sphere;
use ray_tracing::util::random_double;
use ray_tracing::vec3::{Color, Point3, Vec3};
use std::sync::Arc;
use std::time::Instant;

fn render_original_scene_with_color(lambertian_color: Color) {
    // 创建世界和物体
    let mut world = HittableList::new();

    // 添加地面
    let ground_material = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.5)));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    // 添加三个主要球体
    // 1. 玻璃球
    let material1 = Arc::new(Dielectric::new(1.5));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1.clone(),
    )));

    // 2. 漫反射球
    let material2 = Arc::new(Lambertian::new(lambertian_color));
    world.add(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    // 3. 金属球
    let material3 = Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    // 添加更多的小球
    for a in -15..15 {
        for b in -15..15 {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                if choose_mat < 0.8 {
                    // 漫反射材质
                    let albedo = Color::random() * Color::random();
                    let sphere_material = Arc::new(Lambertian::new(albedo));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                } else if choose_mat < 0.95 {
                    // 金属材质
                    let albedo = Color::random_range(0.5, 1.0);
                    let fuzz = random_double() * 0.5;
                    let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                } else {
                    // 玻璃材质
                    let sphere_material = Arc::new(Dielectric::new(1.5));
                    world.add(Arc::new(Sphere::new(center, 0.2, sphere_material)));
                }
            }
        }
    }

    // 配置相机参数 - 高质量设置
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 150;

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(13.0, 2.0, 3.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.6;
    cam.focus_dist = 10.0;

    // Generate random filename
    let random_filename = format!("output_{}.png", random_double());
    cam.output_filename = random_filename;

    // 记录渲染时间
    let start = Instant::now();

    // 渲染场景
    eprintln!("开始渲染原始场景...");
    cam.render(&world);

    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

/// 创建并渲染原始的随机球体场景
fn render_original_scene() {
    // 创建漫反射球体场景
    render_original_scene_with_color(Color::new(0.4, 0.2, 0.1));
}

/// 创建梦幻泡泡场景
fn render_bubble_scene() {
    let mut world = HittableList::new();

    // 地面 - 使用金属材质代替透明材质，降低反射率
    let ground_material = Arc::new(Metal::new(
        Color::new(0.4, 0.5, 0.6), // 柔和的蓝灰色
        0.2,                       // 适度的模糊度
    ));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    // 在不同高度添加混合材质的球体
    for i in -5..5 {
        for j in -5..5 {
            let center = Point3::new(
                i as f64 * 2.0 + random_double() * 0.5, // 减小随机偏移
                0.2 + random_double() * 0.8,            // 降低整体高度
                j as f64 * 2.0 + random_double() * 0.5,
            );

            let choose_mat = random_double();

            if choose_mat < 0.7 {
                // 70% 概率生成透明球体
                let refraction = 1.2 + random_double() * 0.3; // 减小折射率范围
                let sphere_material = Arc::new(Dielectric::new(refraction));
                world.add(Arc::new(Sphere::new(
                    center,
                    0.2 + random_double() * 0.1,
                    sphere_material,
                )));
            } else {
                // 30% 概率生成金属球体
                let albedo = Color::new(
                    0.7 + random_double() * 0.3,
                    0.7 + random_double() * 0.3,
                    0.9 + random_double() * 0.1, // 偏蓝色
                );
                let fuzz = random_double() * 0.3; // 降低模糊度
                let sphere_material = Arc::new(Metal::new(albedo, fuzz));
                world.add(Arc::new(Sphere::new(
                    center,
                    0.15 + random_double() * 0.1,
                    sphere_material,
                )));
            }
        }
    }

    // 添加几个较大的特征球体
    let big_glass = Arc::new(Dielectric::new(1.3));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        big_glass.clone(),
    )));

    let metal_blue = Arc::new(Metal::new(Color::new(0.6, 0.8, 0.95), 0.1));
    world.add(Arc::new(Sphere::new(
        Point3::new(-2.5, 0.7, -2.5),
        0.7,
        metal_blue,
    )));

    // 配置相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 100; // 减少反射次数

    // 调整相机角度和参数
    cam.vfov = 35.0;
    cam.lookfrom = Point3::new(2.0, 3.0, 8.0); // 从侧面略微俯视
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    // 增加景深效果
    cam.defocus_angle = 0.3;
    cam.focus_dist = 8.0;

    cam.output_filename = String::from("bubble_dream.png");

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染梦幻泡泡场景...");
    cam.render(&world);
    eprintln!("渲染完成！总耗时: {:?}", start.elapsed());
}

/// 创建金属王国场景
fn render_metal_scene() {
    let mut world = HittableList::new();

    // 金属地面
    let ground_material = Arc::new(Metal::new(Color::new(0.7, 0.7, 0.8), 0.1));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    // 创建金属球阵列
    for i in -3..3 {
        for j in -3..3 {
            let center = Point3::new(i as f64 * 2.5, 1.0, j as f64 * 2.5);

            // 随机金属颜色和模糊度
            let albedo = Color::new(
                0.5 + random_double() * 0.5,
                0.5 + random_double() * 0.5,
                0.5 + random_double() * 0.5,
            );
            let fuzz = random_double() * 0.5;
            let sphere_material = Arc::new(Metal::new(albedo, fuzz));
            world.add(Arc::new(Sphere::new(center, 1.0, sphere_material)));
        }
    }

    // 配置相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 150;

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(15.0, 5.0, 15.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.3;
    cam.focus_dist = 20.0;

    cam.output_filename = String::from("metal.png");

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染金属王国场景...");
    cam.render(&world);
    eprintln!("渲染完成！总耗时: {:?}", start.elapsed());
}

/// 创建暖阳黄昏场景
fn render_sunset_scene() {
    let mut world = HittableList::new();

    // 暖色调地面
    let ground_material = Arc::new(Lambertian::new(Color::new(0.8, 0.6, 0.3)));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    // 创建不同大小和色调的漫反射球体
    let warm_colors = [
        Color::new(0.9, 0.3, 0.2), // 红橙色
        Color::new(0.9, 0.6, 0.2), // 金黄色
        Color::new(0.7, 0.3, 0.1), // 深橙色
        Color::new(0.8, 0.4, 0.3), // 珊瑚色
    ];

    for i in -4..4 {
        for j in -4..4 {
            let center = Point3::new(
                i as f64 * 2.0 + random_double(),
                0.4 + random_double() * 1.2,
                j as f64 * 2.0 + random_double(),
            );

            // 修复：正确计算随机索引
            let color_index = (random_double() * warm_colors.len() as f64) as usize;
            let color = warm_colors[color_index];

            let sphere_material = Arc::new(Lambertian::new(color));
            let radius = 0.3 + random_double() * 0.3;
            world.add(Arc::new(Sphere::new(center, radius, sphere_material)));
        }
    }

    // 配置相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 150;

    cam.vfov = 25.0;
    cam.lookfrom = Point3::new(8.0, 3.0, 8.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.5;
    cam.focus_dist = 12.0;

    cam.output_filename = String::from("sunset.png");

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染暖阳黄昏场景...");
    cam.render(&world);
    eprintln!("渲染完成！总耗时: {:?}", start.elapsed());
}

fn first_test() {
    eprintln!("\n=== 开始渲染所有场景 ===\n");

    eprintln!("1. 渲染原始的随机球体场景");
    render_original_scene();
    eprintln!("\n-------------------------------\n");

    eprintln!("2. 渲染梦幻泡泡场景");
    render_bubble_scene();
    eprintln!("\n-------------------------------\n");

    eprintln!("3. 渲染金属王国场景");
    render_metal_scene();
    eprintln!("\n-------------------------------\n");

    eprintln!("4. 渲染暖阳黄昏场景");
    render_sunset_scene();

    eprintln!("\n=== 所有场景渲染完成 ===");
}

fn second_test() {
    for _ in 0..10 {
        render_original_scene_with_color(Color::random_range(0.0, 1.0));
    }
}

fn main() {
    // first_test();
    second_test();
}
