//! filmrust-studio - 物理级胶片模拟工具核心库
//!
//! 模块结构:
//! - `error`: 统一错误类型 (`FilmRustError`, `FilmRustResult`)
//! - `presets`: 胶片预设管理 (基于 filmr)
//! - `ps_jsx`: Photoshop .jsx 脚本生成器
//!
//! 依赖参考:
//! - [rust-core-lib](file:///f:/trae-cn/.trae/templates/rust-core-lib/src/lib.rs) 的错误处理与路径处理风格
//! - [ps-extendscript-exe-integration](file:///f:/trae-cn/.trae/skills/ps-extendscript-exe-integration.md) 的 PS 脚本生成规范

pub mod error;
pub mod layers;
pub mod presets;
pub mod ps_jsx;

pub use error::{FilmRustError, FilmRustResult};
pub use presets::{FilmPreset, default_sim_config, find_filmr_stock, find_preset, get_all_filmr_stocks, get_all_presets};
pub use ps_jsx::{JsxConfig, JsxGenerator, generate_jsx_for_preset};

/// 对 RGB 图像应用色调倾斜（绿←→品红）
/// tint < 0 = 偏绿, tint > 0 = 偏品红
pub fn apply_tint_to_rgb(img: &image::RgbImage, tint: f32) -> image::RgbImage {
    if tint.abs() < 0.01 { return img.clone(); }
    let amount = tint * 0.12;
    let (rm, gm, bm) = if amount > 0.0 {
        (1.0 + amount, 1.0 - amount * 0.6, 1.0 + amount)
    } else {
        let a = amount.abs();
        (1.0 - a * 0.5, 1.0 + a, 1.0 - a * 0.5)
    };
    let mut out = img.clone();
    for pixel in out.pixels_mut() {
        pixel[0] = ((pixel[0] as f32 / 255.0 * rm).clamp(0.0, 1.0) * 255.0) as u8;
        pixel[1] = ((pixel[1] as f32 / 255.0 * gm).clamp(0.0, 1.0) * 255.0) as u8;
        pixel[2] = ((pixel[2] as f32 / 255.0 * bm).clamp(0.0, 1.0) * 255.0) as u8;
    }
    out
}

use filmr::{FilmStock, SimulationConfig};
use image::RgbImage;

/// 应用胶片效果到图像 (核心 API)
///
/// # Arguments
/// - `input`: 输入 RGB 图像
/// - `stock`: filmr 胶片预设 (来自 `find_filmr_stock`)
/// - `config`: 模拟参数 (来自 `default_sim_config`)
pub fn apply_film(
    input: &RgbImage,
    stock: &FilmStock,
    config: &SimulationConfig,
) -> FilmRustResult<RgbImage> {
    Ok(filmr::process_image(input, stock, config))
}

/// 便捷的单张图像处理
pub fn process_file(
    input_path: &std::path::Path,
    output_path: &std::path::Path,
    preset_query: &str,
    config: &SimulationConfig,
) -> FilmRustResult<()> {
    // 检查输入
    if !input_path.exists() {
        return Err(error::anyhow_err!("输入文件不存在: {:?}", input_path));
    }

    // 查找预设
    let stock = find_filmr_stock(preset_query)?;

    // 读取图像
    let img = image::open(input_path)
        .map_err(|e| error::anyhow_err!("无法读取图像: {} ({} 是不是受支持的格式?)", e,
            input_path.extension()
                .and_then(|s| s.to_str())
                .unwrap_or("unknown")))?;
    let rgb = img.to_rgb8();

    // 应用效果
    let result = apply_film(&rgb, &stock, config)?;

    // 保存
    result.save(output_path)
        .map_err(|e| error::anyhow_err!("保存图像失败: {}", e))?;

    Ok(())
}
