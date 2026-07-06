//! 胶片预设管理 - 基于 filmr 实际 API
//!
//! filmr 的 API:
//! - `filmr::presets::get_all_stocks() -> Vec<Rc<FilmStock>>`
//! - `FilmStock` 包含: name, iso, reciprocity, halation_*, grain_model 等
//! - `process_image(&RgbImage, &FilmStock, &SimulationConfig) -> RgbImage`

use filmr::{FilmStock, SimulationConfig};
use serde::{Deserialize, Serialize};
use std::rc::Rc;

use crate::custom_presets;
use crate::error::{anyhow_err, FilmRustResult};

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
    /// 色调描述（一句话概括风格）
    pub description: String,
    /// 建议场景
    pub tags: Vec<String>,
    /// 默认色温偏移（模拟扫片校色，负=偏蓝，正=偏暖）
    pub default_warmth: f32,
    /// 默认色调偏移（模拟扫片校色，正=偏品红，负=偏绿）
    pub default_tint: f32,
    /// 默认饱和度（模拟扫片校色，1.0=中性）
    pub default_saturation: f32,
    /// 肤色优化默认值（增强版）
    pub skin_remove_yellow: f32, // 0~100 去黄
    pub skin_reduce_green: f32, // 0~100 减绿
    pub skin_add_pink: f32,     // 0~100 加粉
    pub skin_add_red: f32,      // 0~100 加红
    pub skin_brightness: f32,   // -50~+50 亮度微调
    /// 色调分离默认值
    pub split_hh: f32, // 高光色相 0~360
    pub split_hs: f32,          // 高光饱和 0~100
    pub split_sh: f32,          // 阴影色相 0~360
    pub split_ss: f32,          // 阴影饱和 0~100
    pub split_balance: f32,     // -100~+100
    pub split_strength: f32,    // 0~100
    /// 锐化默认值
    pub sharp_amount: f32, // 0~100
}

impl FilmPreset {
    pub fn from_filmr(stock: &FilmStock) -> Self {
        let (manufacturer, name) = split_manufacturer_and_name(&stock.name);

        let tags = infer_tags(
            &stock.name,
            stock.iso,
            stock.reciprocity.beta,
            stock.halation_strength,
        );
        let description = infer_description(&stock.name, &manufacturer, stock.iso);
        // filmr 的 color_matrix 已自带色罩补偿，不做额外默认校色

        FilmPreset {
            id: format!(
                "{}_{}",
                manufacturer.to_lowercase().replace(' ', "_"),
                name.to_lowercase().replace(' ', "_")
            ),
            name: stock.name.clone(),
            manufacturer,
            iso: stock.iso,
            reciprocity_beta: stock.reciprocity.beta,
            halation_strength: stock.halation_strength,
            grain_alpha: stock.grain_model.alpha,
            description,
            tags,
            // 色调/肤色/色调分离/锐化全部归零，用户手动启用
            default_warmth: 0.0,
            default_tint: 0.0,
            default_saturation: 1.0,
            skin_remove_yellow: 0.0,
            skin_reduce_green: 0.0,
            skin_add_pink: 0.0,
            skin_add_red: 0.0,
            skin_brightness: 0.0,
            split_hh: 0.0,
            split_hs: 0.0,
            split_sh: 0.0,
            split_ss: 0.0,
            split_balance: 0.0,
            split_strength: 0.0,
            sharp_amount: 0.0,
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
            let rest = full
                .to_lowercase()
                .replace(keyword, "")
                .trim_matches(|c: char| c == '-' || c == '_' || c.is_whitespace())
                .to_string();
            let display_name = if rest.is_empty() {
                full.to_string()
            } else {
                rest
            };
            return (brand.to_string(), capitalize_name(&display_name));
        }
    }

    // 第二阶段：关键词映射（filmr 的 "Portra 400" → Kodak）
    let kodak_keywords = [
        "portra",
        "tri-x",
        "ektar",
        "gold",
        "ultramax",
        "t-max",
        "plus-x",
        "panatomic",
    ];
    let fuji_keywords = [
        "velvia",
        "provia",
        "superia",
        "reala",
        "xtra",
        "fujicolor",
        "neopan",
    ];
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

    // 按 ISO 分
    if iso >= 800.0 {
        tags.push("弱光/夜景".to_string());
    } else if iso <= 100.0 {
        tags.push("日光/细颗粒".to_string());
    } else {
        tags.push("通用感光".to_string());
    }

    // 按胶片类型分
    if lower.contains("portra") {
        tags.push("人像肤色".to_string());
    } else if lower.contains("gold") {
        tags.push("暖调日常".to_string());
    } else if lower.contains("ektar") || lower.contains("velvia") {
        tags.push("高饱和风景".to_string());
    } else if lower.contains("ektachrome") || lower.contains("provia") {
        tags.push("正片/真实色彩".to_string());
    } else if lower.contains("tri-x") || lower.contains("hp5") {
        tags.push("高反差黑白".to_string());
    } else if lower.contains("delta") || lower.contains("fp4") || lower.contains("pan") {
        tags.push("细颗粒黑白".to_string());
    } else if lower.contains("cinestill") || lower.contains("800t") {
        tags.push("电影感/光晕".to_string());
    } else if lower.contains("superia") || lower.contains("fujicolor") {
        tags.push("日系清新".to_string());
    } else if lower.contains("lomo") {
        tags.push("创意/lomo".to_string());
    } else if lower.contains("polaroid") {
        tags.push("拍立得/怀旧".to_string());
    } else if lower.contains("solaris") || lower.contains("ferrania") {
        tags.push("复古暖调".to_string());
    }

    if halation > 1.0 {
        tags.push("强光晕".to_string());
    }
    if reciprocity > 0.15 {
        tags.push("长曝光".to_string());
    }
    if tags.is_empty() {
        tags.push("通用".to_string());
    }
    tags
}

/// 推断胶片的色调风格描述（静态查找表 + 关键词匹配）
fn infer_description(name: &str, manufacturer: &str, iso: f32) -> String {
    const DESC_TABLE: &[(&str, &str)] = &[
        (
            "portra 800",
            "暖调肤色·宽容度高·低光柔和 — 婚礼/室内/弱光人像首选",
        ),
        (
            "portra 400",
            "暖调肤色·低反差·高光滚降柔和 — 经典人像/婚礼万能卷",
        ),
        (
            "portra 160",
            "极细颗粒·中性肤色·高宽容度 — 商业人像/时尚/棚拍",
        ),
        ("gold 200", "暖金黄调·复古感·日常亲切 — 旅行/家庭/生活记录"),
        ("ektar 100", "极高饱和·锐利细腻·色彩浓烈 — 风景/建筑/产品"),
        (
            "ektachrome 100 vs",
            "高饱和正片·鲜艳浓郁·冷暖分明 — 自然风光/花卉/户外",
        ),
        (
            "ektachrome 100",
            "正片·色彩真实·中高饱和·冷调 — 风景/产品/商业摄影",
        ),
        (
            "kodachrome 64",
            "经典正片·暖调·红色突出·时代感 — 旅行/纪实/街拍",
        ),
        (
            "kodachrome 25",
            "极细颗粒·低感·色彩浓郁 — 静物/风景/阳光充足",
        ),
        ("tri-x 400", "经典粗颗粒·高反差·强戏剧感 — 街拍/纪实/新闻"),
        ("plus-x 125", "细颗粒·中反差·影调丰富 — 风景/静物/人像"),
        (
            "velvia 100",
            "极高饱和正片·绿色突出·风光之王 — 自然风光/花草/户外",
        ),
        (
            "velvia 50",
            "超饱和正片·极细颗粒·低感 — 风光/静物/三脚架拍摄",
        ),
        (
            "provia 100f",
            "正片·色彩准确·中饱和·真实还原 — 风景/产品/商业",
        ),
        ("superia 400", "冷调偏绿·日系清新·通用 — 日常/旅行/街头"),
        ("superia 200", "冷调柔和·细颗粒·清淡 — 日常/人像/旅行"),
        ("superia 100", "冷调·极细颗粒·低感 — 风景/静物/阳光充足"),
        ("fujicolor 100", "冷调·低感·细颗粒·清新 — 风景/静物/户外"),
        ("xtra 400", "暖调·消费级·通用 — 日常/家庭/旅行"),
        ("neopan 400", "黑白·中反差·影调平滑 — 街拍/纪实/通用"),
        ("neopan 100", "黑白·细颗粒·中反差 — 风景/建筑/静物"),
        (
            "cinestill 800t",
            "钨丝灯色温·冷蓝调·红晕光·电影感 — 夜景/街拍/霓虹灯",
        ),
        (
            "cinestill 50d",
            "日光平衡·细颗粒·电影感·柔和 — 白天/风景/人像",
        ),
        ("hp5 plus 400", "经典黑白·中颗粒·高宽容度 — 街拍/纪实/通用"),
        ("fp4 plus 125", "细颗粒·丰富影调·中反差 — 风景/人像/静物"),
        ("delta 3200", "超高感·粗颗粒·强氛围·暗光 — 音乐会/夜景/情绪"),
        ("delta 100", "极细颗粒·高锐度·中反差 — 风景/建筑/静物"),
        ("xp2 super", "C41黑白·平滑影调·染料片 — 通用/人像"),
        ("pan f plus", "极细颗粒·高反差·高锐度 — 风景/静物/微距"),
        ("polaroid 600", "暖调·低反差·拍立得风格 — 日常/怀旧/创意"),
        ("sx-70", "经典拍立得·柔焦感·怀旧色调 — 怀旧/艺术/创意"),
        (
            "lomochrome purple",
            "紫色调·超现实色彩·创意 — 创意/艺术/实验",
        ),
        (
            "lomography color chrome",
            "高饱和·强对比·lomo风格 — 创意/街头/旅行",
        ),
        ("solaris 400", "复古暖调·意式风格·颗粒感 — 怀旧/旅行/日常"),
        ("solaris 100", "复古暖调·细颗粒·阳光 — 怀旧/风景/旅行"),
        ("orwo un54", "电影黑白·德味影调·中反差 — 电影/纪实/街拍"),
        ("orwo un64", "电影黑白·细颗粒·低感 — 电影/纪实/静物"),
        ("gr street night", "夜景专用·高反差·冷调 — 夜景/街拍/城市"),
        ("standard daylight", "标准日光·中性参考 — 测试/基准"),
        ("vista 400", "暖调·中颗粒·复古感 — 日常/旅行/怀旧"),
        ("vista 200", "暖调柔和·日用 — 日常/人像"),
        ("vista 100", "暖调·细颗粒·低感 — 风景/户外"),
        ("apx 400", "黑白·中颗粒·高宽容度 — 街拍/纪实"),
        ("apx 100", "黑白·细颗粒·高锐度 — 风景/静物"),
        ("precisa 100", "正片·色彩鲜艳·细颗粒 — 风景/户外"),
        ("scala 200", "黑白反转片·高反差·独特 — 创意/艺术"),
        ("optima 200", "暖调柔和·中感光度 — 日常/人像"),
        (
            "ultramax 400",
            "暖调高饱和·浓郁色彩·消费卷王 — 旅行/家庭/街拍",
        ),
        (
            "pro 400h",
            "冷蓝阴影·暖粉高光·日系人像王 — 人像/婚礼/生活记录",
        ),
        (
            "pro_400h",
            "冷蓝阴影·暖粉高光·日系人像王 — 人像/婚礼/生活记录",
        ),
        (
            "natura 1600",
            "高速月光·暖调浓郁·青绿阴影·独特颗粒 — 夜景/室内/街拍",
        ),
    ];
    let lower = name.to_lowercase();
    for &(keyword, desc) in DESC_TABLE {
        if lower.contains(keyword) {
            return desc.to_string();
        }
    }
    format!("{} · ISO {} — 通用胶片风格", manufacturer, iso as i32)
}

/// 获取所有可用的 filmr 预设 (包装版)
pub fn get_all_presets() -> Vec<FilmPreset> {
    let stocks = get_all_filmr_stocks();
    stocks.iter().map(|rc| FilmPreset::from_filmr(rc)).collect()
}

/// 获取所有 filmr 原始预设 (用于直接传递给 process_image)
pub fn get_all_filmr_stocks() -> Vec<Rc<FilmStock>> {
    let mut stocks = custom_presets::get_custom_stocks();
    stocks.extend(filmr::presets::get_all_stocks().into_iter().filter(|s| {
        let n = s.name.to_lowercase();
        // 移除已知画质不稳的预设
        !n.contains("c200") && !n.contains("colorplus 200")
    }));
    stocks
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
    if let Some(p) = presets
        .iter()
        .find(|p| p.name.to_lowercase() == query_lower)
    {
        return Ok(p.clone());
    }

    // 模糊匹配
    if let Some(p) = presets.iter().find(|p| {
        p.name.to_lowercase().contains(&query_lower)
            || p.manufacturer.to_lowercase().contains(&query_lower)
    }) {
        return Ok(p.clone());
    }

    Err(anyhow_err!("找不到胶片预设: {}", query))
}

/// 根据预设名或 ID 查找 filmr 原始 FilmStock
/// 支持: 原始名称 ("Portra 400")、ID ("kodak_portra_400")、关键词 ("portra")
pub fn find_filmr_stock(query: &str) -> FilmRustResult<Rc<FilmStock>> {
    let query_lower = query.to_lowercase().trim().to_string();
    let stocks = get_all_filmr_stocks();

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
    let manufacturers = [
        "kodak ",
        "fujifilm ",
        "fuji ",
        "ilford ",
        "polaroid ",
        "agfa ",
        "lomography ",
        "cinestill ",
        "ferrania ",
        "orwo ",
        "ricoh ",
        "lucky ",
        "standard ",
        "vintage ",
    ];
    let mut cleaned = query_lower.clone();
    for mfr in &manufacturers {
        cleaned = cleaned.replace(mfr, "");
    }
    let cleaned = cleaned.trim().to_string();
    if cleaned != query_lower && !cleaned.is_empty() {
        if let Some(s) = stocks
            .iter()
            .find(|s| s.name.to_lowercase().contains(&cleaned))
        {
            return Ok(s.clone());
        }
    }

    // 策略4: stock.name 包含查询词
    if let Some(s) = stocks
        .iter()
        .find(|s| s.name.to_lowercase().contains(&query_lower))
    {
        return Ok(s.clone());
    }

    // 策略5: 无空格交叉匹配 (处理 "portra400" 无空格)
    let query_nospace: String = query_lower.chars().filter(|c| !c.is_whitespace()).collect();
    if query_nospace != query_lower {
        if let Some(s) = stocks.iter().find(|s| {
            let sn: String = s
                .name
                .to_lowercase()
                .chars()
                .filter(|c| !c.is_whitespace())
                .collect();
            sn.contains(&query_nospace) || query_nospace.contains(&sn)
        }) {
            return Ok(s.clone());
        }
    }

    // 策略6: 关键词全部匹配 (所有非厂商单词都出现在 stock.name 中)
    let keywords: Vec<&str> = query_lower
        .split([' ', '_'])
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
        // portra 是 filmr 内置预设，应能找到
        assert!(
            find_filmr_stock("portra").is_ok(),
            "portra 是已知预设，应能匹配成功"
        );
    }

    #[test]
    fn test_default_sim_config() {
        let config = default_sim_config();
        assert!((config.exposure_time - 0.008).abs() < 0.01);
        assert!(config.enable_grain);
    }
}
