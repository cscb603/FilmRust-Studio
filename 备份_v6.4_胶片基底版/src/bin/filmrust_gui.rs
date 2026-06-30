//! FilmRust Studio — 独立 GUI 版 v4.0
//! 拖拽/浏览文件 → 选择胶片风格 → 左右对比预览 → 导出/批量处理
//! 中文字体: 自动探测系统字体 (msyh/simhei/simsun)
//! 图标: 嵌入 PNG 二进制

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Ui};
use egui::{ColorImage, TextureHandle, Vec2, pos2, vec2, CentralPanel, IconData};
use filmrust::{apply_film, find_filmr_stock, get_all_presets};
use filmr::SimulationConfig;
use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::thread;

const WATERMARK: &str = "星TAP 软件 2026  csb603@qq.com";

/// 胶片色调倾向预设 (名称, 色温warmth, 色调tint)
const CAST_PRESETS: &[(&str, f32, f32)] = &[
    ("标准",    0.0,   0.0),
    ("暖黄",    0.4,  -0.05),
    ("冷蓝",   -0.3,   0.05),
    ("青绿",   -0.1,  -0.4),
    ("橙调",    0.3,   0.2),
    ("品红",    0.1,   0.4),
];

fn cast_color(idx: usize) -> egui::Color32 {
    match idx {
        0 => egui::Color32::from_rgb(80,80,80),
        1 => egui::Color32::from_rgb(180,140,60),
        2 => egui::Color32::from_rgb(60,120,180),
        3 => egui::Color32::from_rgb(60,150,100),
        4 => egui::Color32::from_rgb(200,120,40),
        5 => egui::Color32::from_rgb(180,80,140),
        _ => egui::Color32::from_rgb(80,80,80),
    }
}

// ============================================================
//  加载嵌入图标 (PNG → IconData)
// ============================================================
fn load_app_icon() -> IconData {
    let png_bytes = include_bytes!("../../guitubiao.png");
    match image::load_from_memory(png_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            IconData { rgba: rgba.into_raw(), width: w, height: h }
        }
        Err(_) => IconData { rgba: vec![0; 64*64*4], width: 64, height: 64 }
    }
}

// ============================================================
//  中文字体设置 (自动探测系统字体)
// ============================================================
fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();

    let candidates = [
        r"C:\Windows\Fonts\msyh.ttc",
        r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\msyhl.ttc",
        r"C:\Windows\Fonts\simhei.ttf",
        r"C:\Windows\Fonts\simsun.ttc",
    ];

    for path in &candidates {
        if let Ok(data) = std::fs::read(path) {
            fonts.font_data.insert("chinese".into(), Arc::new(
                egui::FontData::from_owned(data).tweak(egui::FontTweak { scale: 1.0, y_offset_factor: -0.05, ..Default::default() })
            ));
            fonts.families.entry(egui::FontFamily::Proportional).or_default().insert(0, "chinese".into());
            fonts.families.entry(egui::FontFamily::Monospace).or_default().insert(0, "chinese".into());
            break;
        }
    }
    ctx.set_fonts(fonts);
}

fn setup_style(ctx: &egui::Context) {
    let mut style = (*ctx.global_style()).clone();
    style.spacing.item_spacing = vec2(12.0, 8.0);
    style.visuals.widgets.noninteractive.corner_radius = egui::CornerRadius::same(18);
    style.visuals.widgets.inactive.corner_radius = egui::CornerRadius::same(18);
    style.visuals.widgets.hovered.corner_radius = egui::CornerRadius::same(18);
    style.visuals.widgets.active.corner_radius = egui::CornerRadius::same(18);
    style.visuals.window_corner_radius = egui::CornerRadius::same(18);
    style.visuals.window_shadow = egui::epaint::Shadow {
        offset: [0, 12],
        blur: 16,
        spread: 0,
        color: egui::Color32::BLACK.gamma_multiply(0.3),
    };
    ctx.set_global_style(style);
}

// ============================================================
//  消息类型
// ============================================================

struct ProcessResult {
    ok: bool,
    image: Option<image::DynamicImage>,
    error: Option<String>,
}

enum BatchMsg {
    Progress(usize, usize),
    Done,
}

// ============================================================
//  状态定义
// ============================================================

struct FilmRustGui {
    files: Vec<PathBuf>,
    selected_idx: usize,
    output_dir: Option<PathBuf>,

    original_img: Option<image::DynamicImage>,
    processed_img: Option<image::DynamicImage>,
    original_tex: Option<TextureHandle>,
    processed_tex: Option<TextureHandle>,
    display_img_w: u32,
    display_img_h: u32,

    is_processing: bool,
    has_processed: bool,
    proc_result_rx: Option<mpsc::Receiver<ProcessResult>>,

    style_idx: usize,
    cast_idx: usize,
    strength: f32,
    grain: f32,
    warmth: f32,
    tint: f32,
    split_pos: f32,

    batch_running: bool,
    batch_current: usize,
    batch_total: usize,
    batch_results: Vec<String>,
    batch_rx: Option<mpsc::Receiver<BatchMsg>>,

    preset_ids: Vec<String>,
    preset_names: Vec<String>,
    preset_descriptions: Vec<String>,
    preset_tags: Vec<Vec<String>>,

    status: String,
    status_ok: bool,
}

impl Default for FilmRustGui {
    fn default() -> Self {
        let presets = get_all_presets();
        let ids: Vec<String> = presets.iter().map(|p| p.id.clone()).collect();
        let names: Vec<String> = presets.iter().map(|p| {
            format!("{} — {} ({})", p.name, p.manufacturer,
                p.tags.first().map(|t| t.as_str()).unwrap_or("通用"))
        }).collect();
        let descriptions: Vec<String> = presets.iter().map(|p| p.description.clone()).collect();
        let tags: Vec<Vec<String>> = presets.iter().map(|p| p.tags.clone()).collect();

        Self {
            files: Vec::new(),
            selected_idx: 0,
            output_dir: None,
            original_img: None, processed_img: None,
            original_tex: None, processed_tex: None,
            display_img_w: 0, display_img_h: 0,
            is_processing: false, has_processed: false,
            proc_result_rx: None,
            style_idx: 0,
            cast_idx: 0,
            strength: 100.0, grain: 100.0,
            warmth: 0.0, tint: 0.0,
            split_pos: 0.5,
            batch_running: false, batch_current: 0, batch_total: 0,
            batch_results: Vec::new(), batch_rx: None,
            preset_ids: ids, preset_names: names,
            preset_descriptions: descriptions, preset_tags: tags,
            status: "就绪 — 拖拽图片到窗口或点击「打开文件」".to_string(),
            status_ok: true,
        }
    }
}

impl FilmRustGui {
    fn add_file(&mut self, path: PathBuf) {
        let ext = path.extension()
            .and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
        if matches!(ext.as_str(), "jpg" | "jpeg" | "png" | "tiff" | "tif" | "bmp") && !self.files.contains(&path) {
            self.files.push(path);
        }
    }

    fn load_image_for_display(&mut self, ctx: &egui::Context) {
        if self.selected_idx >= self.files.len() { return; }
        let path = &self.files[self.selected_idx];
        let fname = path.file_name().unwrap_or_default().to_string_lossy().to_string();

        match image::open(path) {
            Ok(img) => {
                self.display_img_w = img.width();
                self.display_img_h = img.height();
                self.original_img = Some(img);
                self.processed_img = None;
                self.has_processed = false;

                if let Some(ref orig) = self.original_img {
                    let display = Self::scale_for_display(orig, 1600);
                    let rgba = display.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let ci = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
                    self.original_tex = Some(ctx.load_texture("original", ci, Default::default()));
                }
                self.status = format!("已加载: {} ({}×{})", fname, self.display_img_w, self.display_img_h);
                self.status_ok = true;
            }
            Err(e) => { self.status = format!("加载失败: {} — {}", fname, e); self.status_ok = false; }
        }
    }

    fn scale_for_display(img: &image::DynamicImage, max_px: u32) -> image::DynamicImage {
        let (w, h) = (img.width(), img.height());
        if w <= max_px && h <= max_px { return img.clone(); }
        let sc = if w > h { max_px as f64 / w as f64 } else { max_px as f64 / h as f64 };
        img.resize_exact((w as f64 * sc).round() as u32, (h as f64 * sc).round() as u32,
            image::imageops::FilterType::Lanczos3)
    }

    fn do_process(img: image::DynamicImage, style_id: &str, strength: f32, grain: f32, warmth: f32, tint: f32) -> ProcessResult {
        let factor = strength / 100.0;
        match find_filmr_stock(style_id) {
            Ok(stock) => {
                let rgb = img.to_rgb8();
                let config = SimulationConfig {
                    exposure_time: 1.0,
                    auto_levels: true,
                    white_balance_mode: filmr::WhiteBalanceMode::Off,
                    enable_grain: grain > 5.0,
                    motion_blur_amount: 0.0,
                    object_motion_amount: 0.0,
                    light_leak: filmr::light_leak::LightLeakConfig { enabled: false, leaks: Vec::new() },
                    saturation: 1.0 + (factor - 1.0) * 0.15,
                    warmth,
                    ..Default::default()
                };
                match apply_film(&rgb, &stock, &config) {
                    Ok(mut r) => {
                        if tint.abs() > 0.005 {
                            r = filmrust::apply_tint_to_rgb(&r, tint);
                        }
                        ProcessResult { ok: true, image: Some(image::DynamicImage::ImageRgb8(r)), error: None }
                    }
                    Err(e) => ProcessResult { ok: false, image: None, error: Some(e.to_string()) },
                }
            }
            Err(e) => ProcessResult { ok: false, image: None, error: Some(e.to_string()) },
        }
    }

    fn process_current(&mut self, ctx: &egui::Context) {
        if self.is_processing || self.selected_idx >= self.files.len() || self.original_img.is_none() { return; }
        self.is_processing = true;
        self.status = "处理中...".to_string();
        self.status_ok = true;

        let style_id = self.preset_ids.get(self.style_idx).cloned().unwrap_or_default();
        let img = self.original_img.as_ref().unwrap().clone();
        let s = self.strength; let g = self.grain; let w = self.warmth; let t = self.tint;
        let c = ctx.clone();
        let (tx, rx) = mpsc::channel();
        self.proc_result_rx = Some(rx);
        thread::spawn(move || { let _ = tx.send(Self::do_process(img, &style_id, s, g, w, t)); c.request_repaint(); });
    }

    fn check_process_result(&mut self, ctx: &egui::Context) {
        let rx = match self.proc_result_rx.take() {
            Some(rx) => rx,
            None => return,
        };
        if let Ok(result) = rx.try_recv() {
            self.is_processing = false;
            if result.ok {
                if let Some(img) = result.image {
                    self.processed_img = Some(img.clone());
                    self.has_processed = true;
                    let d = Self::scale_for_display(&img, 1600);
                    let rgba = d.to_rgba8();
                    let (w, h) = rgba.dimensions();
                    let ci = ColorImage::from_rgba_unmultiplied([w as usize, h as usize], rgba.as_raw());
                    self.processed_tex = Some(ctx.load_texture("processed", ci, Default::default()));
                    self.status = "处理完成 — 拖动分割线对比原图".to_string();
                    self.status_ok = true;
                }
            } else {
                self.status = format!("处理失败: {}", result.error.unwrap_or_default());
                self.status_ok = false;
            }
        } else {
            self.proc_result_rx = Some(rx);
        }
    }

    fn export_current(&self, path: &Path) -> Result<(), String> {
        match self.processed_img { Some(ref img) => img.save(path).map_err(|e| format!("保存失败: {}", e)), None => Err("没有已处理的图片".to_string()) }
    }

    fn poll_batch(rx: mpsc::Receiver<BatchMsg>, me: &mut Self) -> Option<mpsc::Receiver<BatchMsg>> {
        let mut keep = true;
        while let Ok(msg) = rx.try_recv() {
            match msg {
                BatchMsg::Progress(cur, total) => {
                    me.batch_current = cur; me.batch_total = total;
                    me.status = format!("批量处理: {}/{}", cur, total); me.status_ok = true;
                }
                BatchMsg::Done => {
                    me.batch_running = false;
                    me.status = format!("批量完成! 共 {} 张", me.batch_total); me.status_ok = true;
                    keep = false;
                }
            }
        }
        if keep { Some(rx) } else { None }
    }
}

// ============================================================
//  eframe App 实现
// ============================================================

impl eframe::App for FilmRustGui {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx().clone();

        // 拖拽文件
        ctx.input(|i| { for f in &i.raw.dropped_files { if let Some(p) = &f.path { self.add_file(p.clone()); } } });
        if !self.files.is_empty() && self.original_tex.is_none() {
            if self.selected_idx >= self.files.len() { self.selected_idx = 0; }
            self.load_image_for_display(&ctx);
        }

        self.check_process_result(&ctx);

        if let Some(rx) = self.batch_rx.take() {
            self.batch_rx = Self::poll_batch(rx, self);
        }

        // ========== 顶部工具栏 ==========
        egui::Panel::top("toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                // 用文字图标代替 emoji（避免字体问题）
                ui.heading("[FR] FilmRust Studio");
                ui.separator();

                if ui.button("[打开] 图片").clicked() {
                    if let Some(files) = rfd::FileDialog::new()
                        .add_filter("图片", &["jpg","jpeg","png","tiff","tif","bmp"]).pick_files()
                    {
                        for f in files { self.add_file(f); }
                        if !self.files.is_empty() { self.selected_idx = self.files.len() - 1; self.load_image_for_display(&ctx); }
                    }
                }

                if ui.button("[打开] 文件夹").clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        let mut cnt = 0;
                        if let Ok(entries) = std::fs::read_dir(&dir) {
                            for e in entries.flatten() { let p = e.path(); if p.is_file() { self.add_file(p); cnt += 1; } }
                        }
                        if cnt > 0 { self.selected_idx = self.files.len() - cnt; self.load_image_for_display(&ctx); }
                    }
                }

                ui.separator();

                if ui.add_enabled(self.has_processed, egui::Button::new("[保存] 导出")).clicked() {
                    if let Some(p) = rfd::FileDialog::new().add_filter("JPEG",&["jpg"]).add_filter("PNG",&["png"])
                        .set_file_name("output.jpg").save_file()
                    {
                        match self.export_current(&p) {
                            Ok(()) => { self.status = "已导出".to_string(); self.status_ok = true; }
                            Err(e) => { self.status = e; self.status_ok = false; }
                        }
                    }
                }

                if ui.add_enabled(!self.files.is_empty(), egui::Button::new("[批量] 处理")).clicked() {
                    if let Some(dir) = rfd::FileDialog::new().pick_folder() {
                        self.output_dir = Some(dir.clone());
                        self.batch_running = true; self.batch_current = 0; self.batch_total = self.files.len();
                        self.batch_results.clear();

                        let sid = self.preset_ids.get(self.style_idx).cloned().unwrap_or_default();
                        let s = self.strength; let g = self.grain; let w = self.warmth; let t = self.tint;
                        let files = self.files.clone(); let c = ctx.clone();
                        let (btx, brx) = mpsc::channel();
                        self.batch_rx = Some(brx);

                        thread::spawn(move || {
                            for (i, file) in files.iter().enumerate() {
                                let out_name = format!("{}_film.jpg", file.file_stem().unwrap_or_default().to_string_lossy());
                                if let Ok(img) = image::open(file) {
                                    let r = Self::do_process(img, &sid, s, g, w, t);
                                    if let Some(pi) = r.image { let _ = pi.save(dir.join(&out_name)); }
                                }
                                let _ = btx.send(BatchMsg::Progress(i+1, files.len()));
                                c.request_repaint();
                            }
                            let _ = btx.send(BatchMsg::Done);
                            c.request_repaint();
                        });
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.selected_idx < self.files.len() {
                        let nm = self.files[self.selected_idx].file_name().unwrap_or_default().to_string_lossy().to_string();
                        ui.label(format!("[{}]  {}x{}", nm, self.display_img_w, self.display_img_h));
                    }
                });
            });
        });

        // ========== 底部控制面板 (必须在 CentralPanel 之前) ==========
        egui::Panel::bottom("controls_panel").min_size(120.0).show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                ui.vertical(|ui| {
                    ui.label("胶片风格 (57种)");
                    egui::ComboBox::from_id_salt("style_select").width(320.0)
                        .selected_text(self.preset_names.get(self.style_idx).map(|s| s.as_str()).unwrap_or("选择风格"))
                        .show_ui(ui, |ui| {
                            for (i, nm) in self.preset_names.iter().enumerate() {
                                if ui.selectable_label(i == self.style_idx, nm).clicked() {
                                    self.style_idx = i; self.has_processed = false; self.processed_tex = None;
                                }
                            }
                        });
                    // 胶片描述
                    if let Some(desc) = self.preset_descriptions.get(self.style_idx) {
                        ui.add_space(4.0);
                        ui.label(egui::RichText::new(desc).size(10.5).weak());
                    }
                    // 标签
                    if let Some(tags) = self.preset_tags.get(self.style_idx) {
                        ui.horizontal(|ui| {
                            ui.add_space(4.0);
                            for t in tags {
                                ui.label(egui::RichText::new(format!("[{}]", t)).size(10.0).color(egui::Color32::from_rgb(140, 160, 180)));
                                ui.add_space(2.0);
                            }
                        });
                    }
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.add(egui::Slider::new(&mut self.strength, 10.0..=150.0).text("强度").suffix("%"));
                    ui.add(egui::Slider::new(&mut self.grain, 0.0..=200.0).text("颗粒").suffix("%"));
                });
                ui.separator();
                ui.vertical(|ui| {
                    ui.add_space(4.0);
                    let ready = self.original_img.is_some() && !self.files.is_empty();
                    if ui.add_enabled(ready && !self.is_processing, egui::Button::new("[处理] 应用").min_size(vec2(100.0,32.0))).clicked() {
                        self.process_current(&ctx);
                    }
                    ui.add_space(4.0);
                    if ui.add_enabled(self.has_processed, egui::Button::new("[保存] 导出").min_size(vec2(100.0,28.0))).clicked() {
                        if let Some(p) = rfd::FileDialog::new().add_filter("JPEG",&["jpg"]).add_filter("PNG",&["png"]).set_file_name("output.jpg").save_file() {
                            match self.export_current(&p) { Ok(()) => { self.status="已导出".to_string(); self.status_ok=true; } Err(e)=>{ self.status=e; self.status_ok=false; } }
                        }
                    }
                });
                if self.batch_running {
                    ui.separator();
                    ui.vertical(|ui| {
                        ui.label("批量处理中...");
                        let p = if self.batch_total > 0 { self.batch_current as f32 / self.batch_total as f32 } else { 0.0 };
                        ui.add(egui::ProgressBar::new(p).text(format!("{}/{}",self.batch_current,self.batch_total)).desired_width(120.0));
                    });
                }
            });
            // ========== 色调倾向控制区（两行布局） ==========
            ui.separator();
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("色调预设:").size(11.0).weak());
                ui.add_space(8.0);
                // 预设按钮组 - 使用横向流式布局，按钮间加间距
                for (i, &(label, w, t)) in CAST_PRESETS.iter().enumerate() {
                    let is_sel = i == self.cast_idx;
                    let color = cast_color(i);
                    let txt = egui::RichText::new(label).color(egui::Color32::WHITE).size(11.0);
                    let btn = egui::Button::new(txt).min_size(vec2(52.0, 24.0))
                        .fill(if is_sel { color } else { egui::Color32::from_rgb(60,60,60) });
                    if ui.add(btn).clicked() {
                        self.cast_idx = i;
                        self.warmth = w;
                        self.tint = t;
                        self.has_processed = false;
                        self.processed_tex = None;
                    }
                    if i < CAST_PRESETS.len() - 1 {
                        ui.add_space(4.0);
                    }
                }
                // 右侧标签
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    let wv = self.warmth;
                    let tv = self.tint;
                    let label = if wv.abs() < 0.05 && tv.abs() < 0.05 { "标准" }
                        else if wv > 0.15 && tv.abs() < 0.1 { "暖调" }
                        else if wv < -0.15 && tv.abs() < 0.1 { "冷调" }
                        else if tv < -0.15 { "青绿" }
                        else if tv > 0.15 { "品红" }
                        else { "混合" };
                    ui.label(egui::RichText::new(format!("[{}]", label)).size(11.0).color(egui::Color32::GRAY));
                });
            });
            ui.horizontal(|ui| {
                ui.add_space(4.0);
                // 色温滑块
                ui.label(egui::RichText::new("色温:").size(11.0));
                let old_warmth = self.warmth;
                ui.add(egui::Slider::new(&mut self.warmth, -1.0..=1.0).show_value(false));
                ui.label(egui::RichText::new(format!("{:.1}", self.warmth)).size(11.0).monospace());
                // 色调滑块
                ui.add_space(12.0);
                ui.label(egui::RichText::new("色调:").size(11.0));
                let old_tint = self.tint;
                ui.add(egui::Slider::new(&mut self.tint, -1.0..=1.0).show_value(false));
                ui.label(egui::RichText::new(format!("{:.1}", self.tint)).size(11.0).monospace());
                // 滑块变化时重置处理结果
                if (old_warmth - self.warmth).abs() > 0.001 || (old_tint - self.tint).abs() > 0.001 {
                    self.cast_idx = CAST_PRESETS.len(); // 自定义值，不匹配任何预设
                    self.has_processed = false;
                    self.processed_tex = None;
                }
            });
            ui.separator();
            ui.horizontal(|ui| {
                let c = if self.status_ok { egui::Color32::from_rgb(140,200,140) } else { egui::Color32::from_rgb(240,140,140) };
                ui.label(egui::RichText::new(&self.status).color(c));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(egui::RichText::new(WATERMARK).size(11.0).color(egui::Color32::DARK_GRAY));
                    ui.separator();
                    let sn = self.preset_names.get(self.style_idx).and_then(|s| s.split(" — ").next()).unwrap_or("未选择");
                    ui.label(format!("[{}]", sn));
                });
            });
            ui.label(egui::RichText::new("  推荐: 曝光正常、直方图覆盖中灰-高光的 JPEG/TIFF/PNG。拖拽图片到窗口或点击「打开文件」。").size(10.0).weak());
        });

        // ========== 左侧文件列表 ==========
        egui::Panel::left("file_panel")
            .resizable(true)
            .default_size(220.0).min_size(140.0)
            .show_inside(ui, |ui| {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("文件列表 ({})", self.files.len()));
                        if !self.files.is_empty() && ui.button("[×] 清除").clicked() {
                            self.files.clear();
                            self.selected_idx = 0;
                            self.original_img = None; self.processed_img = None;
                            self.original_tex = None; self.processed_tex = None;
                            self.has_processed = false;
                            self.status = "列表已清空".to_string();
                        }
                    });
                    ui.separator();
                    if self.files.is_empty() { ui.add_space(40.0); ui.label("拖拽图片到这里"); return; }
                    let sel = self.selected_idx;
                    let fnames: Vec<String> = self.files.iter().map(|p| p.file_name().unwrap_or_default().to_string_lossy().to_string()).collect();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let mut to_remove: Option<usize> = None;
                        for (i, nm) in fnames.iter().enumerate() {
                            ui.horizontal(|ui| {
                                let lb = if i == sel { format!(" > {}", nm) } else { format!("   {}", nm) };
                                if ui.selectable_label(i == sel, &lb).clicked() { self.selected_idx = i; self.load_image_for_display(&ctx); }
                                if ui.button("[×]").clicked() { to_remove = Some(i); }
                            });
                        }
                        if let Some(idx) = to_remove {
                            self.files.remove(idx);
                            if self.files.is_empty() {
                                self.selected_idx = 0;
                                self.original_img = None; self.processed_img = None;
                                self.original_tex = None; self.processed_tex = None;
                                self.has_processed = false;
                            } else if self.selected_idx >= self.files.len() {
                                self.selected_idx = self.files.len() - 1;
                            }
                            self.load_image_for_display(&ctx);
                        }
                    });
                });
            });

        // ========== 中央预览区 ==========
        CentralPanel::default().show_inside(ui, |ui| {
            if self.files.is_empty() {
                ui.vertical_centered(|ui| {
                    ui.add_space(ui.available_height() * 0.25);
                    ui.heading(egui::RichText::new("[FR] FilmRust Studio").size(36.0));
                    ui.label("物理级胶片模拟工具 -- 60 种经典胶片风格");
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new("拖拽图片到窗口，或点击上方 [打开] 开始").size(16.0));
                    ui.label(egui::RichText::new("支持 JPG / PNG / TIFF / BMP").size(13.0).color(egui::Color32::GRAY));
                    ui.add_space(8.0);
                    ui.label(egui::RichText::new(WATERMARK).size(12.0).color(egui::Color32::DARK_GRAY));
                    ui.add_space(16.0);
                    if ui.button("[打开] 图片文件").clicked() {
                        if let Some(files) = rfd::FileDialog::new()
                            .add_filter("图片", &["jpg","jpeg","png","tiff","tif","bmp"]).pick_files()
                        {
                            for f in files { self.add_file(f); }
                            if !self.files.is_empty() { self.selected_idx = self.files.len() - 1; self.load_image_for_display(&ctx); }
                        }
                    }
                });
                return;
            }

            if self.selected_idx >= self.files.len() { return; }
            if self.original_tex.is_none() { self.load_image_for_display(&ctx); }
            let orig_tex = match self.original_tex.as_ref() { Some(t) => t.clone(), None => { ui.label("加载中..."); return; } };

            let avail = ui.available_size();
            let (tw, th) = (orig_tex.size()[0] as f32, orig_tex.size()[1] as f32);
            if tw < 1.0 || th < 1.0 { return; }
            let aspect = tw / th;
            let margin = 8.0;
            let (mw, mh) = (avail.x - margin*2.0, avail.y - margin*2.0);
            let img_size = if mw/mh > aspect { vec2(mh*aspect, mh) } else { vec2(mw, mw/aspect) };
            let x_off = ((avail.x - img_size.x)*0.5).max(0.0);
            let (_, area) = ui.allocate_space(vec2(avail.x, avail.y - 2.0));
            let img_rect = egui::Rect::from_min_size(pos2(area.min.x + x_off, area.min.y + margin), img_size);
            let p = ui.painter();

            if self.is_processing {
                p.image(orig_tex.id(), img_rect, egui::Rect::from_min_max(pos2(0.0,0.0), pos2(1.0,1.0)), egui::Color32::GRAY);
                p.text(img_rect.center(), egui::Align2::CENTER_CENTER, "正在处理...", egui::FontId::proportional(24.0), egui::Color32::WHITE);
                return;
            }

            if self.has_processed {
                if let Some(ref proc_tex) = self.processed_tex {
                    p.image(proc_tex.id(), img_rect, egui::Rect::from_min_max(pos2(0.0,0.0), pos2(1.0,1.0)), egui::Color32::WHITE);
                    let split_x = img_rect.min.x + img_rect.width() * self.split_pos;
                    let br = egui::Rect::from_min_max(img_rect.min, pos2(split_x, img_rect.max.y));
                    let buv = egui::Rect::from_min_max(pos2(0.0,0.0), pos2(self.split_pos,1.0));
                    p.image(orig_tex.id(), br, buv, egui::Color32::WHITE);
                    p.line_segment([pos2(split_x,img_rect.min.y), pos2(split_x,img_rect.max.y)], (2.5, egui::Color32::WHITE));
                    p.circle_filled(pos2(split_x, img_rect.center().y), 8.0, egui::Color32::WHITE);
                    p.text(pos2(img_rect.min.x+8.0, img_rect.min.y+8.0), egui::Align2::LEFT_TOP, "原图",
                        egui::FontId::proportional(14.0), egui::Color32::WHITE.gamma_multiply(0.85));
                    p.text(pos2(img_rect.max.x-8.0, img_rect.min.y+8.0), egui::Align2::RIGHT_TOP, "处理后",
                        egui::FontId::proportional(14.0), egui::Color32::WHITE.gamma_multiply(0.85));
                    let resp = ui.allocate_rect(img_rect, egui::Sense::click_and_drag());
                    if resp.dragged_by(egui::PointerButton::Primary) {
                        if let Some(hov) = resp.hover_pos() { self.split_pos = ((hov.x - img_rect.min.x)/img_rect.width()).clamp(0.05, 0.95); }
                    }
                    return;
                }
            }

            p.image(orig_tex.id(), img_rect, egui::Rect::from_min_max(pos2(0.0,0.0), pos2(1.0,1.0)), egui::Color32::WHITE);
            p.text(pos2(img_rect.center().x, img_rect.max.y-24.0), egui::Align2::CENTER_CENTER,
                "选择风格后点击 [处理]", egui::FontId::proportional(16.0), egui::Color32::WHITE.gamma_multiply(0.6));
        });

    }

}

// ============================================================
//  主入口
// ============================================================

fn main() -> eframe::Result<()> {
    let app_icon = load_app_icon();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size(Vec2::new(1360.0, 900.0))
            .with_min_inner_size(Vec2::new(900.0, 600.0))
            .with_icon(app_icon)
            .with_title("FilmRust Studio -- 胶片模拟工具 v4.0"),
        ..Default::default()
    };

    eframe::run_native("FilmRust Studio", options,
        Box::new(|cc| {
            setup_chinese_fonts(&cc.egui_ctx);
            setup_style(&cc.egui_ctx);
            Ok(Box::new(FilmRustGui::default()))
        }))
}
