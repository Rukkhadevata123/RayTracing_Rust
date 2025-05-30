mod ray_tracing;
mod scenes;

use scenes::{
    cornell_box::{CornellBoxConfig, cornell_box_with_glass_sphere},
    final_scene::{FinalSceneConfig, final_scene_next_week},
};
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    // 根据命令行参数选择场景
    match args.get(1).map(String::as_str) {
        Some("cornell") => {
            let config = CornellBoxConfig {
                image_width: 600,
                samples_per_pixel: 1000,
                max_depth: 50,
                output_filename: "cornell_box_glass.png".to_string(),
            };
            cornell_box_with_glass_sphere(config);
        }
        Some("final") => {
            let config = FinalSceneConfig {
                image_width: 800,
                samples_per_pixel: 5000,
                max_depth: 75,
                output_filename: "final_scene.png".to_string(),
            };
            final_scene_next_week(config);
        }
        Some("texture") => {
            // 纹理展示场景
            let config = FinalSceneConfig {
                image_width: 1024,
                samples_per_pixel: 1000,
                max_depth: 75,
                output_filename: "texture_showcase.png".to_string(),
            };
            scenes::final_scene::texture_showcase_scene(config);
        }
        Some("quick") => {
            // 快速测试版本
            let config = FinalSceneConfig {
                image_width: 400,
                samples_per_pixel: 100,
                max_depth: 20,
                output_filename: "quick_test.png".to_string(),
            };
            final_scene_next_week(config);
        }
        _ => {
            eprintln!("用法: {} [cornell|final|texture|quick]", args[0]);
            eprintln!("  cornell - 康奈尔盒子场景");
            eprintln!("  final   - 最终复杂场景");
            eprintln!("  texture - 纹理展示场景");
            eprintln!("  quick   - 快速测试场景");
        }
    }
}
