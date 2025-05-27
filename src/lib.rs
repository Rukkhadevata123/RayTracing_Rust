//! # Ray Tracing Rest of Life
//!
//! 基于《Ray Tracing: The Rest of Your Life》的Rust实现
//!
//! 本项目实现了高级光线追踪技术，包括：
//! - 重要性采样
//! - 概率密度函数（PDF）
//! - 蒙特卡洛方法优化
//! - 体积渲染
//! - BVH加速结构

pub mod ray_tracing;
pub mod scenes;

// 重新导出主要模块
pub use ray_tracing::*;
pub use scenes::*;
