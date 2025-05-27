pub mod dielectric;
pub mod diffuse_light;
pub mod isotropic;
pub mod lambertian;
pub mod material;
pub mod metal;
pub mod texture;

// 重新导出主要类型
pub use dielectric::Dielectric;
pub use diffuse_light::DiffuseLight;
pub use lambertian::Lambertian;
pub use material::*;
pub use metal::Metal;
