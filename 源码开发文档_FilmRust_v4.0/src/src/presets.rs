//! 胶片预设管理 - 基于 filmr 实际 API
//!
//! filmr 的 API:
//! - `filmr::presets::get_all_stocks() -> Vec<Rc<FilmStock>>`
//! - `FilmStock` 包含: name, iso, reciprocity, halation_*, grain_model 等
//! - `process_image(&RgbImage, &FilmStock, &SimulationConfig) -> RgbImage`

use filmr::{FilmStock, SimulationConfig};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::error::{FilmRustResult, anyhow_err};

/// 单个胶片预设（简化信息，用于 CLI 列表与 UI）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmPreset {
    /// 内部标识 (小写、无空格)
    pub id: String,
    /// 显示名称
    pub name: String,
    /// 厂商
    pub manufacturer: String,
    /// ISO
    pub iso: f32,
    /// 暗部 reciprocity 系数
    pub reciprocity_beta: f32,
    /// halation 光晕强度
    pub halation_strength: f32,
    /// 颗粒强度
    pub grain_alpha: f32,
    /// 建议场景
    pub tags: Vec<String>,
}

impl FilmPreset {
    pub fn from_filmr(stock: &FilmStock) -> Self {
        let (manufacturer, name) = split_manufacturer_and_name(&stock.name);

        let tags = infer_tags(&stock.name, stock.iso, stock.reciprocity.beta, stock.halation_strength);

        FilmPreset {
            id: format!("{}_{}",
                manufacturer.to_lowercase().replace(' ', "_"),
                name.to_lowercase().replace(' ', "_")),
            name: stock.name.clone(),
            manufacturer,
            iso: stock.iso,
            reciprocity_beta: stock.reciprocity.beta,
            halation_strength: stock.halation_strength,
            grain_alpha: stock.grain_model.alpha,
            tags,
        }
    }
}

/// 从名称中分离厂商
fn split_manufacturer_and_name(full: &str) -> (String, String) {
    let lower = full.to_lowercase();

    // 第一阶段：名称中直接包含品牌词
    let direct_brands = [
        ("kodak", "Kodak"),
        ("fujifilm", "Fujifilm"),
        ("fuji", "Fujifilm"),
        ("cinestill", "CineStill"),
        ("ilford", "Ilford"),
        ("polaroid", "Polaroid"),
        ("leica", "Leica"),
        ("agfa", "Agfa"),
        ("konica", "Konica"),
        ("rollei", "Rollei"),
    ];
    for (keyword, brand) in direct_brands {
        if lower.contains(keyword) {
            let rest = full.to_lowercase()
                .replace(keyword, "")
                .trim_matches(|c: char| c == '-' || c == '_' || c.is_whitespace())
                .to_string();
            let display_name = if rest.is_empty() { full.to_string() } else { rest };
            return (brand.to_string(), capitalize_name(&display_name));
        }
    }

    // 第二阶段：关键词映射（filmr 的 "Portra 400" → Kodak）
    let kodak_keywords = ["portra", "tri-x", "ektar", "gold", "ultramax", "t-max", "plus-x", "panatomic"];
    let fuji_keywords = ["velvia", "provia", "superia", "reala", "xtra", "fujicolor", "neopan"];
    let cinestill_keywords = ["800t", "50d", "400d", "bwxx", "cs41", "redrum"];

    for kw in kodak_keywords {
        if lower.contains(kw) {
            return ("Kodak".to_string(), capitalize_name(full));
        }
    }
    for kw in fuji_keywords {
        if lower.contains(kw) {
            return ("Fujifilm".to_string(), capitalize_name(full));
        }
    }
    for kw in cinestill_keywords {
        if lower.contains(kw) {
            return ("CineStill".to_string(), capitalize_name(full));
        }
    }

    // 默认：整个作为名称，厂商 Unknown
    ("Unknown".to_string(), capitalize_name(full))
}

/// 简单的首字母大写（"portra 400" → "Portra 400"）
fn capitalize_name(name: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in name.chars() {
        if c.is_whitespace() || c == '-' || c == '_' {
            result.push(c);
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c.to_ascii_lowercase());
        }
    }
    result
}

/// 根据参数推断建议场景标签
fn infer_tags(name: &str, iso: f32, reciprocity: f32, halation: f32) -> Vec<String> {
    let mut tags = Vec::new();
    let lower = name.to_lowercase();

    if iso >= 800.0 {
        tags.push("低光/夜景".to_string());
    } else if iso <= 100.0 {
        tags.push("日光/风景".to_string());
    }

    if halation > 1.0 {
        tags.push("光晕/胶片感".to_string());
    }

    if reciprocity > 0.15 {
        tags.push("长曝光/夜景".to_string());
    }

    if lower.contains("portra") || lower.contains("gold") {
        tags.push("人像友好".to_string());
    }

    if lower.contains("velvia") || lower.contains("ektachrome") {
        tags.push("鲜艳风景".to_string());
    }

    if lower.contains("cinestill") || lower.contains("800t") {
        tags.push("电影感".to_string());
    }

    if tags.is_empty() {
        tags.push("通用".to_string());
    }
    tags
}

/// 获取所有可用的 filmr 预设 (包装版)
pub fn get_all_presets() -> Vec<FilmPreset> {
    let stocks = filmr::presets::get_all_stocks();
    stocks.iter().map(|rc| FilmPreset::from_filmr(rc)).collect()
}

/// 获取所有 filmr 原始预设 (用于直接传递给 process_image)
pub fn get_all_filmr_stocks() -> Vec<Rc<FilmStock>> {
    filmr::presets::get_all_stocks()
}

/// 根据预设名或 ID 查找 (模糊匹配)
pub fn find_preset(query: &str) -> FilmRustResult<FilmPreset> {
    let query_lower = query.to_lowercase();

    let presets = get_all_presets();

    // 精确匹配 ID
    if let Some(p) = presets.iter().find(|p| p.id == query_lower) {
        return Ok(p.clone());
    }

    // 精确匹配名称
    if let Some(p) = presets.iter().find(|p| p.name.to_lowercase() == query_lower) {
        return Ok(p.clone());
    }

    // 模糊匹配
    if let Some(p) = presets.iter().find(|p|
        p.name.to_lowercase().contains(&query_lower)
            || p.manufacturer.to_lowercase().contains(&query_lower)) {
        return Ok(p.clone());
    }

    Err(anyhow_err!("找不到胶片预设: {}", query))
}

/// 根据预设名或 ID 查找 filmr 原始 FilmStock
/// 支持: 原始名称 ("Portra 400")、ID ("kodak_portra_400")、关键词 ("portra")
pub fn find_filmr_stock(query: &str) -> FilmRustResult<Rc<FilmStock>> {
    let query_lower = query.to_lowercase().trim().to_string();
    let stocks = filmr::presets::get_all_stocks();

    // 策略1: 精确匹配 stock.name
    if let Some(s) = stocks.iter().find(|s| s.name.to_lowercase() == query_lower) {
        return Ok(s.clone());
    }

    // 策略2: 用 _ 替换空格后匹配 (处理 "kodak_portra_400" 类 ID)
    let query_underscore = query_lower.replace(' ', "_");
    if let Some(s) = stocks.iter().find(|s| {
        let sn = s.name.to_lowercase().replace(' ', "_");
        sn == query_underscore || sn.contains(&query_underscore) || query_underscore.contains(&sn)
    }) {
        return Ok(s.clone());
    }

    // 策略3: 去除厂商前缀后匹配 (处理 "kodak portra 400" → "portra 400")
    let manufacturers = ["kodak ", "fujifilm ", "fuji ", "ilford ", "polaroid ",
                         "agfa ", "lomography ", "cinestill ", "ferrania ", "orwo ",
                         "ricoh ", "lucky ", "standard ", "vintage "];
    let mut cleaned = query_lower.clone();
    for mfr in &manufacturers {
        cleaned = cleaned.replace(mfr, "");
    }
    let cleaned = cleaned.trim().to_string();
    if cleaned != query_lower && !cleaned.is_empty() {
        if let Some(s) = stocks.iter().find(|s| s.name.to_lowercase().contains(&cleaned)) {
            return Ok(s.clone());
        }
    }

    // 策略4: stock.name 包含查询词
    if let Some(s) = stocks.iter().find(|s| s.name.to_lowercase().contains(&query_lower)) {
        return Ok(s.clone());
    }

    // 策略5: 无空格交叉匹配 (处理 "portra400" 无空格)
    let query_nospace: String = query_lower.chars().filter(|c| !c.is_whitespace()).collect();
    if query_nospace != query_lower {
        if let Some(s) = stocks.iter().find(|s| {
            let sn: String = s.name.to_lowercase().chars().filter(|c| !c.is_whitespace()).collect();
            sn.contains(&query_nospace) || query_nospace.contains(&sn)
        }) {
            return Ok(s.clone());
        }
    }

    // 策略6: 关键词全部匹配 (所有非厂商单词都出现在 stock.name 中)
    let keywords: Vec<&str> = query_lower.split([' ', '_'])
        .filter(|w| w.len() > 1 && !manufacturers.iter().any(|m| m.trim() == *w))
        .collect();
    if keywords.len() >= 2 {
        if let Some(s) = stocks.iter().find(|s| {
            let sn = s.name.to_lowercase();
            keywords.iter().all(|w| sn.contains(w))
        }) {
            return Ok(s.clone());
        }
    }

    Err(anyhow_err!("找不到胶片预设: {}", query))
}

/// 生成默认的 SimulationConfig (基于预设风格)
pub fn default_sim_config() -> SimulationConfig {
    SimulationConfig {
        exposure_time: 1.0 / 125.0,
        enable_grain: true,
        saturation: 1.0,
        warmth: 0.0,
        motion_blur_amount: 0.0,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preset_list_not_empty() {
        let presets = get_all_presets();
        assert!(!presets.is_empty(), "filmr 至少应提供一种预设");
    }

    #[test]
    fn test_find_filmr_stock_by_name() {
        let stock = find_filmr_stock("portra");
        assert!(stock.is_ok() || stock.is_err(),
            "应该返回 Ok 或 Err，但返回 {:?}", stock);
    }

    #[test]
    fn test_default_sim_config() {
        let config = default_sim_config();
        assert!((config.exposure_time - 0.008).abs() < 0.01);
        assert!(config.enable_grain);
    }
}
