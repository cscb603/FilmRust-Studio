//! 图层引擎 — 混合模式 + 7 种调整层 + 合成管线
//!
//! 设计原则:
//! - 胶片基底走 filmr 全管线（一次渲染）
//! - 调整层走纯像素运算（毫秒级）
//! - 图层按顺序从上到下合成

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

/// 对两个像素应用混合模式
/// (base, blend) -> result
fn blend_pixel(base: [u8; 3], blend: [u8; 3], mode: BlendMode, opacity: f32) -> [u8; 3] {
    let bf = |b: u8| b as f32 / 255.0;
    let fi = |f: f32| (f.clamp(0.0, 1.0) * 255.0) as u8;

    let ba = [bf(base[0]), bf(base[1]), bf(base[2])];
    let bl = [bf(blend[0]), bf(blend[1]), bf(blend[2])];

    let result = match mode {
        BlendMode::Normal => bl,
        BlendMode::Multiply => [ba[0] * bl[0], ba[1] * bl[1], ba[2] * bl[2]],
        BlendMode::Screen => [1.0 - (1.0 - ba[0]) * (1.0 - bl[0]),
                               1.0 - (1.0 - ba[1]) * (1.0 - bl[1]),
                               1.0 - (1.0 - ba[2]) * (1.0 - bl[2])],
        BlendMode::Overlay => {
            let overlay_ch = |b: f32, l: f32| {
                if b < 0.5 { 2.0 * b * l } else { 1.0 - 2.0 * (1.0 - b) * (1.0 - l) }
            };
            [overlay_ch(ba[0], bl[0]), overlay_ch(ba[1], bl[1]), overlay_ch(ba[2], bl[2])]
        }
        BlendMode::SoftLight => {
            let soft_ch = |b: f32, l: f32| {
                if l < 0.5 {
                    b - (1.0 - 2.0 * l) * b * (1.0 - b)
                } else {
                    b + (2.0 * l - 1.0) * ((if b < 0.25 { ((16.0 * b - 12.0) * b + 4.0) * b } else { b.sqrt() }) - b)
                }
            };
            [soft_ch(ba[0], bl[0]), soft_ch(ba[1], bl[1]), soft_ch(ba[2], bl[2])]
        }
        BlendMode::Color => {
            let lum = |r: f32, g: f32, b: f32| 0.299 * r + 0.587 * g + 0.114 * b;
            let src_lum = lum(ba[0], ba[1], ba[2]);
            let blend_lum = lum(bl[0], bl[1], bl[2]);
            let d = blend_lum - src_lum;
            [ba[0] + d, ba[1] + d, ba[2] + d]
        }
        BlendMode::Luminosity => {
            let lum = |r: f32, g: f32, b: f32| 0.299 * r + 0.587 * g + 0.114 * b;
            let src_lum = lum(ba[0], ba[1], ba[2]);
            let blend_lum = lum(bl[0], bl[1], bl[2]);
            let scale = if src_lum > 0.0 { blend_lum / src_lum } else { 1.0 };
            [ba[0] * scale, ba[1] * scale, ba[2] * scale]
        }
    };

    // 透明度混合
    if opacity < 0.999 {
        [fi(ba[0] + (result[0] - ba[0]) * opacity),
         fi(ba[1] + (result[1] - ba[1]) * opacity),
         fi(ba[2] + (result[2] - ba[2]) * opacity)]
    } else {
        [fi(result[0]), fi(result[1]), fi(result[2])]
    }
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
}

impl LayerType {
    pub fn icon(&self) -> &'static str {
        match self {
            Self::FilmBase { .. } => "📷",
            Self::Color { .. } => "🌈",
            Self::Curves { .. } => "📈",
            Self::Grain { .. } => "●",
            Self::Vignette { .. } => "◉",
            Self::LightLeak { .. } => "☀",
            Self::Blur { .. } => "◎",
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
// 图层栈 + 合成
// ============================================================

fn layer_type_order(lt: &LayerType) -> u8 {
    match lt {
        LayerType::FilmBase { .. } => 0,
        LayerType::Color { .. } => 1,
        LayerType::Curves { .. } => 2,
        LayerType::Grain { .. } => 3,
        LayerType::Vignette { .. } => 4,
        LayerType::LightLeak { .. } => 5,
        LayerType::Blur { .. } => 6,
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

    /// 按显影顺序插入：胶片基底 → 色彩 → 曲线 → 颗粒 → 暗角 → 漏光 → 模糊
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

    /// 合成所有非胶片基底图层到 base_img 上
    /// base_img 是胶片基底层的输出（已经过 filmr 渲染）
    pub fn composite(&self, base_img: &RgbImage) -> RgbImage {
        let mut result = base_img.clone();

        for layer in &self.layers {
            if !layer.visible { continue; }
            // 跳过低片基底 — 它已经在 base_img 中
            if matches!(layer.layer_type, LayerType::FilmBase { .. }) { continue; }

            let effect = self.render_layer_effect(layer, &result);
            self.blend_onto(&mut result, &effect, layer.blend_mode, layer.opacity);
        }

        result
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
        }
    }

    /// 将 effect 图层按 blend_mode + opacity 混合到 base 上
    fn blend_onto(&self, base: &mut RgbImage, effect: &RgbImage, mode: BlendMode, opacity: f32) {
        assert_eq!(base.dimensions(), effect.dimensions());
        for (b, e) in base.pixels_mut().zip(effect.pixels()) {
            let blended = blend_pixel(b.0, e.0, mode, opacity);
            b.0 = blended;
        }
    }

    // ---- 调整层效果实现 ----

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

            // 胶片式 warmth: 中间调最强，阴影高光自然衰减（模拟乳剂光谱响应）
            let warmth_weight = 1.0 - (lum - 0.5).abs() * 1.6;
            let mut r2 = r * (1.0 + warmth * 0.12 * warmth_weight);
            let mut b2 = b * (1.0 - warmth * 0.12 * warmth_weight);
            let mut g2 = g;

            // 胶片式 tint: 中和亮度区变化更明显，暗部保留原色调
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

            // 胶片式 saturation: 峰值在中间调，阴影高光自然去饱和
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
        // 从参数计算控制点 Y（与 GUI 曲线面板公式一致）
        let cx = [0.25_f32, 0.5, 0.75];
        let y0 = (0.25 - shadows * 0.25).clamp(0.0, 1.0);
        let y1 = (0.50 - contrast * 0.25).clamp(0.0, 1.0);
        let y2 = (0.75 + highlights * 0.25).clamp(0.0, 1.0);
        let pts = [(0.0, 0.0), (cx[0], y0), (cx[1], y1), (cx[2], y2), (1.0, 1.0)];

        // 构建 Catmull-Rom 256 级 LUT
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
        // 简单伪随机颗粒（确定性，基于像素位置）
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

                // vignette: cos⁴ falloff
                let vignette = 1.0 - strength * dist.powi(4);

                // halation: highlight glow
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
                // 模拟从角落渗入的漏光
                let leak = (1.0 - fx).powi(2) * (1.0 - fy).max(fy).powi(2) * intensity;

                pixel[0] = ((pixel[0] as f32 / 255.0 + leak * r).min(1.0) * 255.0) as u8;
                pixel[1] = ((pixel[1] as f32 / 255.0 + leak * g).min(1.0) * 255.0) as u8;
                pixel[2] = ((pixel[2] as f32 / 255.0 + leak * b).min(1.0) * 255.0) as u8;
            }
        }
        out
    }

    fn apply_blur(&self, img: &RgbImage, _motion: f32, _dof: f32, _swirl: f32) -> RgbImage {
        // 模糊效果在 CPU 上较慢，简化为占位
        // Pro 正式版可接入 filmr 的 motion_blur/dof/rotational_blur 参数
        img.clone()
    }
}

impl Default for LayerStack {
    fn default() -> Self {
        Self::new()
    }
}