mod ray_tracing;

use ray_tracing::camera::Camera;
use ray_tracing::hittable::RotateY;
use ray_tracing::hittable::Translate;
use ray_tracing::hittable_list::HittableList;
use ray_tracing::material::{Dielectric, DiffuseLight, Lambertian, NoMaterial};
use ray_tracing::quad::{Quad, box_new};
use ray_tracing::sphere::Sphere;
use ray_tracing::util::random_double;
use ray_tracing::vec3::{Color, Point3, Vec3};
use std::sync::Arc;
use std::time::Instant;

fn cornell_box_with_glass_sphere() {
    // 创建场景
    let mut world = HittableList::new();

    // 创建材质
    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new_color(Color::new(15.0, 15.0, 15.0)));
    let glass = Arc::new(Dielectric::new(1.5));

    // 康奈尔盒侧面
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        Vec3::new(0.0, 555.0, 0.0),
        green,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(0.0, 0.0, -555.0),
        Vec3::new(0.0, 555.0, 0.0),
        red,
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 555.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -555.0),
        white.clone(),
    )));
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 555.0),
        Vec3::new(-555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    )));

    // 光源
    world.add(Arc::new(Quad::new(
        Point3::new(213.0, 554.0, 227.0),
        Vec3::new(130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 105.0),
        light.clone(),
    )));

    // 盒子
    let box1 = box_new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        white.clone(),
    );
    let box1_rotated = Arc::new(RotateY::new(Arc::new(box1), 15.0));
    let box1_translated = Arc::new(Translate::new(box1_rotated, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1_translated);

    // 玻璃球
    world.add(Arc::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        glass,
    )));

    // 光源列表（用于重要性采样）
    let mut lights = HittableList::new();
    lights.add(Arc::new(Quad::new(
        Point3::new(343.0, 554.0, 332.0),
        Vec3::new(-130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, -105.0),
        Arc::new(NoMaterial {}), // 使用空材质代替light
    )));
    lights.add(Arc::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        Arc::new(NoMaterial {}), // 使用空材质代替light
    )));

    // 配置相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = 600;
    cam.samples_per_pixel = 5000;
    cam.max_depth = 75;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(278.0, 278.0, -800.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    cam.output_filename = "output_cornell_box_glass.png".to_string();

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染康奈尔盒场景...");
    cam.render(&world, Some(Arc::new(lights)));
    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

fn monte_carlo_integral(
    f: fn(f64) -> f64,
    icd: fn(f64) -> f64,
    pdf: fn(f64) -> f64,
    n: i32,
) -> f64 {
    let mut sum = 0.0;
    for _ in 0..n {
        let x = random_double();
        let icd_result = icd(x);
        sum += f(icd_result) / pdf(icd_result);
    }
    sum / n as f64
}

fn main() {
    // 渲染康奈尔盒场景（与C++版本相似）
    cornell_box_with_glass_sphere();

    // 保留蒙特卡洛积分函数演示
    let result = monte_carlo_integral(
        |x| x * x,
        |x| (8.0 * x).powf(1.0 / 3.0),
        |x| 3.0 * x * x / 8.0,
        1000,
    );
    println!("Monte Carlo Integral Result: {}", result);
}
