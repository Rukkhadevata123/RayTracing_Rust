use image::RgbImage;
use indicatif::{ProgressBar, ProgressStyle};
use rayon::prelude::*;
use std::sync::Arc;

use super::color::{color_to_rgb_with_samples, save_image};
use super::hittable::{HitRecord, Hittable};
use super::interval::Interval;
use super::material::ScatterRecord;
use super::pdf::{HittablePDF, MixturePDF, PDF};
use super::ray::Ray;
use super::util::{degrees_to_radians, random_double, random_double_range};
use super::vec3::{Color, Point3, Vec3, cross, unit_vector};

pub struct Camera {
    // 公共参数
    pub aspect_ratio: f64,       // 图像宽高比
    pub image_width: i32,        // 以像素计的图像宽度
    pub samples_per_pixel: i32,  // 每个像素的随机采样数
    pub max_depth: i32,          // 最大光线反射次数
    pub background: Color,       // 背景色
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
    sqrt_spp: i32,            // 每个像素的采样数的平方根
    recip_sqrt_spp: f64,      // 每个像素的采样数的平方根的倒数
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
            background: Color::new(0.7, 0.8, 1.0),
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
            sqrt_spp: 0,
            recip_sqrt_spp: 0.0,
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

        self.sqrt_spp = (self.samples_per_pixel as f64).sqrt() as i32;
        self.pixel_samples_scale = 1.0 / (self.sqrt_spp * self.sqrt_spp) as f64;
        self.recip_sqrt_spp = 1.0 / (self.sqrt_spp as f64);

        self.center = self.lookfrom;

        // 计算viewport尺寸
        let theta = degrees_to_radians(self.vfov);
        let h = f64::tan(theta / 2.0);
        let viewport_height = 2.0 * h * self.focus_dist;
        let viewport_width =
            viewport_height * (f64::from(self.image_width) / self.image_height as f64);

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
    fn get_ray(&self, i: i32, j: i32, s_i: i32, s_j: i32) -> Ray {
        // 使用分层采样获取像素内的偏移
        let offset = self.sample_square_stratified(s_i, s_j);
        // let offset = self.sample_square();

        // 计算像素采样点的位置
        let pixel_sample = self.pixel00_loc
            + ((i as f64 + offset.x) * self.pixel_delta_u)
            + ((j as f64 + offset.y) * self.pixel_delta_v);

        // 确定光线起点
        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };

        // 计算光线方向
        let ray_direction = pixel_sample - ray_origin;
        let ray_time = random_double_range(0.0, 1.0);

        Ray::new(ray_origin, ray_direction, ray_time)
    }

    // 在单位正方形内采样
    fn sample_square(&self) -> Vec3 {
        // 返回[-0.5,-0.5]-[+0.5,+0.5]单位正方形内的随机点
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    // 分层采样
    fn sample_square_stratified(&self, s_i: i32, s_j: i32) -> Vec3 {
        // 返回[-0.5,-0.5]-[+0.5,+0.5]单位正方形内的随机点
        let x = (s_i as f64 + random_double()) * self.recip_sqrt_spp as f64 - 0.5;
        let y = (s_j as f64 + random_double()) * self.recip_sqrt_spp as f64 - 0.5;
        Vec3::new(x, y, 0.0)
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

    // 更新 ray_color 方法实现，添加俄罗斯轮盘赌优化
    fn ray_color(
        &self,
        r: &Ray,
        depth: i32,
        world: &dyn Hittable,
        lights: Option<&Arc<dyn Hittable>>,
    ) -> Color {
        // 如果达到最大深度限制，直接返回黑色（不再反射）
        if depth <= 0 {
            return Color::zeros();
        }

        let mut rec = HitRecord::default();

        // 如果没有击中任何物体，返回背景色
        if !world.hit(r, Interval::new(0.001, f64::INFINITY), &mut rec) {
            return self.background;
        }

        // 光源发射的光
        let emission = rec.mat.emitted(rec.u, rec.v, &rec.p);

        // 创建一个散射记录
        let mut srec = ScatterRecord::new();

        // 如果材质不散射光线，只返回发射光
        if !rec.mat.scatter(r, &rec, &mut srec) {
            return emission;
        }

        // 处理跳过PDF的情况 (镜面反射等)
        if srec.skip_pdf {
            return emission
                + srec.attenuation * self.ray_color(&srec.skip_pdf_ray, depth - 1, world, lights);
        }

        // 使用光源和BRDF混合采样
        let light_ptr =
            lights.map(|l| -> Arc<dyn PDF> { Arc::new(HittablePDF::new(l.clone(), &rec.p)) });

        // 生成散射光线
        let scattered_direction;
        let pdf_value;

        if let Some(light_pdf) = light_ptr {
            // 使用混合PDF生成方向
            let mixture_pdf = MixturePDF::new(
                light_pdf,
                srec.pdf_ptr.expect("材质返回了没有PDF的散射记录"),
            );

            scattered_direction = mixture_pdf.generate();
            pdf_value = mixture_pdf.value(&scattered_direction);
        } else {
            // 只使用BRDF采样
            let pdf = srec.pdf_ptr.expect("材质返回了没有PDF的散射记录");
            scattered_direction = pdf.generate();
            pdf_value = pdf.value(&scattered_direction);
        }

        let scattered = Ray::new(rec.p, scattered_direction, r.time);
        let scattering_pdf = rec.mat.scattering_pdf(r, &rec, &scattered);

        // 实现俄罗斯轮盘赌优化
        // 当深度大于3时，以一定概率提前终止光线
        if depth < self.max_depth - 3 {
            // 终止概率随深度增加而提高
            let rr_prob = 0.8; // 继续跟踪的概率

            // 生成随机数决定是否继续追踪
            if super::util::random_double() > rr_prob {
                // 提前终止，返回当前累积的发射光
                return emission;
            }

            // 如果继续追踪，需要对贡献进行权重调整，除以继续概率
            let rr_scale = 1.0 / rr_prob;

            // 计算最终颜色贡献
            return emission
                + rr_scale
                    * (srec.attenuation
                        * scattering_pdf
                        * self.ray_color(&scattered, depth - 1, world, lights))
                    / pdf_value;
        }

        // 正常计算颜色贡献
        emission
            + (srec.attenuation
                * scattering_pdf
                * self.ray_color(&scattered, depth - 1, world, lights))
                / pdf_value
    }

    fn calculate_pixel_color(
        &self,
        i: i32,
        j: i32,
        world: &dyn Hittable,
        lights: Option<&Arc<dyn Hittable>>,
    ) -> Color {
        let mut pixel_color = Color::zeros();

        // 使用分层采样
        // 将像素分成 sqrt_spp × sqrt_spp 的网格
        for s_i in 0..self.sqrt_spp {
            for s_j in 0..self.sqrt_spp {
                // 在每个子区域内生成一条光线
                let ray = self.get_ray(i, j, s_i, s_j);
                pixel_color += self.ray_color(&ray, self.max_depth, world, lights);
            }
        }

        pixel_color
    }

    // 渲染场景
    pub fn render(&mut self, world: &dyn Hittable, lights: Option<Arc<dyn Hittable>>) {
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
                        let pixel_color = self.calculate_pixel_color(i, j, world, lights.as_ref());
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
