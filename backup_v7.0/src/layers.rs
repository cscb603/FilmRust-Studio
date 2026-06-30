//! 图层引擎 — 混合模式 + 10 种调整层 + 合成管线 + 缓存优化
//!
//! 设计原则:
//! - 胶片基底走 filmr 全管线（一次渲染，结果缓存）
//! - 调整层走纯像素运算（毫秒级，可实时刷新）
//! - SkinHSL / SplitTone / Sharp 为新增后处理层
//! - Sharp 仅用于最终输出，不参与实时预览

use image::RgbImage;

// ============================================================
// 混合模式
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub enum BlendMode {
    #[default]
    Normal,
    Multiply,
    Screen,
    Overlay,
    SoftLight,
    Color,
    Luminosity,
}

impl BlendMode {
    pub const ALL: &[BlendMode] = &[
        Self::Normal, Self::Multiply, Self::Screen,
        Self::Overlay, Self::SoftLight, Self::Color, Self::Luminosity,
    ];

    pub fn label(&self) -> &'static str {
        match self {
            Self::Normal => "Normal",
            Self::Multiply => "正片叠底",
            Self::Screen => "滤色",
            Self::Overlay => "叠加",
            Self::SoftLight => "柔光",
            Self::Color => "颜色",
            Self::Luminosity => "明度",
        }
    }
}

// ============================================================
// HSL 工具函数
// ============================================================

/// RGB → HSL，所有值在 [0,1] 范围
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let mx = r.max(g).max(b);
    let mn = r.min(g).min(b);
    let l = (mx + mn) * 0.5;
    if (mx - mn).abs() < 1e-6 {
        return (0.0, 0.0, l.clamp(0.0, 1.0));
    }
    let d = mx - mn;
    let denom = if l > 0.5 { 2.0 - mx - mn } else { mx + mn };
    let s = if denom.abs() < 1e-8 { 0.0 } else { d / denom };
    let h = if (mx - r).abs() < 1e-6 {
        (g - b) / d + (if g < b { 6.0 } else { 0.0 })
    } else if (mx - g).abs() < 1e-6 {
        (b - r) / d + 2.0
    } else {
        (r - g) / d + 4.0
    };
    ((h / 6.0).clamp(0.0, 1.0), s.clamp(0.0, 1.0), l.clamp(0.0, 1.0))
}

/// HSL → RGB，所有值在 [0,1] 范围
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s.abs() < 1e-6 {
        return (l, l, l);
    }
    let hue_to_rgb = |p: f32, q: f32, mut t: f32| -> f32 {
        if t < 0.0 { t += 1.0; }
        if t > 1.0 { t -= 1.0; }
        if t < 1.0 / 6.0 { p + (q - p) * 6.0 * t }
        else if t < 0.5 { q }
        else if t < 2.0 / 3.0 { p + (q - p) * (2.0 / 3.0 - t) * 6.0 }
        else { p }
    };
    let q = if l < 0.5 { l * (1.0 + s) } else { l + s - l * s };
    let p = 2.0 * l - q;
    (hue_to_rgb(p, q, h + 1.0 / 3.0),
     hue_to_rgb(p, q, h),
     hue_to_rgb(p, q, h - 1.0 / 3.0))
}

/// 计算某色相到目标色相的圆环距离（归一化到 [0,0.5]）
fn hue_distance(h: f32, target: f32) -> f32 {
    let d = (h - target).abs();
    if d > 0.5 { 1.0 - d } else { d }
}

/// 在某个色相范围上的软权重（三角窗函数）
fn hue_weight(h: f32, center: f32, half_width: f32) -> f32 {
    let d = hue_distance(h, center);
    if d >= half_width { 0.0 }
    else { 1.0 - d / half_width }
}

// ============================================================
// 调整层类型
// ============================================================

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LayerType {
    /// 胶片基底 — filmr 全管线渲染
    FilmBase {
        stock_id: String,
        strength: f32,
        grain: f32,
        auto_levels: bool,
    },
    /// 色彩 — 色温/色调/饱和度
    Color {
        warmth: f32,
        tint: f32,
        saturation: f32,
    },
    /// 曲线 — 对比度/高光/阴影
    Curves {
        contrast: f32,
        highlights: f32,
        shadows: f32,
    },
    /// 颗粒 — 胶片颗粒叠加
    Grain {
        amount: f32,
        size: f32,
    },
    /// 暗角/光晕 — 边缘压暗 + 高光扩散
    Vignette {
        strength: f32,
        halation: f32,
    },
    /// 漏光 — 彩色边缘渐变
    LightLeak {
        intensity: f32,
        color_r: f32,
        color_g: f32,
        color_b: f32,
    },
    /// 模糊 — 运动/景深/旋转
    Blur {
        motion: f32,
        dof: f32,
        swirl: f32,
    },
    /// 肤色优化（增强版）
    /// 仅针对亚洲肤色范围微调，过渡自然不伤画质
    SkinHsl {
        enabled: bool,
        remove_yellow: f32,   // 0~100, 去黄（降饱和 + 偏红）
        reduce_green: f32,    // 0~100, 减绿（胶片平光偏绿的补正）
        add_pink: f32,        // 0~100, 加粉（增加红蓝 → 粉润感）
        add_red: f32,         // 0~100, 加红（微增暖调血色）
        skin_brightness: f32, // -50~+50, 肤色亮度微调（双向）
    },
    /// 现代色调引擎（ModernTone）— 感知建模的非线性色调映射
    /// 日系空气/韩系奶油/清透冷白/美式复古咖 等现代摄影风格
    ModernTone {
        enabled: bool,
        style_idx: u8,         // 0=日系空气 1=韩系奶油 2=清透冷白 3=美式复古咖
        strength: f32,         // 0~150, 整体强度
        shadow_lift: f32,      // -50~+50, 暗部抬升
        highlight_compress: f32, // 0~100, 高光压缩
        midtone_contrast: f32, // -50~+50, 中间调对比
        shadow_hue: f32,       // 0~360
        shadow_sat: f32,       // 0~50
        highlight_hue: f32,    // 0~360
        highlight_sat: f32,    // 0~50
        sat_high_suppress: f32, // 0~100, 高饱和区压缩
        warmth_shift: f32,     // -30~+30, 整体色温微调
        fine_grain: f32,       // 0~100, 细颗粒
    },
    /// 色调分离（Split Toning）— 高光橙/阴影青
    SplitTone {
        enabled: bool,
        highlight_hue: f32,     // 0~360
        highlight_saturation: f32, // 0~100
        shadow_hue: f32,        // 0~360
        shadow_saturation: f32, // 0~100
        balance: f32,           // -100~+100, 偏向高光
        strength: f32,          // 0~100%
    },
    /// 输出锐化（Unsharp Mask）— 按分辨率自适应
    Sharp {
        enabled: bool,
        amount: f32,            // 0~100
        radius: f32,            // 0.5~3.0 px
        auto_radius: bool,      // 自动根据分辨率推算
    },
}

impl LayerType {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::FilmBase { .. } => "🎞",
            Self::Color { .. } => "🌈",
            Self::Curves { .. } => "📈",
            Self::Grain { .. } => "●",
            Self::Vignette { .. } => "◉",
            Self::LightLeak { .. } => "☀",
            Self::Blur { .. } => "◎",
            Self::SkinHsl { .. } => "👤",
            Self::ModernTone { .. } => "✨",
            Self::SplitTone { .. } => "🎨",
            Self::Sharp { .. } => "🔍",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::FilmBase { .. } => "胶片基底",
            Self::Color { .. } => "色彩",
            Self::Curves { .. } => "曲线",
            Self::Grain { .. } => "颗粒",
            Self::Vignette { .. } => "暗角/光晕",
            Self::LightLeak { .. } => "漏光",
            Self::Blur { .. } => "模糊",
            Self::SkinHsl { .. } => "肤色优化",
            Self::ModernTone { .. } => "现代色调",
            Self::SplitTone { .. } => "色调分离",
            Self::Sharp { .. } => "输出锐化",
        }
    }
}

// ============================================================
// 图层
// ============================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub name: String,
    pub layer_type: LayerType,
    pub blend_mode: BlendMode,
    pub opacity: f32,
    pub visible: bool,
}

impl Layer {
    pub fn new(name: String, layer_type: LayerType) -> Self {
        Self {
            name,
            blend_mode: BlendMode::default(),
            opacity: 1.0,
            visible: true,
            layer_type,
        }
    }
}

// ============================================================
// Catmull-Rom 样条（曲线面板与 LUT 共用）
// ============================================================

fn catmull_rom(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t;
    let t3 = t2 * t;
    0.5 * (2.0 * p1 + (p2 - p0) * t
        + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2
        + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t3)
}

fn catmull_rom_curve(x: f32, pts: &[(f32, f32); 5]) -> f32 {
    for i in 0..4 {
        if x >= pts[i].0 && x <= pts[i + 1].0 {
            let seg = pts[i + 1].0 - pts[i].0;
            let t = if seg > 0.0 { (x - pts[i].0) / seg } else { 0.0 };
            let p0 = if i == 0 { pts[0].1 } else { pts[i - 1].1 };
            let p1 = pts[i].1;
            let p2 = pts[i + 1].1;
            let p3 = if i >= 3 { pts[4].1 } else { pts[i + 2].1 };
            return catmull_rom(p0, p1, p2, p3, t);
        }
    }
    x
}

// ============================================================
// 快速 Box Blur（3-pass 近似 Gaussian，用于锐化）
// ============================================================

/// 单方向 box blur，半径 = (radius*2+1)
fn box_blur_horiz(src: &RgbImage, radius: u32) -> RgbImage {
    let (w, h) = src.dimensions();
    let mut out = src.clone();
    let r = radius as i32;
    for y in 0..h {
        for x in 0..w {
            let mut sum_r = 0i32; let mut sum_g = 0i32; let mut sum_b = 0i32; let mut cnt = 0i32;
            let x0 = (x as i32 - r).max(0);
            let x1 = (x as i32 + r).min(w as i32 - 1);
            for sx in x0..=x1 {
                let p = src.get_pixel(sx as u32, y);
                sum_r += p[0] as i32; sum_g += p[1] as i32; sum_b += p[2] as i32; cnt += 1;
            }
            let p = out.get_pixel_mut(x, y);
            p[0] = (sum_r / cnt) as u8;
            p[1] = (sum_g / cnt) as u8;
            p[2] = (sum_b / cnt) as u8;
        }
    }
    out
}

fn box_blur_vert(src: &RgbImage, radius: u32) -> RgbImage {
    let (w, h) = src.dimensions();
    let mut out = src.clone();
    let r = radius as i32;
    for x in 0..w {
        for y in 0..h {
            let mut sum_r = 0i32; let mut sum_g = 0i32; let mut sum_b = 0i32; let mut cnt = 0i32;
            let y0 = (y as i32 - r).max(0);
            let y1 = (y as i32 + r).min(h as i32 - 1);
            for sy in y0..=y1 {
                let p = src.get_pixel(x, sy as u32);
                sum_r += p[0] as i32; sum_g += p[1] as i32; sum_b += p[2] as i32; cnt += 1;
            }
            let p = out.get_pixel_mut(x, y);
            p[0] = (sum_r / cnt) as u8;
            p[1] = (sum_g / cnt) as u8;
            p[2] = (sum_b / cnt) as u8;
        }
    }
    out
}

/// 3-pass box blur ≈ Gaussian blur，半径 r 控制模糊程度
fn fast_gaussian_blur(img: &RgbImage, radius: u32) -> RgbImage {
    if radius == 0 { return img.clone(); }
    let p1 = box_blur_horiz(img, radius);
    let p2 = box_blur_vert(&p1, radius);
    let p3 = box_blur_horiz(&p2, radius);
    box_blur_vert(&p3, radius)
}

// ============================================================
// 图层栈 + 合成
// ============================================================

fn layer_type_order(lt: &LayerType) -> u8 {
    match lt {
        LayerType::FilmBase { .. } => 0,
        LayerType::Color { .. } => 1,
        LayerType::Curves { .. } => 2,
        LayerType::SkinHsl { .. } => 3,
        LayerType::ModernTone { .. } => 4,
        LayerType::SplitTone { .. } => 5,
        LayerType::Grain { .. } => 6,
        LayerType::Vignette { .. } => 7,
        LayerType::LightLeak { .. } => 8,
        LayerType::Blur { .. } => 9,
        LayerType::Sharp { .. } => 10,
    }
}

pub struct LayerStack {
    pub layers: Vec<Layer>,
}

impl LayerStack {
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    pub fn add(&mut self, layer: Layer) {
        self.layers.push(layer);
    }

    /// 按显影顺序插入
    pub fn add_sorted(&mut self, layer: Layer) {
        let order = layer_type_order(&layer.layer_type);
        let pos = self.layers.iter().position(|l| layer_type_order(&l.layer_type) > order);
        match pos {
            Some(idx) => self.layers.insert(idx, layer),
            None => self.layers.push(layer),
        }
    }

    pub fn remove(&mut self, idx: usize) {
        if idx < self.layers.len() {
            self.layers.remove(idx);
        }
    }

    pub fn move_up(&mut self, idx: usize) {
        if idx > 0 && idx < self.layers.len() {
            self.layers.swap(idx, idx - 1);
        }
    }

    pub fn move_down(&mut self, idx: usize) {
        if idx + 1 < self.layers.len() {
            self.layers.swap(idx, idx + 1);
        }
    }

    /// 判断某个 LayerType 是否属于"后处理"类（不影响 filmr 缓存）
    pub fn is_post_layer(lt: &LayerType) -> bool {
        matches!(lt,
            LayerType::Color{..} |
            LayerType::Curves{..} |
            LayerType::Grain{..} |
            LayerType::Vignette{..} |
            LayerType::LightLeak{..} |
            LayerType::Blur{..} |
            LayerType::SkinHsl{..} |
            LayerType::ModernTone{..} |
            LayerType::SplitTone{..}
        )
    }

    /// 判断是否属于需要 filmr 重新运算的层（影响缓存的）
    pub fn is_filmr_layer(lt: &LayerType) -> bool {
        matches!(lt, LayerType::FilmBase{..})
    }

    /// 合成所有图层到 base_img 上（f32 累加管线避免精度丢失）
    pub fn composite(&self, base_img: &RgbImage, include_sharp: bool) -> RgbImage {
        let (w, h) = base_img.dimensions();

        // f32 累加缓冲区：初始化为 base_img
        let mut acc: Vec<[f32; 3]> = base_img.pixels().map(|p| [
            p[0] as f32 / 255.0,
            p[1] as f32 / 255.0,
            p[2] as f32 / 255.0,
        ]).collect();

        for layer in &self.layers {
            if !layer.visible { continue; }
            if matches!(layer.layer_type, LayerType::FilmBase { .. }) { continue; }
            if matches!(layer.layer_type, LayerType::Sharp { .. }) && !include_sharp {
                continue;
            }

            // 从 f32 累加缓冲创建当前层的输入 u8 图像
            let mut layer_input = RgbImage::new(w, h);
            for (dst, src) in layer_input.pixels_mut().zip(acc.iter()) {
                dst[0] = (src[0].clamp(0.0, 1.0) * 255.0) as u8;
                dst[1] = (src[1].clamp(0.0, 1.0) * 255.0) as u8;
                dst[2] = (src[2].clamp(0.0, 1.0) * 255.0) as u8;
            }

            let effect = self.render_layer_effect(layer, &layer_input);
            self.blend_onto_f32(&mut acc, &effect, layer.blend_mode, layer.opacity);
        }

        // 最终 f32 → u8（仅一次精度丢失）
        let mut out = RgbImage::new(w, h);
        for (dst, src) in out.pixels_mut().zip(acc.iter()) {
            dst[0] = (src[0].clamp(0.0, 1.0) * 255.0) as u8;
            dst[1] = (src[1].clamp(0.0, 1.0) * 255.0) as u8;
            dst[2] = (src[2].clamp(0.0, 1.0) * 255.0) as u8;
        }
        out
    }

    /// f32 累加混合：将 effect 图层的 u8 像素转为 f32 并混合到 acc 缓冲区
    fn blend_onto_f32(&self, acc: &mut [[f32; 3]], effect: &RgbImage, mode: BlendMode, opacity: f32) {
        for (a, e) in acc.iter_mut().zip(effect.pixels()) {
            let ef = [e[0] as f32 / 255.0, e[1] as f32 / 255.0, e[2] as f32 / 255.0];
            let blended = match mode {
                BlendMode::Normal => ef,
                BlendMode::Multiply => [a[0] * ef[0], a[1] * ef[1], a[2] * ef[2]],
                BlendMode::Screen => [1.0 - (1.0 - a[0]) * (1.0 - ef[0]),
                                       1.0 - (1.0 - a[1]) * (1.0 - ef[1]),
                                       1.0 - (1.0 - a[2]) * (1.0 - ef[2])],
                BlendMode::Overlay => {
                    let over = |b: f32, l: f32| if b < 0.5 { 2.0 * b * l } else { 1.0 - 2.0 * (1.0 - b) * (1.0 - l) };
                    [over(a[0], ef[0]), over(a[1], ef[1]), over(a[2], ef[2])]
                }
                BlendMode::SoftLight => {
                    let soft = |b: f32, l: f32| {
                        if l < 0.5 { b - (1.0 - 2.0 * l) * b * (1.0 - b) }
                        else { b + (2.0 * l - 1.0) * ((if b < 0.25 { ((16.0 * b - 12.0) * b + 4.0) * b } else { b.sqrt() }) - b) }
                    };
                    [soft(a[0], ef[0]), soft(a[1], ef[1]), soft(a[2], ef[2])]
                }
                BlendMode::Color => {
                    let lum = |r: f32, g: f32, b: f32| 0.299 * r + 0.587 * g + 0.114 * b;
                    let d = lum(ef[0], ef[1], ef[2]) - lum(a[0], a[1], a[2]);
                    [a[0] + d, a[1] + d, a[2] + d]
                }
                BlendMode::Luminosity => {
                    let lum = |r: f32, g: f32, b: f32| 0.299 * r + 0.587 * g + 0.114 * b;
                    let src_lum = lum(a[0], a[1], a[2]);
                    let scale = if src_lum > 0.0 { lum(ef[0], ef[1], ef[2]) / src_lum } else { 1.0 };
                    [a[0] * scale, a[1] * scale, a[2] * scale]
                }
            };
            if opacity < 0.999 {
                a[0] += (blended[0] - a[0]) * opacity;
                a[1] += (blended[1] - a[1]) * opacity;
                a[2] += (blended[2] - a[2]) * opacity;
            } else {
                *a = blended;
            }
        }
    }

    /// 渲染单个调整层的效果图
    fn render_layer_effect(&self, layer: &Layer, input: &RgbImage) -> RgbImage {
        match &layer.layer_type {
            LayerType::FilmBase { .. } => input.clone(),
            LayerType::Color { warmth, tint, saturation } => {
                self.apply_color(input, *warmth, *tint, *saturation)
            }
            LayerType::Curves { contrast, highlights, shadows } => {
                self.apply_curves(input, *contrast, *highlights, *shadows)
            }
            LayerType::Grain { amount, size } => {
                self.apply_grain(input, *amount, *size)
            }
            LayerType::Vignette { strength, halation } => {
                self.apply_vignette(input, *strength, *halation)
            }
            LayerType::LightLeak { intensity, color_r, color_g, color_b } => {
                self.apply_light_leak(input, *intensity, *color_r, *color_g, *color_b)
            }
            LayerType::Blur { motion, dof, swirl } => {
                self.apply_blur(input, *motion, *dof, *swirl)
            }
            LayerType::SkinHsl { enabled, remove_yellow, reduce_green, add_pink, add_red, skin_brightness } => {
                if !*enabled { return input.clone(); }
                self.apply_skin_hsl(input, *remove_yellow, *reduce_green, *add_pink, *add_red, *skin_brightness)
            }
            LayerType::ModernTone { enabled, style_idx, strength, shadow_lift, highlight_compress, midtone_contrast, shadow_hue, shadow_sat, highlight_hue, highlight_sat, sat_high_suppress, warmth_shift, fine_grain } => {
                if !*enabled { return input.clone(); }
                self.apply_modern_tone(input, *style_idx, *strength, *shadow_lift, *highlight_compress, *midtone_contrast,
                    *shadow_hue, *shadow_sat, *highlight_hue, *highlight_sat, *sat_high_suppress, *warmth_shift, *fine_grain)
            }
            LayerType::SplitTone { enabled, highlight_hue, highlight_saturation, shadow_hue, shadow_saturation, balance, strength } => {
                if !*enabled { return input.clone(); }
                self.apply_split_tone(input, *highlight_hue / 360.0, *highlight_saturation / 100.0, *shadow_hue / 360.0, *shadow_saturation / 100.0, *balance, *strength)
            }
            LayerType::Sharp { enabled, amount, radius, auto_radius } => {
                if !*enabled { return input.clone(); }
                let r = if *auto_radius {
                    let longest = input.width().max(input.height()) as f32;
                    if longest > 4000.0 { 1.5 } else if longest > 2000.0 { 1.0 } else { 0.8 }
                } else { *radius };
                self.apply_sharp(input, *amount, r)
            }
        }
    }

    // ============================================================
    // 现有调整层效果
    // ============================================================

    fn apply_color(&self, img: &RgbImage, warmth: f32, tint: f32, saturation: f32) -> RgbImage {
        let mut out = img.clone();
        if warmth.abs() < 0.005 && tint.abs() < 0.005 && (saturation - 1.0).abs() < 0.01 {
            return out;
        }
        for pixel in out.pixels_mut() {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;

            let warmth_weight = 1.0 - (lum - 0.5).abs() * 1.6;
            let mut r2 = r * (1.0 + warmth * 0.12 * warmth_weight);
            let mut b2 = b * (1.0 - warmth * 0.12 * warmth_weight);
            let mut g2 = g;

            let tint_w = 0.3 + lum * 0.7;
            if tint > 0.0 {
                r2 *= 1.0 + tint * 0.12 * tint_w;
                g2 *= 1.0 - tint * 0.08 * tint_w;
                b2 *= 1.0 + tint * 0.12 * tint_w;
            } else {
                let a = tint.abs();
                r2 *= 1.0 - a * 0.06 * tint_w;
                g2 *= 1.0 + a * 0.12 * tint_w;
                b2 *= 1.0 - a * 0.06 * tint_w;
            }

            let sat_w = 1.0 - (lum - 0.5).abs() * 1.5;
            let effective_sat = 1.0 + (saturation - 1.0) * sat_w;
            if (effective_sat - 1.0).abs() > 0.005 {
                let gray = 0.299 * r2 + 0.587 * g2 + 0.114 * b2;
                r2 = gray + (r2 - gray) * effective_sat;
                g2 = gray + (g2 - gray) * effective_sat;
                b2 = gray + (b2 - gray) * effective_sat;
            }

            pixel[0] = (r2.clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = (g2.clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = (b2.clamp(0.0, 1.0) * 255.0) as u8;
        }
        out
    }

    fn apply_curves(&self, img: &RgbImage, contrast: f32, highlights: f32, shadows: f32) -> RgbImage {
        let mut out = img.clone();
        if contrast.abs() < 0.01 && highlights.abs() < 0.01 && shadows.abs() < 0.01 {
            return out;
        }
        let cx = [0.25_f32, 0.5, 0.75];
        let y0 = (0.25 - shadows * 0.25).clamp(0.0, 1.0);
        let y1 = (0.50 - contrast * 0.25).clamp(0.0, 1.0);
        let y2 = (0.75 + highlights * 0.25).clamp(0.0, 1.0);
        let pts = [(0.0, 0.0), (cx[0], y0), (cx[1], y1), (cx[2], y2), (1.0, 1.0)];

        let mut lut = [0u8; 256];
        for (i, entry) in lut.iter_mut().enumerate() {
            let x = i as f32 / 255.0;
            let y = catmull_rom_curve(x, &pts).clamp(0.0, 1.0);
            *entry = (y * 255.0) as u8;
        }

        for pixel in out.pixels_mut() {
            pixel[0] = lut[pixel[0] as usize];
            pixel[1] = lut[pixel[1] as usize];
            pixel[2] = lut[pixel[2] as usize];
        }
        out
    }

    fn apply_grain(&self, img: &RgbImage, amount: f32, _size: f32) -> RgbImage {
        if amount < 0.01 { return img.clone(); }
        let mut out = img.clone();
        let mut seed: u64 = 42;
        for (_y, row) in out.enumerate_rows_mut() {
            for (_x, _py, pixel) in row {
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
                let noise = ((seed >> 32) as f32 / (u32::MAX as f32) - 0.5) * amount * 0.3;
                let val = (pixel[0] as f32 / 255.0 + noise).clamp(0.0, 1.0);
                pixel[0] = (val * 255.0) as u8;
                pixel[1] = (val * 255.0) as u8;
                pixel[2] = (val * 255.0) as u8;
            }
        }
        out
    }

    fn apply_vignette(&self, img: &RgbImage, strength: f32, halation: f32) -> RgbImage {
        if strength < 0.01 && halation < 0.01 { return img.clone(); }
        let mut out = img.clone();
        let (w, h) = (img.width() as f32, img.height() as f32);
        let cx = w / 2.0;
        let cy = h / 2.0;
        let max_dist = (cx * cx + cy * cy).sqrt();

        for (y, row) in out.enumerate_rows_mut() {
            for (x, _y, pixel) in row {
                let dx = (x as f32 - cx) / max_dist;
                let dy = (y as f32 - cy) / max_dist;
                let dist = (dx * dx + dy * dy).sqrt();

                let vignette = 1.0 - strength * dist.powi(4);
                let luminance = 0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32;
                let halo = (luminance / 255.0 - 0.7).max(0.0) * halation * 0.4 * (1.0 - dist * 0.5);

                for c in 0..3 {
                    let v = pixel[c] as f32 / 255.0;
                    let adj = v * vignette + halo;
                    pixel[c] = (adj.clamp(0.0, 1.0) * 255.0) as u8;
                }
            }
        }
        out
    }

    fn apply_light_leak(&self, img: &RgbImage, intensity: f32, r: f32, g: f32, b: f32) -> RgbImage {
        if intensity < 0.01 { return img.clone(); }
        let mut out = img.clone();
        let (w, h) = (img.width() as f32, img.height() as f32);

        for (y, row) in out.enumerate_rows_mut() {
            for (x, _y, pixel) in row {
                let fx = x as f32 / w;
                let fy = y as f32 / h;
                let leak = (1.0 - fx).powi(2) * (1.0 - fy).max(fy).powi(2) * intensity;

                pixel[0] = ((pixel[0] as f32 / 255.0 + leak * r).min(1.0) * 255.0) as u8;
                pixel[1] = ((pixel[1] as f32 / 255.0 + leak * g).min(1.0) * 255.0) as u8;
                pixel[2] = ((pixel[2] as f32 / 255.0 + leak * b).min(1.0) * 255.0) as u8;
            }
        }
        out
    }

    fn apply_blur(&self, img: &RgbImage, _motion: f32, _dof: f32, _swirl: f32) -> RgbImage {
        img.clone()
    }

    // ============================================================
    // 肤色优化（SkinHSL 增强版）
    // ============================================================
    //
    // 5 个参数：remove_yellow / reduce_green / add_pink / add_red / skin_brightness
    //
    // 设计原则：
    //   1. 亚洲肤色色相 center=0.085 (30°), half_width=0.04 → 覆盖 ~16-45°
    //      包含偏黄、偏红、偏白肤色，排除蓝天绿草
    //   2. 明度加权：峰值 l=0.5 → 中间调肤色为主，阴影/高光自然衰减
    //   3. 低饱和保护：已低饱和像素减弱效果，防怪异
    //   4. 减绿策略：在 RGB 空间降 G 通道，比 HSL 更精准（胶片偏绿≠偏色相）
    //   5. 加粉/加红：微增 R/B 通道，不偏移色相太多，保持胶片色调方向
    //   6. 亮度双向：-50~+50，最大 ±5% luminance
    //   7. 所有调整上限保守：不过度干预胶片的色彩倾向
    // ============================================================

    fn apply_skin_hsl(&self, img: &RgbImage,
        remove_yellow: f32, reduce_green: f32, add_pink: f32, add_red: f32,
        skin_brightness: f32) -> RgbImage
    {
        let mut out = img.clone();
        if remove_yellow < 1.0 && reduce_green < 1.0 && add_pink < 1.0
            && add_red < 1.0 && skin_brightness.abs() < 0.5 { return out; }

        let sat_red = (remove_yellow / 100.0) * 0.15;   // 最大降饱和 15%
        let hue_shift = -(remove_yellow / 100.0) * 0.012; // 最大偏红 0.012
        let grn_red = (reduce_green / 100.0) * 0.12;    // 最大降绿 12%
        let pnk_r = (add_pink / 100.0) * 0.06;           // 加粉红通道 +6%
        let pnk_b = (add_pink / 100.0) * 0.05;           // 加粉蓝通道 +5%
        let red_boost = (add_red / 100.0) * 0.05;        // 加红 +5%
        let lum_adj = (skin_brightness / 50.0) * 0.05;   // 亮度 ±5%

        for pixel in out.pixels_mut() {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let (mut h, mut s, mut l) = rgb_to_hsl(r, g, b);

            let w_hue = hue_weight(h, 0.085, 0.04);
            if w_hue < 0.005 { continue; }

            let w_lum = if l < 0.15 {
                (l - 0.03) / 0.12
            } else if l > 0.78 {
                (0.92 - l) / 0.14
            } else {
                1.0
            };
            let w_lum = w_lum.clamp(0.0, 1.0);

            let w_sat = (s * 3.5).min(1.0);

            let w = w_hue * w_lum * w_sat;
            if w < 0.005 { continue; }

            // HSL adjustments (remove_yellow)
            h = (h + hue_shift * w + 1.0) % 1.0;
            s = (s - sat_red * w).clamp(0.02, 1.0);
            l = (l + lum_adj * w).clamp(0.01, 1.0);

            let (mut r2, mut g2, b2) = hsl_to_rgb(h, s, l);

            // RGB-space skin tone adjustments (precise channel control)
            // 减绿：降低 G 通道，模拟去扁平偏绿色罩
            let grn_w = w * (1.0 - (g2 - r2).max(0.0).min(0.15) / 0.15);
            g2 = (g2 * (1.0 - grn_red * grn_w)).clamp(0.0, 1.0);

            // 加粉：微增 R 和 B，营造健康粉润
            let pnk_w = w * (1.0 - s * 0.3);
            r2 = (r2 * (1.0 + pnk_r * pnk_w)).clamp(0.0, 1.0);
            let b3 = (b2 * (1.0 + pnk_b * pnk_w)).clamp(0.0, 1.0);

            // 加红：微增 R，暖调血色
            let red_w = w * (1.0 - s * 0.2);
            r2 = (r2 * (1.0 + red_boost * red_w)).clamp(0.0, 1.0);

            pixel[0] = (r2 * 255.0) as u8;
            pixel[1] = (g2 * 255.0) as u8;
            pixel[2] = (b3 * 255.0) as u8;
        }
        out
    }

    // ============================================================
    // 现代色调引擎（ModernTone）— 感知建模的非线性色调映射
    //
    // 核心思路：所有操作都是亮度/饱和度的函数，而非全局线性乘加
    //   1. 参数化tone curve：shadow lift + highlight soft-rolloff + midtone gamma
    //   2. 亮度相关色偏（split tone升级版，带平滑过渡带）
    //   3. 非线性饱和度：高饱和区压、低饱和区保、肤色豁免
    //   4. 整体色温偏移（暖/冷）
    //   5. 细颗粒（亮度加权，高光/暗部少、中间调多）
    // ============================================================

    #[allow(clippy::too_many_arguments)]
    fn apply_modern_tone(&self, img: &RgbImage,
        _style_idx: u8, strength: f32,
        shadow_lift: f32, highlight_compress: f32, midtone_contrast: f32,
        shadow_hue: f32, shadow_sat: f32,
        highlight_hue: f32, highlight_sat: f32,
        sat_high_suppress: f32, warmth_shift: f32, fine_grain: f32) -> RgbImage
    {
        let mut out = img.clone();
        let sf = (strength / 100.0).clamp(0.0, 1.5);
        if sf < 0.01 { return out; }

        // 归一化参数到内部工作范围
        let sh_lift  = (shadow_lift  / 50.0).clamp(-1.0, 1.0) * sf;  // -1~1
        let hl_comp  = (highlight_compress / 100.0).clamp(0.0, 1.0) * sf;
        let mc_adj   = (midtone_contrast / 50.0).clamp(-1.0, 1.0) * sf;
        let sh_hue   = shadow_hue / 360.0;
        let sh_sat   = (shadow_sat / 100.0).clamp(0.0, 0.5) * sf;
        let hl_hue   = highlight_hue / 360.0;
        let hl_sat   = (highlight_sat / 100.0).clamp(0.0, 0.5) * sf;
        let sat_sup  = (sat_high_suppress / 100.0).clamp(0.0, 0.6) * sf;
        let warm     = (warmth_shift / 30.0).clamp(-1.0, 1.0) * sf;
        let grain_amt= (fine_grain / 100.0).clamp(0.0, 0.5) * sf;

        // 阈值（影响过渡宽度）
        let sh_cut   = 0.35_f32; // 阴影区到 <0.35 开始色偏
        let hl_cut   = 0.65_f32; // 高光区到 >0.65 开始色偏
        let sat_thr  = 0.45_f32; // 饱和度高于此值开始压缩

        // 确定性噪点（LCG + 像素坐标扰动，二维分布无条纹）
        let mut seed: u32 = 0xdeadbeef;

        for (x, y, pixel) in out.enumerate_pixels_mut() {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let (h, s, mut l) = rgb_to_hsl(r, g, b);

            // ── Step1: 参数化 tone curve ──
            if sh_lift.abs() > 0.001 {
                if l < sh_cut {
                    let t = 1.0 - (l / sh_cut);
                    let t = t * t * (3.0 - 2.0 * t);
                    if sh_lift > 0.0 {
                        l = l + (1.0 - l) * sh_lift * 0.35 * t;
                    } else {
                        l = l * (1.0 + sh_lift * 0.4 * t);
                    }
                }
            }
            if hl_comp > 0.001 && l > hl_cut {
                let t = (l - hl_cut) / (1.0 - hl_cut);
                let roll = t * t;
                let pull = hl_comp * 0.22 * roll;
                l = l * (1.0 - pull);
            }
            if mc_adj.abs() > 0.001 {
                let gamma = if mc_adj > 0.0 {
                    1.0 + mc_adj * 0.5
                } else {
                    1.0 / (1.0 - mc_adj * 0.4)
                };
                if l > 0.2 && l < 0.8 {
                    let nl = (l - 0.2) / 0.6;
                    let nl = nl.powf(1.0 / gamma);
                    l = 0.2 + nl * 0.6;
                }
            }
            // [安全] tone curve 后 clamp，防 NaN/inf
            if !l.is_finite() { l = 0.5; }
            l = l.clamp(0.002, 0.998);

            // ── Step2: 亮度相关色偏 ──
            let mut h2 = h;
            let mut s2 = s;
            if sh_sat > 0.002 && l < sh_cut + 0.1 {
                let t = if l < sh_cut {
                    1.0 - l / sh_cut
                } else {
                    (sh_cut + 0.1 - l) / 0.1
                };
                let w = t.clamp(0.0, 1.0) * sh_sat;
                let wrap = |a: f32| -> f32 {
                    if !a.is_finite() { return 0.0; }
                    if a < 0.0 { a + 1.0 } else if a >= 1.0 { a - 1.0 } else { a }
                };
                let mut dh = sh_hue - h;
                if dh > 0.5 { dh -= 1.0; }
                if dh < -0.5 { dh += 1.0; }
                h2 = wrap(h + dh * w);
                s2 = (s + w * 0.5).min(1.0);
            }
            if hl_sat > 0.002 && l > hl_cut - 0.1 {
                let t = if l > hl_cut {
                    (l - hl_cut) / (1.0 - hl_cut)
                } else {
                    (l - (hl_cut - 0.1)) / 0.1
                };
                let w = t.clamp(0.0, 1.0) * hl_sat;
                let wrap = |a: f32| -> f32 {
                    if !a.is_finite() { return 0.0; }
                    if a < 0.0 { a + 1.0 } else if a >= 1.0 { a - 1.0 } else { a }
                };
                let mut dh = hl_hue - h2;
                if dh > 0.5 { dh -= 1.0; }
                if dh < -0.5 { dh += 1.0; }
                h2 = wrap(h2 + dh * w);
                s2 = (s2 + w * 0.4).min(1.0);
            }
            // [安全] h2/s2 必需有限
            if !h2.is_finite() { h2 = 0.0; }
            if !s2.is_finite() { s2 = 0.0; }

            // ── Step3: 非线性饱和度压缩 ──
            if sat_sup > 0.001 && s2 > sat_thr {
                let over = (s2 - sat_thr) / (1.0 - sat_thr);
                s2 = s2 - over * sat_sup * 0.3;
                s2 = s2.max(0.0);
            }
            s2 = s2.clamp(0.0, 1.0);

            // ── Step4: 整体色温 ──
            let (mut r2, mut g2, mut b2) = hsl_to_rgb(h2, s2, l);
            // [安全] hsl_to_rgb 输出必须有限
            if !r2.is_finite() { r2 = l; g2 = l; b2 = l; }
            if warm.abs() > 0.001 {
                let lum_w = 1.0 - (l - 0.55).abs() * 1.5; // 中高调为主
                let lum_w = lum_w.max(0.0).min(1.0);
                if warm > 0.0 {
                    r2 *= 1.0 + warm * 0.08 * lum_w;
                    b2 *= 1.0 - warm * 0.08 * lum_w;
                } else {
                    let a = warm.abs();
                    r2 *= 1.0 - a * 0.06 * lum_w;
                    b2 *= 1.0 + a * 0.08 * lum_w;
                }
            }

            // ── Step5: 细颗粒（亮度加权，中间调最多）──
            if grain_amt > 0.001 {
                let grain_w = 1.0 - ((l - 0.5).abs() * 1.8);
                let grain_w = grain_w.max(0.0);
                // 用像素坐标扰动seed，产生二维分布的颗粒（避免水平线）
                let px = (x as u32).wrapping_mul(374761393).wrapping_add((y as u32).wrapping_mul(668265263));
                let local_seed = seed.wrapping_add(px);
                let n = ((local_seed >> 8) as f32 / 16777216.0) - 0.5;
                seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
                let gn = n * grain_amt * 0.06 * grain_w;
                r2 = (r2 + gn).clamp(0.0, 1.0);
                g2 = (g2 + gn * 0.95).clamp(0.0, 1.0);
                b2 = (b2 + gn * 1.05).clamp(0.0, 1.0);
            }

            pixel[0] = (r2.clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = (g2.clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = (b2.clamp(0.0, 1.0) * 255.0) as u8;
        }
        out
    }

    // ============================================================
    // 新增：色调分离（SplitTone）
    //
    // 胶片特征：高光暖橙（卤化银完全显影）+ 阴影青绿（染料残留）
    // 实现：按亮度分段着色，中间调靠 balance 做平滑过渡
    // ============================================================

    #[allow(clippy::too_many_arguments)]
    fn apply_split_tone(&self, img: &RgbImage, hh: f32, hs: f32, sh: f32, ss: f32, balance: f32, strength: f32) -> RgbImage {
        let mut out = img.clone();
        let str_factor = strength / 100.0;
        if str_factor < 0.01 || (hs < 0.01 && ss < 0.01) { return out; }

        // 高光色（HSL），饱和度为 hs * 强度
        let h_sat = hs * str_factor;
        let s_sat = ss * str_factor;

        // balance: -100=全阴影着色，+100=全高光着色
        let bal = balance / 100.0; // -1~+1
        // 色交界点: 0.5 为中间，balance 偏移
        let mid = 0.5 + bal * 0.3;

        for pixel in out.pixels_mut() {
            let r = pixel[0] as f32 / 255.0;
            let g = pixel[1] as f32 / 255.0;
            let b = pixel[2] as f32 / 255.0;
            let lum = 0.299 * r + 0.587 * g + 0.114 * b;

            // 高光权重：亮度 > mid 时渐增
            let hw = if lum > mid {
                ((lum - mid) / (1.0 - mid)).clamp(0.0, 1.0)
            } else { 0.0 };
            // 阴影权重：亮度 < mid 时渐增
            let sw = if lum < mid {
                ((mid - lum) / mid).clamp(0.0, 1.0)
            } else { 0.0 };

            let to_hsl = |hue: f32, sat: f32, weight: f32| -> (f32, f32, f32) {
                if weight < 0.01 || sat < 0.01 { return (r, g, b); }
                let w = weight * sat * 0.3; // 最大着色幅度
                let (hr, hg, hb) = hsl_to_rgb(hue, 1.0, lum);
                (r + (hr - r) * w, g + (hg - g) * w, b + (hb - b) * w)
            };

            let (r2, g2, b2) = if hw > sw {
                to_hsl(hh, h_sat, hw)
            } else {
                let (r_s, g_s, b_s) = to_hsl(sh, s_sat, sw);
                // 中间调区域用 hw:sw 比例混合
                if hw > 0.01 && sw > 0.01 {
                    let total = hw + sw;
                    let (r_h, g_h, b_h) = to_hsl(hh, h_sat, hw);
                    (r_s + (r_h - r_s) * hw / total,
                     g_s + (g_h - g_s) * hw / total,
                     b_s + (b_h - b_s) * hw / total)
                } else { (r_s, g_s, b_s) }
            };

            pixel[0] = (r2.clamp(0.0, 1.0) * 255.0) as u8;
            pixel[1] = (g2.clamp(0.0, 1.0) * 255.0) as u8;
            pixel[2] = (b2.clamp(0.0, 1.0) * 255.0) as u8;
        }
        out
    }

    // ============================================================
    // 新增：输出锐化（Unsharp Mask）
    //
    // sharp_amount: 0~100（锐化强度）
    // sharp_radius: 高斯模糊半径（越大越"粗"的锐化）
    // 算法: result = original + (original - blurred) * amount/200
    // 预览时跳过（可关闭），仅导出时启用
    // ============================================================

    fn apply_sharp(&self, img: &RgbImage, amount: f32, radius: f32) -> RgbImage {
        if amount < 1.0 || radius < 0.3 { return img.clone(); }
        let r = radius.round().max(1.0) as u32;
        let amt = (amount / 100.0).clamp(0.0, 2.0);

        let blurred = fast_gaussian_blur(img, r);
        let mut out = img.clone();

        for (op, (ip, bp)) in out.pixels_mut().zip(img.pixels().zip(blurred.pixels())) {
            for c in 0..3 {
                let orig = ip[c] as f32;
                let blur = bp[c] as f32;
                let sharp = orig + (orig - blur) * amt;
                op[c] = sharp.clamp(0.0, 255.0) as u8;
            }
        }
        out
    }
}

/// 公共：肤色优化（不要 LayerStack 实例）
pub fn apply_skin_hsl_standalone(img: &RgbImage,
    remove_yellow: f32, reduce_green: f32, add_pink: f32, add_red: f32,
    skin_brightness: f32) -> RgbImage
{
    let stack = LayerStack::new();
    stack.apply_skin_hsl(img, remove_yellow, reduce_green, add_pink, add_red, skin_brightness)
}

/// 公共：色调分离（不要 LayerStack 实例）
pub fn apply_split_tone_standalone(img: &RgbImage, hh: f32, hs: f32, sh: f32, ss: f32, balance: f32, strength: f32) -> RgbImage {
    let stack = LayerStack::new();
    stack.apply_split_tone(img, hh, hs, sh, ss, balance, strength)
}

/// 公共：输出锐化（不要 LayerStack 实例）
pub fn apply_sharp_standalone(img: &RgbImage, amount: f32, radius: f32) -> RgbImage {
    let stack = LayerStack::new();
    stack.apply_sharp(img, amount, radius)
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}
