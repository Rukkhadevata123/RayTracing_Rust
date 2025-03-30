mod ray_tracing;

use ray_tracing::bvh::BvhNode;
use ray_tracing::camera::Camera;
use ray_tracing::hittable_list::HittableList;
use ray_tracing::material::{Dielectric, Lambertian, Metal};
use ray_tracing::sphere::Sphere;
use ray_tracing::texture::{CheckerTexture, ImageTexture, NoiseTexture};
use ray_tracing::util::{random_double, random_double_range};
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
                    let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);
                    world.add(Arc::new(Sphere::new_moving(
                        center,
                        center2,
                        0.2,
                        sphere_material,
                    )));
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

    let bvh_world = BvhNode::new(&world);

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
    cam.render(&bvh_world);

    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

/// 创建并渲染原始的随机球体场景
fn render_original_scene() {
    // 创建漫反射球体场景
    render_original_scene_with_color(Color::new(0.4, 0.2, 0.1));
}

// 添加新的带纹理的场景渲染函数
fn render_checker_scene() {
    // 创建世界和物体
    let mut world = HittableList::new();

    // 创建棋盘格纹理的地面
    let checker_texture = Arc::new(CheckerTexture::new_colors(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));

    let ground_material = Arc::new(Lambertian::new_texture(checker_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    // 添加中心球体 - 使用噪声纹理
    let noise_texture = Arc::new(NoiseTexture::new(4.0));
    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::new(Lambertian::new_texture(noise_texture)),
    )));

    // 添加金属球
    world.add(Arc::new(Sphere::new(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        Arc::new(Metal::new(Color::new(0.7, 0.6, 0.5), 0.0)),
    )));

    // 添加玻璃球
    world.add(Arc::new(Sphere::new(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        Arc::new(Dielectric::new(1.5)),
    )));

    // 使用 BVH 加速
    let bvh_world = BvhNode::new(&world);

    // 配置相机参数
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 50;

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(13.0, 2.0, 3.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0; // 无景深效果
    cam.focus_dist = 10.0;

    cam.output_filename = "output_checker_texture.png".to_string();

    // 记录渲染时间
    let start = Instant::now();

    // 渲染场景
    eprintln!("开始渲染棋盘格纹理场景...");
    cam.render(&bvh_world);

    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

// 添加带有图像纹理的场景
fn render_earth() {
    let mut world = HittableList::new();

    // 地球纹理
    let earth_texture = Arc::new(ImageTexture::new("textures/earthmap.jpg"));
    let earth_surface = Arc::new(Lambertian::new_texture(earth_texture));

    world.add(Arc::new(Sphere::new(
        Point3::new(0.0, 0.0, 0.0),
        2.0,
        earth_surface,
    )));

    // 使用 BVH 加速
    let bvh_world = BvhNode::new(&world);

    // 配置相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 500;
    cam.max_depth = 50;

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(0.0, 0.0, 12.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;

    cam.output_filename = "output_earth.png".to_string();

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染地球纹理场景...");
    cam.render(&bvh_world);
    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

fn second_test() {
    for _ in 0..10 {
        render_original_scene_with_color(Color::random_range(0.0, 1.0));
    }
}

fn main() {
    // first_test();
    // second_test();
    // render_original_scene();
    // render_checker_scene();     // 棋盘格纹理场景
    render_earth();          // 地球纹理场景
}
