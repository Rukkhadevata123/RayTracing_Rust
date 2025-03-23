use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;

use super::color::{color_to_rgb_with_samples, save_image};
use super::hittable::{HitRecord, Hittable};
use super::hittable_list::HittableList;
use super::interval::Interval;
use super::ray::Ray;
use super::util::{degrees_to_radians, random_double};
use super::vec3::{Color, Point3, Vec3, cross, unit_vector};

pub struct Camera {
    // 公共参数
    pub aspect_ratio: f64,       // 图像宽高比
    pub image_width: i32,        // 以像素计的图像宽度
    pub samples_per_pixel: i32,  // 每个像素的随机采样数
    pub max_depth: i32,          // 最大光线反射次数
    pub output_filename: String, // 输出文件名

    pub vfov: f64,        // 垂直视角（视场角）
    pub lookfrom: Point3, // 相机位置点
    pub lookat: Point3,   // 相机观察点
    pub vup: Vec3,        // 相机向上方向

    pub defocus_angle: f64, // 散焦光圈的角度
    pub focus_dist: f64,    // 焦距

    // 私有参数
    image_height: i32,        // 以像素计的图像高度
    pixel_samples_scale: f64, // 像素样本的颜色缩放因子
    center: Point3,           // 相机中心
    pixel00_loc: Point3,      // 像素(0,0)的位置
    pixel_delta_u: Vec3,      // 水平像素偏移向量
    pixel_delta_v: Vec3,      // 垂直像素偏移向量
    u: Vec3,                  // 相机坐标系基向量u
    v: Vec3,                  // 相机坐标系基向量v
    w: Vec3,                  // 相机坐标系基向量w
    defocus_disk_u: Vec3,     // 散焦光圈水平半径向量
    defocus_disk_v: Vec3,     // 散焦光圈垂直半径向量
}

impl Camera {
    // 创建具有默认参数的相机
    pub fn new() -> Self {
        Self {
            // 公共参数的默认值
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::new(0.0, 0.0, 0.0),
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,

            output_filename: String::from("output.png"),

            // 私有参数将在initialize()中设置
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::zeros(),
            pixel00_loc: Point3::zeros(),
            pixel_delta_u: Vec3::zeros(),
            pixel_delta_v: Vec3::zeros(),
            u: Vec3::zeros(),
            v: Vec3::zeros(),
            w: Vec3::zeros(),
            defocus_disk_u: Vec3::zeros(),
            defocus_disk_v: Vec3::zeros(),
        }
    }

    // 初始化相机参数
    fn initialize(&mut self) {
        // 计算图像高度，确保至少为1
        self.image_height = (f64::from(self.image_width) / self.aspect_ratio) as i32;
        self.image_height = if self.image_height < 1 {
            1
        } else {
            self.image_height
        };

        self.pixel_samples_scale = 1.0 / f64::from(self.samples_per_pixel);

        self.center = self.lookfrom;

        // 计算viewport尺寸
        let theta = degrees_to_radians(self.vfov);
        let h = f64::tan(theta / 2.0);
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (f64::from(self.image_width) / self.image_height as f64);

        // 计算相机坐标系的基向量u,v,w
        self.w = unit_vector(&(self.lookfrom - self.lookat));
        self.u = unit_vector(&cross(&self.vup, &self.w));
        self.v = cross(&self.w, &self.u);

        // 计算viewport边缘的向量
        let viewport_u = viewport_width * self.u; // 横向viewport边缘向量
        let viewport_v = viewport_height * -self.v; // 纵向viewport边缘向量

        // 计算像素到像素的水平和垂直增量向量
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // 计算左上角像素的位置
        let viewport_upper_left =
            self.center - (self.focus_dist * self.w) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        // 计算散焦光圈的基向量
        let defocus_radius =
            self.focus_dist * f64::tan(degrees_to_radians(self.defocus_angle / 2.0));
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    // 获取指定像素位置的光线
    fn get_ray(&self, i: i32, j: i32) -> Ray {
        // 生成一条从散焦光圈出发，穿过像素(i,j)周围随机采样点的光线

        let offset = self.sample_square();
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        let ray_direction = pixel_sample - ray_origin;

        Ray::new(ray_origin, ray_direction)
    }

    // 在单位正方形内采样
    fn sample_square(&self) -> Vec3 {
        // 返回[-0.5,-0.5]-[+0.5,+0.5]单位正方形内的随机点
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    // 在单位圆盘内采样
    fn sample_disk(&self, radius: f64) -> Vec3 {
        // 返回原点为中心半径为radius的圆盘内的随机点
        radius * Vec3::random_in_unit_disk()
    }

    // 在散焦光圈上采样
    fn defocus_disk_sample(&self) -> Point3 {
        // 返回相机散焦光圈内的随机点
        let p = self.sample_disk(1.0);
        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    // 更新 ray_color 方法实现
    fn ray_color(&self, r: &Ray, depth: i32, world: &impl Hittable) -> Color {
        Self::ray_color_internal(r, depth, world)
    }

    // 添加静态方法处理递归逻辑
    fn ray_color_internal(r: &Ray, depth: i32, world: &impl Hittable) -> Color {
        // 如果达到反射次数限制，不再收集光线
        if depth <= 0 {
            return Color::new(0.0, 0.0, 0.0);
        }

        let mut rec = HitRecord::default();

        if world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
            let mut scattered = Ray::default();
            let mut attenuation = Color::default();

            if rec.mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * Self::ray_color_internal(&scattered, depth - 1, world);
            }
            return Color::new(0.0, 0.0, 0.0);
        }

        // 未命中任何物体，返回背景色（简单天空渐变）
        let unit_direction = unit_vector(&r.dir);
        let a = 0.5 * (unit_direction.y + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }

    fn calculate_pixel_color(&self, i: i32, j: i32, world: &impl Hittable) -> Color {
        let mut pixel_color = Color::zeros();

        // Sample multiple rays per pixel for anti-aliasing
        for _ in 0..self.samples_per_pixel {
            let ray = self.get_ray(i, j);
            pixel_color += self.ray_color(&ray, self.max_depth, world);
        }

        pixel_color
    }

    // 渲染场景
    pub fn render(&mut self, world: &HittableList) {
        self.initialize();

        // Create a new image buffer
        let mut img = RgbImage::new(self.image_width as u32, self.image_height as u32);

        // Show progress bar
        let progress_bar = ProgressBar::new(self.image_height as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        // Use rayon for parallel processing
        let pixel_colors: Vec<(i32, i32, Color)> = (0..self.image_height)
            .into_par_iter()
            .flat_map(|j| {
                progress_bar.inc(1);
                (0..self.image_width)
                    .map(|i| {
                        let pixel_color = self.calculate_pixel_color(i, j, world);
                        (i, j, pixel_color)
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        // Fill the image buffer
        for (i, j, color) in pixel_colors {
            let rgb = color_to_rgb_with_samples(&color, self.samples_per_pixel);
            // Fix the inverted image by using j directly instead of flipping it
            img.put_pixel(
                i as u32, j as u32, // Use j directly without flipping
                rgb,
            );
        }

        // Save the image
        match save_image(img, &self.output_filename) {
            Ok(_) => eprintln!("图像已保存为 {}", self.output_filename),
            Err(e) => eprintln!("保存图像时出错: {}", e),
        }

        progress_bar.finish_and_clear();
    }
}
