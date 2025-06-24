use crate::ray_tracing::geometry::hittable_list::HittableList;
use crate::ray_tracing::geometry::quad::{Quad, box_new};
use crate::ray_tracing::geometry::sphere::Sphere;
use crate::ray_tracing::geometry::transforms::rotate_y::RotateY;
use crate::ray_tracing::geometry::transforms::translate::Translate;
use crate::ray_tracing::materials::dielectric::Dielectric;
use crate::ray_tracing::materials::diffuse_light::DiffuseLight;
use crate::ray_tracing::materials::lambertian::Lambertian;
use crate::ray_tracing::materials::material::NoMaterial;
use crate::ray_tracing::math::vec3::{Color, Point3, Vec3};
use crate::ray_tracing::rendering::camera::Camera;
use std::sync::Arc;
use std::time::Instant;

/// 康奈尔盒场景配置
pub struct CornellBoxConfig {
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub output_filename: String,
}

impl Default for CornellBoxConfig {
    fn default() -> Self {
        Self {
            image_width: 600,
            samples_per_pixel: 1000,
            max_depth: 50,
            output_filename: "cornell_box.png".to_string(),
        }
    }
}

/// 构建基础康奈尔盒场景
pub fn build_cornell_box_scene() -> (HittableList, HittableList) {
    let mut world = HittableList::new();
    let mut lights = HittableList::new();

    // 创建材质
    let red = Arc::new(Lambertian::new(Color::new(0.65, 0.05, 0.05)));
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let green = Arc::new(Lambertian::new(Color::new(0.12, 0.45, 0.15)));
    let light = Arc::new(DiffuseLight::new_color(Color::new(15.0, 15.0, 15.0)));

    // 康奈尔盒的六个面
    // 右面（绿色）
    world.add(Arc::new(Quad::new(
        Point3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        green,
    )));

    // 左面（红色）
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        red,
    )));

    // 顶面（白色）
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 555.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));

    // 底面（白色）
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 0.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 555.0),
        white.clone(),
    )));

    // 后面（白色）
    world.add(Arc::new(Quad::new(
        Point3::new(0.0, 0.0, 555.0),
        Vec3::new(555.0, 0.0, 0.0),
        Vec3::new(0.0, 555.0, 0.0),
        white.clone(),
    )));

    // 光源
    let light_quad = Arc::new(Quad::new(
        Point3::new(213.0, 554.0, 227.0),
        Vec3::new(130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 105.0),
        light,
    ));
    world.add(light_quad.clone());

    // 光源列表（用于重要性采样）
    lights.add(Arc::new(Quad::new(
        Point3::new(213.0, 554.0, 227.0),
        Vec3::new(130.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 105.0),
        Arc::new(NoMaterial),
    )));

    (world, lights)
}

/// 康奈尔盒 + 玻璃球场景
pub fn cornell_box_with_glass_sphere(config: CornellBoxConfig) {
    let (mut world, mut lights) = build_cornell_box_scene();

    // 添加白色盒子
    let white = Arc::new(Lambertian::new(Color::new(0.73, 0.73, 0.73)));
    let box1 = box_new(
        Point3::new(0.0, 0.0, 0.0),
        Point3::new(165.0, 330.0, 165.0),
        white,
    );
    let box1_rotated = Arc::new(RotateY::new(Arc::new(box1), 15.0));
    let box1_translated = Arc::new(Translate::new(box1_rotated, Vec3::new(265.0, 0.0, 295.0)));
    world.add(box1_translated);

    // 添加玻璃球
    let glass_sphere = Arc::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
        Arc::new(Dielectric::new(1.5)),
    ));
    world.add(glass_sphere.clone());

    // 将玻璃球也加入光源列表（用于重要性采样）
    lights.add(Arc::new(Sphere::new(
        Point3::new(190.0, 90.0, 190.0),
        90.0,
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
    camera.lookfrom = Point3::new(278.0, 278.0, -800.0);
    camera.lookat = Point3::new(278.0, 278.0, 0.0);
    camera.vup = Vec3::new(0.0, 1.0, 0.0);
    camera.defocus_angle = 0.0;
    camera.output_filename = config.output_filename;

    // 渲染
    let start = Instant::now();
    eprintln!("开始渲染康奈尔盒场景...");
    eprintln!(
        "图像大小: {}x{}, 采样数: {}, 反射深度: {}",
        config.image_width, config.image_width, config.samples_per_pixel, config.max_depth
    );

    camera.render(&world, Some(Arc::new(lights)));

    let duration = start.elapsed();
    eprintln!("渲染完成！总耗时: {:?}", duration);
}
