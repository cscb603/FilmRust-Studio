use image::RgbImage;
use filmrust::layers::{Layer, LayerStack, LayerType};

fn main() {
    let img = image::open(r"H:\中转待处理\工大街拍IMG_1661.JPG").unwrap().to_rgb8();
    let (w, h) = img.dimensions();
    eprintln!("原图 {}x{} 像素", w, h);
    let n = (w * h) as f64;

    let layers = LayerStack::default();

    // 1. 默认参数验证 — 应该 100% 相同
    let layer = Layer::new("color_def".into(), LayerType::Color { warmth: 0.0, tint: 0.0, saturation: 1.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Color 默认应与原图完全相同", &out, &img, n);
    out.save(r"H:\中转待处理\t1_color_default.png").ok();

    // 2. 曲线默认参数验证
    let layer = Layer::new("curve_def".into(), LayerType::Curves { contrast: 0.0, highlights: 0.0, shadows: 0.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Curves 默认应与原图完全相同", &out, &img, n);
    out.save(r"H:\中转待处理\t2_curve_default.png").ok();

    // 3. Color 色温 +15（常用值）
    let layer = Layer::new("warm15".into(), LayerType::Color { warmth: 15.0, tint: 0.0, saturation: 1.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Color +15色温（常用）", &out, &img, n);
    out.save(r"H:\中转待处理\t3_color_warm15.png").ok();

    // 4. Color 色温 +30（极端值）
    let layer = Layer::new("warm30".into(), LayerType::Color { warmth: 30.0, tint: 0.0, saturation: 1.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Color +30色温（极端）", &out, &img, n);
    out.save(r"H:\中转待处理\t4_color_warm30.png").ok();

    // 5. Color 色调 +15
    let layer = Layer::new("tint15".into(), LayerType::Color { warmth: 0.0, tint: 15.0, saturation: 1.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Color +15色调", &out, &img, n);
    out.save(r"H:\中转待处理\t5_color_tint15.png").ok();

    // 6. Curves 中等参数
    let layer = Layer::new("curve_mid".into(), LayerType::Curves { contrast: 15.0, highlights: 10.0, shadows: -10.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Curves 中等参数", &out, &img, n);
    out.save(r"H:\中转待处理\t6_curve_mid.png").ok();

    // 7. Curves 极端参数
    let layer = Layer::new("curve_ext".into(), LayerType::Curves { contrast: 40.0, highlights: 40.0, shadows: -40.0 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Curves 极端参数", &out, &img, n);
    out.save(r"H:\中转待处理\t7_curve_ext.png").ok();

    // 8. ModernTone 中等
    let layer = Layer::new("modern".into(), LayerType::ModernTone {
        enabled: true, style_idx: 0, strength: 80.0, shadow_lift: 8.0, highlight_compress: 12.0,
        midtone_contrast: 10.0, shadow_hue: 210.0, shadow_sat: 15.0,
        highlight_hue: 30.0, highlight_sat: 10.0, sat_high_suppress: 8.0,
        warmth_shift: 5.0, fine_grain: 0.0,
    });
    let out = layers.render_layer_effect(&layer, &img);
    check("ModernTone 中等", &out, &img, n);
    out.save(r"H:\中转待处理\t8_modern_mid.png").ok();

    // 9. SplitTone
    let layer = Layer::new("split".into(), LayerType::SplitTone {
        enabled: true, highlight_hue: 30.0, highlight_saturation: 50.0,
        shadow_hue: 210.0, shadow_saturation: 50.0, balance: 0.0, strength: 80.0,
    });
    let out = layers.render_layer_effect(&layer, &img);
    check("SplitTone 80%", &out, &img, n);
    out.save(r"H:\中转待处理\t9_split.png").ok();

    // 10. 全部叠一层（模拟真实使用：color→curve→modern→split）
    let mut pipeline = img.clone();
    let c1 = Layer::new("warm".into(), LayerType::Color { warmth: -5.0, tint: 2.0, saturation: 1.05 });
    pipeline = layers.render_layer_effect(&c1, &pipeline);
    let c2 = Layer::new("contrast".into(), LayerType::Curves { contrast: 10.0, highlights: 5.0, shadows: -3.0 });
    pipeline = layers.render_layer_effect(&c2, &pipeline);
    check("管线: Color→Curves", &pipeline, &img, n);
    pipeline.save(r"H:\中转待处理\t10_pipeline_color_curve.png").ok();

    // 11. Color 饱和度 0.5
    let layer = Layer::new("sat05".into(), LayerType::Color { warmth: 0.0, tint: 0.0, saturation: 0.5 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Color 饱和度 0.5", &out, &img, n);
    out.save(r"H:\中转待处理\t11_color_sat05.png").ok();

    // 12. Color 饱和度 1.5
    let layer = Layer::new("sat15".into(), LayerType::Color { warmth: 0.0, tint: 0.0, saturation: 1.5 });
    let out = layers.render_layer_effect(&layer, &img);
    check("Color 饱和度 1.5", &out, &img, n);
    out.save(r"H:\中转待处理\t12_color_sat15.png").ok();

    // 13. ModernTone 强度150+极端阴影提亮（测试软膝边界）
    let layer = Layer::new("modern_ext".into(), LayerType::ModernTone {
        enabled: true, style_idx: 0, strength: 150.0, shadow_lift: 40.0, highlight_compress: 40.0,
        midtone_contrast: 30.0, shadow_hue: 300.0, shadow_sat: 40.0,
        highlight_hue: 60.0, highlight_sat: 30.0, sat_high_suppress: 50.0,
        warmth_shift: 20.0, fine_grain: 0.0,
    });
    let out = layers.render_layer_effect(&layer, &img);
    check("ModernTone 极端参数", &out, &img, n);
    out.save(r"H:\中转待处理\t13_modern_extreme.png").ok();

    eprintln!("\n=== 全部测试完成 ===");
}

fn check(label: &str, out: &RgbImage, orig: &RgbImage, n: f64) {
    let (w, h) = out.dimensions();
    let mut max_diff: f64 = 0.0;
    let mut bad: u64 = 0;
    let mut blue_shift: u64 = 0;
    let mut black_dot: u64 = 0;
    let mut white_dot: u64 = 0;
    for (row_out, row_orig) in out.rows().zip(orig.rows()) {
        for (po, p_orig) in row_out.zip(row_orig) {
            let dr = po[0] as i32 - p_orig[0] as i32;
            let dg = po[1] as i32 - p_orig[1] as i32;
            let db = po[2] as i32 - p_orig[2] as i32;
            for c in 0..3 {
                let d = (po[c] as i32 - p_orig[c] as i32).abs() as f64;
                if d > max_diff { max_diff = d; }
                if d > 30.0 { bad += 1; }
            }
            // 检测蓝移
            if db > 30 && dr < -10 && dg < -10 {
                blue_shift += 1;
            }
            // 检测黑色坏点: RGB 都 <5 但原图 >50
            if po[0] < 5 && po[1] < 5 && po[2] < 5 && p_orig[0] > 50 && p_orig[1] > 50 && p_orig[2] > 50 {
                black_dot += 1;
            }
            // 检测全白: RGB 都 >250 但原图 <200
            if po[0] > 250 && po[1] > 250 && po[2] > 250 && p_orig[0] < 200 && p_orig[1] < 200 && p_orig[2] < 200 {
                white_dot += 1;
            }
        }
    }
    let bad_pct = bad as f64 / (n * 3.0) * 100.0;
    let blue_pct = blue_shift as f64 / n * 100.0;
    let max_diff = max_diff / 255.0 * 100.0;
    eprintln!("[{}] {}x{} bad={:.3}% blue={:.2}% black={} white={} maxΔ={:.1}%",
        label, w, h, bad_pct, blue_pct, black_dot, white_dot, max_diff);
}
