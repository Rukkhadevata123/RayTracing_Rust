use super::color::color_to_rgb_with_samples;
use crate::ray_tracing::geometry::hittable::{HitRecord, Hittable};
use crate::ray_tracing::materials::material::ScatterRecord;
use crate::ray_tracing::math::interval::Interval;
use crate::ray_tracing::math::ray::Ray;
use crate::ray_tracing::math::vec3::*;
use crate::ray_tracing::sampling::pdf::{HittablePDF, MixturePDF, PDF};
use crate::ray_tracing::utils::random::{degrees_to_radians, random_double, random_double_range};
use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Arc;

/// 相机配置和渲染器
#[derive(Debug)]
pub struct Camera {
    // 公共配置参数
    pub aspect_ratio: f64,
    pub image_width: i32,
    pub samples_per_pixel: i32,
    pub max_depth: i32,
    pub background: Color,
    pub output_filename: String,

    // 相机位置和方向
    pub vfov: f64,
    pub lookfrom: Point3,
    pub lookat: Point3,
    pub vup: Vec3,

    // 景深参数
    pub defocus_angle: f64,
    pub focus_dist: f64,

    // 私有计算参数
    image_height: i32,
    pixel_samples_scale: f64,
    sqrt_spp: i32,
    recip_sqrt_spp: f64,
    center: Point3,
    pixel00_loc: Point3,
    pixel_delta_u: Vec3,
    pixel_delta_v: Vec3,
    u: Vec3,
    v: Vec3,
    w: Vec3,
    defocus_disk_u: Vec3,
    defocus_disk_v: Vec3,
}

impl Camera {
    /// 创建默认相机
    #[inline]
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            background: Color::new(0.7, 0.8, 1.0),
            output_filename: "output.png".to_string(),

            vfov: 90.0,
            lookfrom: Point3::origin(),
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),

            defocus_angle: 0.0,
            focus_dist: 10.0,

            // 私有参数在initialize中设置
            image_height: 0,
            pixel_samples_scale: 0.0,
            sqrt_spp: 0,
            recip_sqrt_spp: 0.0,
            center: Point3::origin(),
            pixel00_loc: Point3::origin(),
            pixel_delta_u: Vec3::zeros(),
            pixel_delta_v: Vec3::zeros(),
            u: Vec3::zeros(),
            v: Vec3::zeros(),
            w: Vec3::zeros(),
            defocus_disk_u: Vec3::zeros(),
            defocus_disk_v: Vec3::zeros(),
        }
    }

    /// 初始化相机参数
    fn initialize(&mut self) {
        // 计算图像高度
        self.image_height = ((self.image_width as f64) / self.aspect_ratio) as i32;
        self.image_height = self.image_height.max(1);

        // 计算采样参数
        self.sqrt_spp = (self.samples_per_pixel as f64).sqrt() as i32;
        self.pixel_samples_scale = 1.0 / (self.sqrt_spp * self.sqrt_spp) as f64;
        self.recip_sqrt_spp = 1.0 / (self.sqrt_spp as f64);

        self.center = self.lookfrom;

        // 计算视口参数
        let theta = degrees_to_radians(self.vfov);
        let h = (theta / 2.0).tan();
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // 计算相机基向量
        self.w = (self.lookfrom - self.lookat).normalize();
        self.u = self.vup.cross(&self.w).normalize();
        self.v = self.w.cross(&self.u);

        // 计算视口边缘向量
        let viewport_u = viewport_width * self.u;
        let viewport_v = viewport_height * (-self.v);

        // 计算像素步长
        self.pixel_delta_u = viewport_u / (self.image_width as f64);
        self.pixel_delta_v = viewport_v / (self.image_height as f64);

        // 计算左上角像素位置
        let viewport_upper_left =
            self.center - (self.focus_dist * self.w) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        // 计算散焦光圈参数
        let defocus_radius = self.focus_dist * degrees_to_radians(self.defocus_angle / 2.0).tan();
        self.defocus_disk_u = self.u * defocus_radius;
        self.defocus_disk_v = self.v * defocus_radius;
    }

    /// 生成光线
    #[inline]
    fn get_ray(&self, i: i32, j: i32, s_i: i32, s_j: i32) -> Ray {
        let offset = self.sample_square_stratified(s_i, s_j);
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        let ray_direction = pixel_sample - ray_origin;
        let ray_time = random_double_range(0.0, 1.0);

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    /// 分层采样
    #[inline]
    fn sample_square_stratified(&self, s_i: i32, s_j: i32) -> Vec3 {
        let x = (s_i as f64 + random_double()) * self.recip_sqrt_spp - 0.5;
        let y = (s_j as f64 + random_double()) * self.recip_sqrt_spp - 0.5;
        Vec3::new(x, y, 0.0)
    }

    /// 散焦光圈采样
    #[inline]
    fn defocus_disk_sample(&self) -> Point3 {
        let p = Vec3::random_in_unit_disk();
        self.center + (p.x * self.defocus_disk_u) + (p.y * self.defocus_disk_v)
    }

    /// 计算光线颜色，使用重要性采样和俄罗斯轮盘赌
    fn ray_color(
        &self,
        r: &Ray,
        depth: i32,
        world: &dyn Hittable,
        lights: Option<&Arc<dyn Hittable>>,
    ) -> Color {
        if depth <= 0 {
            return Color::zeros();
        }

        let mut rec = HitRecord::default();
        if !world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
            return self.background;
        }

        // 材质发射的光
        let emission = rec.mat.emitted(rec.u, rec.v, &rec.p);

        // 散射计算
        let mut srec = ScatterRecord::new();
        if !rec.mat.scatter(r, &rec, &mut srec) {
            return emission;
        }

        // 镜面反射跳过PDF
        if srec.skip_pdf {
            return emission
                + srec.attenuation.component_mul(&self.ray_color(
                    &srec.skip_pdf_ray,
                    depth - 1,
                    world,
                    lights,
                ));
        }

        // 重要性采样：混合光源和BRDF采样
        let (scattered_direction, pdf_value) = if let Some(light_objects) = lights {
            let light_pdf = Arc::new(HittablePDF::new(light_objects.clone(), &rec.p));
            let mixture_pdf = MixturePDF::new(light_pdf, srec.pdf_ptr.expect("材质必须提供PDF"));

            let direction = mixture_pdf.generate();
            let pdf = mixture_pdf.value(&direction);
            (direction, pdf)
        } else {
            let pdf = srec.pdf_ptr.expect("材质必须提供PDF");
            let direction = pdf.generate();
            let pdf_val = pdf.value(&direction);
            (direction, pdf_val)
        };

        // 避免除零和无效PDF
        if pdf_value < 1e-6 || !pdf_value.is_finite() {
            return emission;
        }

        let scattered = Ray::new(rec.p, scattered_direction, r.time);
        let scattering_pdf = rec.mat.scattering_pdf(r, &rec, &scattered);

        // 俄罗斯轮盘赌优化
        if depth > 3 {
            let rr_prob = 0.8;
            if random_double() > rr_prob {
                return emission;
            }

            let rr_scale = 1.0 / rr_prob;
            return emission
                + rr_scale
                    * (srec.attenuation.component_mul(
                        &(scattering_pdf * self.ray_color(&scattered, depth - 1, world, lights)),
                    ))
                    / pdf_value;
        }

        // 正常递归
        emission
            + (srec.attenuation.component_mul(
                &(scattering_pdf * self.ray_color(&scattered, depth - 1, world, lights)),
            )) / pdf_value
    }

    /// 计算单个像素的颜色
    fn calculate_pixel_color(
        &self,
        i: i32,
        j: i32,
        world: &dyn Hittable,
        lights: Option<&Arc<dyn Hittable>>,
    ) -> Color {
        let total_samples = self.sqrt_spp * self.sqrt_spp;

        (0..total_samples)
            .into_par_iter()
            .map(|sample_idx| {
                let s_i = sample_idx / self.sqrt_spp;
                let s_j = sample_idx % self.sqrt_spp;
                let ray = self.get_ray(i, j, s_i, s_j);
                self.ray_color(&ray, self.max_depth, world, lights)
            })
            .reduce(Color::zeros, |acc, color| acc + color)
    }

    /// 主渲染方法
    pub fn render(&mut self, world: &dyn Hittable, lights: Option<Arc<dyn Hittable>>) {
        self.initialize();

        let mut img = RgbImage::new(self.image_width as u32, self.image_height as u32);

        // 进度条设置
        let progress_bar = ProgressBar::new((self.image_height * self.image_width) as u64);
        progress_bar.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
                )
                .unwrap()
                .progress_chars("#>-"),
        );

        // 设置块大小 - 通常16x16或32x32效果较好
        let tile_size = 16;
        let num_tiles_x = (self.image_width + tile_size - 1) / tile_size;
        let num_tiles_y = (self.image_height + tile_size - 1) / tile_size;
        let total_tiles = num_tiles_x * num_tiles_y;

        // 并行渲染分块
        let pixel_colors: Vec<(i32, i32, Color)> = (0..total_tiles)
            .into_par_iter()
            .flat_map(|tile_idx| {
                let tile_x = (tile_idx % num_tiles_x) * tile_size;
                let tile_y = (tile_idx / num_tiles_x) * tile_size;

                let mut tile_results = Vec::with_capacity((tile_size * tile_size) as usize);

                // 处理这个块内的所有像素
                for j in tile_y..std::cmp::min(tile_y + tile_size, self.image_height) {
                    for i in tile_x..std::cmp::min(tile_x + tile_size, self.image_width) {
                        let pixel_color = self.calculate_pixel_color(i, j, world, lights.as_ref());
                        tile_results.push((i, j, pixel_color));
                        progress_bar.inc(1);
                    }
                }

                tile_results
            })
            .collect();

        // 填充图像缓冲区
        for (i, j, color) in pixel_colors {
            let rgb = color_to_rgb_with_samples(&color, self.samples_per_pixel);
            img.put_pixel(i as u32, j as u32, rgb);
        }

        // 保存图像
        match img.save(&self.output_filename) {
            Ok(_) => eprintln!("图像已保存为 {}", self.output_filename),
            Err(e) => eprintln!("保存图像时出错: {}", e),
        }

        progress_bar.finish_and_clear();
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
