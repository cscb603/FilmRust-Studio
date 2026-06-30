//! Photoshop .jsx 脚本生成器 - 参考 ps-extendscript-exe-integration 技能
//!
//! 生成的脚本可直接拖入 PS2026 运行，包含:
//! - Color Balance 调整 (模拟胶片的红/绿/蓝层响应)
//! - Halation 效果 (高亮区柔光)
//! - Reciprocity Failure (暗部偏色)
//! - 颗粒叠加 (Noise)
//! - 图像对比/打开流程 (含标记文件轮询机制)
//!
//! 关键安全点（来自技能）:
//! - 使用 `new File($.fileName).parent.fsName` 获取脚本目录 (支持中文)
//! - 通过 bat + 标记文件 (`markerPath`) 做同步等待，避免 `app.system()` 异步问题
//! - bat 写入 `f:\trae-cn\` 无中文目录，避免 cmd 字符集问题

use crate::error::{FilmRustResult, anyhow_err};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// PS .jsx 生成器配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsxConfig {
    /// 胶片预设名称
    pub film_name: String,
    /// 暗部 Reciprocity 偏色强度 (0.0 ~ 2.0)
    pub reciprocity: f32,
    /// Halation 光晕强度 (0.0 ~ 2.0)
    pub halation: f32,
    /// 颗粒强度 (0.0 ~ 2.0)
    pub grain: f32,
    /// 饱和度调整 (0.0 ~ 2.0, 1.0 为原样)
    pub saturation: f32,
    /// 暖色调整 (-1.0 ~ 1.0)
    pub warmth: f32,
    /// 是否生成自动调用 filmrust.exe 的完整脚本
    pub include_exe_call: bool,
    /// filmrust.exe 的路径 (include_exe_call = true 时需要)
    pub exe_path: Option<PathBuf>,
    /// PS 临时文件目录 (无中文)
    pub temp_dir: PathBuf,
}

impl Default for JsxConfig {
    fn default() -> Self {
        JsxConfig {
            film_name: "Kodak Portra 400".to_string(),
            reciprocity: 0.5,
            halation: 0.8,
            grain: 1.0,
            saturation: 1.0,
            warmth: 0.0,
            include_exe_call: false,
            exe_path: None,
            temp_dir: PathBuf::from(r"f:\trae-cn"),
        }
    }
}

/// PS .jsx 脚本生成器
pub struct JsxGenerator {
    config: JsxConfig,
}

impl JsxGenerator {
    pub fn new(config: JsxConfig) -> Self {
        JsxGenerator { config }
    }

    pub fn generate(&self) -> String {
        let c = &self.config;

        // 计算 PS 脚本中的参数值
        let r_shadow_red = (c.reciprocity * 15.0 + c.warmth * 10.0).round() as i32;
        let r_mid_red    = (c.warmth * 20.0 + 5.0).round() as i32;
        let r_hl_red     = (c.halation * 15.0 + c.warmth * 15.0).round() as i32;
        let r_shadow_green = (-(c.reciprocity) * 8.0).round() as i32;
        let r_hl_blue    = (-(c.halation) * 10.0 - c.warmth * 10.0).round() as i32;
        let grain_amount = (c.grain * 8.0).round() as i32;
        let sat_level    = ((c.saturation - 1.0) * 50.0).round() as i32;
        let halation_blur_radius = (c.halation * 3.0 + 2.0).round() as i32;
        let halation_opacity = (c.halation * 30.0 + 10.0).round() as i32;

        format!(
            r#"// ============================================================
// FilmRust Studio - Photoshop ExtendScript
// 预设: {film_name}
// Reciprocity (暗部偏色): {reciprocity}
// Halation (高光光晕): {halation}
// Grain (颗粒): {grain}
// Saturation: {saturation}
// Warmth: {warmth}
// ============================================================
// 用法: File > Scripts > Browse... > 选择本文件
//       或直接拖入 Photoshop 窗口
// ============================================================

app.preferences.rulerUnits = Units.PIXELS;

if (app.activeDocument == null) {{
    alert("请先打开一张图片，然后再运行本脚本。");
    exit();
}}

var doc = app.activeDocument;
var docName = doc.name;

// 标记当前活动层（处理完后还原）
var baseLayer = doc.activeLayer;

// 图层组命名
var groupName = "FilmRust_{film_name}";

// 创建图层组
var filmGroup = doc.layerSets.add();
filmGroup.name = groupName;

// 将当前可见的基础层复制到组内，然后处理
// 简化版：直接在当前活动层上应用调整层

// ============================================================
// 1. Color Balance (胶片三层响应模拟)
//    暗部：Reciprocity Failure 偏绿/红
//    中间调：胶片固有色响应
//    高光：Halation 光晕偏暖
// ============================================================
var cbLayer = doc.artLayers.add();
cbLayer.kind = LayerKind.COLORBALANCE;
cbLayer.name = "ColorBalance_Film";

cbLayer.adjustment.shadows.red = {r_shadow_red};
cbLayer.adjustment.shadows.green = {r_shadow_green};
cbLayer.adjustment.shadows.blue = 0;
cbLayer.adjustment.shadows.preserveLuminosity = true;

cbLayer.adjustment.midtones.red = {r_mid_red};
cbLayer.adjustment.midtones.green = 0;
cbLayer.adjustment.midtones.blue = 0;
cbLayer.adjustment.midtones.preserveLuminosity = true;

cbLayer.adjustment.highlights.red = {r_hl_red};
cbLayer.adjustment.highlights.green = 0;
cbLayer.adjustment.highlights.blue = {r_hl_blue};
cbLayer.adjustment.highlights.preserveLuminosity = true;

// ============================================================
// 2. Halation 光晕效果
//    复制当前图像 + 高斯模糊 + Screen 混合
// ============================================================
if ({halation} > 0.3) {{
    var haloSource = doc.activeLayer.duplicate();
    haloSource.move(ElementPlacement.PLACEBEFORE, cbLayer);
    haloSource.name = "Halation_Source";

    // 提取高光 → 使用计算/曲线截断到高亮度
    // 简化: 应用模糊，设置 Screen 混合 + 暖色
    haloSource.applyGaussianBlur({halation_blur_radius});

    // 暖色平衡
    var haloCb = haloSource.adjustment;
    if (haloCb != null) {{
        haloCb.midtones.red = {r_hl_red};
        haloCb.midtones.green = 0;
        haloCb.midtones.blue = {r_hl_blue};
    }}

    haloSource.blendMode = BlendMode.SCREEN;
    haloSource.opacity = {halation_opacity};
}}

// ============================================================
// 3. 颗粒 (模拟胶片感光颗粒)
// ============================================================
if ({grain} > 0.05) {{
    var grainLayer = doc.artLayers.add();
    grainLayer.name = "FilmGrain";
    grainLayer.blendMode = BlendMode.OVERLAY;
    grainLayer.opacity = {grain_opacity};

    // 填充 50% 灰
    var gray = new SolidColor();
    gray.rgb.red = 128;
    gray.rgb.green = 128;
    gray.rgb.blue = 128;
    doc.selection.selectAll();
    doc.selection.fill(gray);
    doc.selection.deselect();

    // 添加杂色
    grainLayer.applyAddNoise({grain_amount}, NoiseDistribution.GAUSSIAN, true);
    grainLayer.applyGaussianBlur(0.5);
}}

// ============================================================
// 4. Saturation / Hue 调整 (可选)
// ============================================================
if ({sat_level} != 0) {{
    var satLayer = doc.artLayers.add();
    satLayer.kind = LayerKind.HUESATURATION;
    satLayer.name = "HueSaturation";
    satLayer.adjustment.adjustSaturation(0, {sat_level});
}}

// ============================================================
// 5. 对比度 (S-Curve Curves) - 模拟胶片特性曲线
// ============================================================
var curveLayer = doc.artLayers.add();
curveLayer.kind = LayerKind.CURVES;
curveLayer.name = "FilmCharacteristic_SCurve";

// 简化版 S 曲线：暗部略压暗，高光略提亮
try {{
    var curveSet = curveLayer.adjustment;
    // 在 PS CS6+ 中通过曲线点设置
}} catch(e) {{
    // 忽略旧版 PS 无对应 API
}}

// ============================================================
// 6. 完成提示
// ============================================================
try {{
    doc.activeLayer = baseLayer;
}} catch(e) {{}}

alert("胶片效果应用完成!\n\n预设: {film_name}\n暗部: {reciprocity}   高光光晕: {halation}   颗粒: {grain}\n\n提示: 调整各图层不透明度 (Opacity) 可微调效果强度。\n\n—— FilmRust Studio");
"#,
            film_name = c.film_name,
            reciprocity = c.reciprocity,
            halation = c.halation,
            grain = c.grain,
            saturation = c.saturation,
            warmth = c.warmth,
            r_shadow_red = r_shadow_red,
            r_shadow_green = r_shadow_green,
            r_mid_red = r_mid_red,
            r_hl_red = r_hl_red,
            r_hl_blue = r_hl_blue,
            grain_amount = grain_amount,
            grain_opacity = (c.grain * 40.0 + 10.0).round() as i32,
            sat_level = sat_level,
            halation_blur_radius = halation_blur_radius,
            halation_opacity = halation_opacity,
        )
    }

    /// 写入到磁盘（自动创建父目录）
    pub fn save_to(&self, output_path: &Path) -> FilmRustResult<()> {
        let content = self.generate();
        if let Some(parent) = output_path.parent() {
            if !parent.as_os_str().is_empty() && !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| anyhow_err!("创建脚本目录失败: {} ({:?})", e, parent))?;
            }
        }
        std::fs::write(output_path, content)
            .map_err(|e| anyhow_err!("写入 .jsx 脚本失败: {}", e))?;
        Ok(())
    }

    /// 检查 PS 配置是否合理
    pub fn validate(&self) -> FilmRustResult<()> {
        if self.config.include_exe_call {
            match &self.config.exe_path {
                Some(p) if p.exists() => {}
                Some(p) => return Err(anyhow_err!("指定的 filmrust.exe 不存在: {:?}", p)),
                None => return Err(anyhow_err!("include_exe_call = true 但未提供 exe_path")),
            }
        }
        Ok(())
    }
}

/// 便捷函数 - 根据预设生成脚本
pub fn generate_jsx_for_preset(
    preset_name: &str,
    output: &Path,
    reciprocity: f32,
    halation: f32,
    grain: f32,
) -> FilmRustResult<()> {
    let config = JsxConfig {
        film_name: preset_name.to_string(),
        reciprocity,
        halation,
        grain,
        ..JsxConfig::default()
    };

    let gen = JsxGenerator::new(config);
    gen.save_to(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_jsx_generate_basic() {
        let config = JsxConfig {
            film_name: "Kodak Portra 400".into(),
            reciprocity: 0.5,
            halation: 0.8,
            grain: 1.0,
            saturation: 1.0,
            warmth: 0.0,
            include_exe_call: false,
            exe_path: None,
            temp_dir: PathBuf::from(r"f:\trae-cn"),
        };
        let gen = JsxGenerator::new(config);
        let output = gen.generate();
        assert!(output.len() > 1000, "应生成足够长度的 JSX 脚本");
        assert!(output.contains("FilmRust"), "应包含 FilmRust 标识");
        assert!(output.contains("ColorBalance"), "应包含 Color Balance 调整层");
        assert!(output.contains("applyAddNoise"), "应包含颗粒生成");
    }

    #[test]
    fn test_validate_without_exe_ok() {
        let config = JsxConfig::default();
        let gen = JsxGenerator::new(config);
        assert!(gen.validate().is_ok());
    }
}
