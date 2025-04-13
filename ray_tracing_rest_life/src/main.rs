mod ray_tracing;

use ray_tracing::camera::Camera;
use ray_tracing::hittable::RotateY;
use ray_tracing::hittable::Translate;
use ray_tracing::hittable_list::HittableList;
use ray_tracing::material::{Dielectric, DiffuseLight, Lambertian, Metal, NoMaterial};
use ray_tracing::quad::{Quad, box_new};
use ray_tracing::sphere::Sphere;
use ray_tracing::util::{random_double, random_double_range};
use ray_tracing::vec3::{Color, Point3, Vec3};
use std::sync::Arc;
use std::time::Instant;
use ray_tracing::bvh::BvhNode;
use ray_tracing::constant_medium::ConstantMedium;
use ray_tracing::texture::{ImageTexture, NoiseTexture};

fn final_scene_next_week(image_width: u32, samples_per_pixel: u32, max_depth: i32) {
    let mut boxes1 = HittableList::new();
    let ground = Arc::new(Lambertian::new(Color::new(0.48, 0.83, 0.53)));

    // 创建地面上的随机高度盒子
    let boxes_per_side = 20;
    for i in 0..boxes_per_side {
        for j in 0..boxes_per_side {
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

    let mut world = HittableList::new();

    // 使用BVH加速地面上的盒子
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

    // 添加其他球体
    world.add(Arc::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::new(Dielectric::new(1.5)),
    )));
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
        Point3::new(0.0, 0.0, 0.0),
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
    let emat = Arc::new(Lambertian::new_texture(earth_texture));
    world.add(Arc::new(Sphere::new(
        Point3::new(400.0, 200.0, 400.0),
        100.0,
        emat,
    )));

    // 噪声纹理球
    let pertext = Arc::new(NoiseTexture::new(0.2));
    world.add(Arc::new(Sphere::new(
        Point3::new(220.0, 280.0, 300.0),
        80.0,
        Arc::new(Lambertian::new_texture(pertext)),
    )));

    // 添加一组随机小球
    let mut boxes2 = HittableList::new();
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let ns = 1000;
    for _ in 0..ns {
        boxes2.add(Arc::new(Sphere::new(
            Point3::random_range(0.0, 165.0),
            10.0,
            white.clone(),
        )));
    }

    // 使用BVH加速小球集合，然后旋转和平移
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
        Arc::new(NoMaterial {}), // 使用空材质代替light
    )));

     lights.add(Arc::new(Sphere::new(
        Point3::new(260.0, 150.0, 45.0),
        50.0,
        Arc::new(NoMaterial {}), // 使用空材质代替light
     )));

    // 配置相机
    let mut cam = Camera::new();
    cam.aspect_ratio = 1.0;
    cam.image_width = image_width as i32;
    cam.samples_per_pixel = samples_per_pixel as i32;
    cam.max_depth = max_depth;
    cam.background = Color::new(0.0, 0.0, 0.0); // 深蓝色背景

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(478.0, 278.0, -600.0);
    cam.lookat = Point3::new(278.0, 278.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;

    cam.output_filename = format!(
        "output_final_scene_{}x{}_{}spp.png",
        image_width, image_width, samples_per_pixel
    );

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染最终场景...");
    eprintln!(
        "图像大小: {}x{}, 采样数: {}, 反射深度: {}",
        image_width, image_width, samples_per_pixel, max_depth
    );
    
    // 在 rest_of_life 版本中，支持重要性采样
    cam.render(&world, Some(Arc::new(lights)));
    // cam.render(&world, None);
    
    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}

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
    // cornell_box_with_glass_sphere();

    final_scene_next_week(800, 5000, 75);

    // 保留蒙特卡洛积分函数演示
    let result = monte_carlo_integral(
        |x| x * x,
        |x| (8.0 * x).powf(1.0 / 3.0),
        |x| 3.0 * x * x / 8.0,
        1,
    );
    println!("Monte Carlo Integral Result: {}", result);
}
