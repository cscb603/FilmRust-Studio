//! FilmRust Studio Pro v5.7
//! 胶片基底(filmr全管线) + 色彩(warmth/tint/sat) + 曲线(Catmull-Rom LUT)
//! 曲线 LUT 与面板一致 · 胶片衰减特征 · 导出 JPG/PNG · EXIF保留 · 对比模式

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::{Arc, mpsc};
use std::time::Instant;

use eframe::egui::{self, Ui};
use egui::{vec2, pos2, Color32, ColorImage, CornerRadius, Frame, CentralPanel, IconData, Window};
use image::{DynamicImage, RgbImage};
use rfd::FileDialog;

use filmrust::layers::{BlendMode, Layer, LayerStack, LayerType};
use filmrust::presets::{get_all_presets, FilmPreset};
use filmrust::{apply_film, find_filmr_stock};
use filmr::SimulationConfig;

const WATERMARK: &str = "FilmRust Studio Pro";

fn load_app_icon() -> IconData {
    let png_bytes = include_bytes!("../../guitubiao.png");
    match image::load_from_memory(png_bytes) {
        Ok(img) => {
            let rgba = img.to_rgba8();
            let (w, h) = rgba.dimensions();
            IconData { rgba: rgba.into_raw(), width: w, height: h }
        }
        Err(_) => IconData::default(),
    }
}

fn setup_chinese_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    let candidates = [
        "/System/Library/Fonts/PingFang.ttc",
        "/System/Library/Fonts/STHeiti Medium.ttc",
        "/System/Library/Fonts/STHeiti Light.ttc",
        "/System/Library/Fonts/Hiragino Sans GB.ttc",
        r"C:\Windows\Fonts\msyh.ttc", r"C:\Windows\Fonts\msyh.ttf",
        r"C:\Windows\Fonts\msyhl.ttc", r"C:\Windows\Fonts\simhei.ttf",
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

fn tone_color(preset: &FilmPreset) -> Color32 {
    let lower = preset.name.to_lowercase();
    if lower.contains("portra")||lower.contains("gold")||lower.contains("kodachrome")||lower.contains("solaris")||lower.contains("vista")||lower.contains("optima") {Color32::from_rgb(220,160,80)}
    else if lower.contains("superia")||lower.contains("fujicolor")||lower.contains("cinestill 800t")||lower.contains("provia")||lower.contains("gr street") {Color32::from_rgb(80,160,220)}
    else if lower.contains("velvia")||lower.contains("ektar")||lower.contains("ektachrome") {Color32::from_rgb(200,80,160)}
    else if lower.contains("tri-x")||lower.contains("hp5")||lower.contains("fp4")||lower.contains("delta")||lower.contains("neopan")||lower.contains("pan")||lower.contains("apx")||lower.contains("orwo") {Color32::from_rgb(140,140,140)}
    else if lower.contains("lomo")||lower.contains("polaroid") {Color32::from_rgb(180,120,200)}
    else {Color32::from_rgb(120,160,120)}
}

fn film_usage_desc(key: &str) -> &'static str {
    let lower = key.to_lowercase();
    if lower.contains("kodak_portra_400") && !lower.contains("artistic") {
        "肤色表现自然柔和，曝光宽容度极大，圈内称「炮塔」。适合人像写真、婚礼跟拍。黄金时段逆光、室内混合光最佳"
    } else if lower.contains("gold_200") {
        "温暖金色基调，90年代家庭感，柯达最畅销的民用卷。适合旅行记录、家庭聚会。晴天户外、阳光海滩首选"
    } else if lower.contains("ektar_100") {
        "柯达最细腻颗粒负片，红色表现绝佳，色彩鲜艳锐利反差大。适合风光、静物。强光白天、高反差场景"
    } else if lower.contains("tri_x") && !lower.contains("artistic") {
        "黑白传奇，高对比粗颗粒强质感，街拍圣经。适合街头摄影、纪实新闻。白天街光可push到1600"
    } else if lower.contains("superia_400") || lower.contains("superia 400") {
        "富士性价比口粮卷，日系清新偏冷调，宽容度高。适合日常街拍、旅行随拍。阴天散射光表现好"
    } else if lower.contains("provia_100f") {
        "专业反转片，颗粒超细，色彩中性真实，万能ISO 100。适合时装、风光、产品摄影。日光下最佳"
    } else if lower.contains("velvia_50") && !lower.contains("artistic") {
        "风光首席，极致高饱和高对比，绿蓝表现惊人，1990年上市至今标杆。适合风光、自然、日出日落"
    } else if lower.contains("cinestill_800t") || lower.contains("800t") {
        "柯达电影卷去碳层版，高感钨丝灯卷，青橙调标志性色彩。适合夜景、城市灯光、霓虹灯、弱光环境"
    } else if lower.contains("cinestill_50d") || lower.contains("50d") {
        "日光型电影卷，低感细腻，电影感色调过渡柔和。适合白天街拍、柔和日光人像"
    } else if lower.contains("hp5") && !lower.contains("artistic") {
        "黑白经典，粗颗粒高宽容度，可push使用，纪录片首选。适合纪实、街头、新闻·几乎任何光线"
    } else if lower.contains("standard_daylight") {
        "中性基准风格，去胶片化原色校准。适合色彩校准参考、不想有胶片感时使用"
    } else if lower.contains("lomography_color_chrome") || lower.contains("lomo") {
        "LOMO艺术调，高对比大胆偏色。适合创意摄影、社交媒体、艺术表达"
    } else if lower.contains("polaroid_600_color") || lower.contains("polaroid 600") {
        "宝丽来即时显影质感，暖调柔和褪色怀旧。适合聚会抓拍、趣味记录、生活小物"
    } else if lower.contains("portra_160") {
        "低感版Portra，更细腻的肤色过渡。适合棚拍人像、强光人像，比400更平滑"
    } else if lower.contains("portra_800") {
        "高感版Portra，温暖颗粒感。适合弱光人像、室内环境、黄昏街头"
    } else if lower.contains("portra_400_artistic") {
        "增强版Portra，色彩分离更强。适合需要更浓郁Portra色调的创意人像"
    } else if lower.contains("superia_200") {
        "富士Superia低感版，暖调日系清新。适合户外人像、阳光明媚的日常"
    } else if lower.contains("superia_100") {
        "富士Superia最低感，细腻柔和。适合强光下的人像、静物"
    } else if lower.contains("agfa_vista") {
        "德系暖调，浓郁色彩，停产经典。适合人像、街拍。阳光下暖意更足"
    } else if lower.contains("lucky_color_200") {
        "国产乐凯彩色卷，暖调怀旧，性价比高。适合日常记录、怀旧主题"
    } else if lower.contains("ultramax") {
        "柯达消费级卷王，暖调高饱和·浓郁色彩·宽曝光宽容度。适合旅行、家庭、日常街拍，阴天和夜间闪光也强"
    } else if lower.contains("pro_400h") || lower.contains("pro 400h") {
        "富士人像专业卷（已停产），冷蓝阴影·暖粉高光·细颗粒·肤色柔和。擅长人像写真、婚礼跟拍、过曝两档出粉彩效果"
    } else if lower.contains("natura_1600") || lower.contains("natura 1600") {
        "富士月光卷（日本限定），ISO 1600超高速·暖调浓郁·青绿阴影·颗粒控制出色。适合弱光人像、夜景街拍、室内抓拍，可迫冲到3200"
    } else if lower.contains("ektachrome_100vs") || lower.contains("100vs") {
        "Ektachrome超鲜艳版，极致色彩饱和度。适合风光、花卉、日落"
    } else if lower.contains("ektachrome_100") {
        "经典柯达反转片，色彩浓郁漂亮，2019年复产。适合风光、旅行、记录"
    } else if lower.contains("kodachrome_64") {
        "传奇柯达克罗姆暖调浓郁，已停产但色彩风格永存。适合暖调风光、怀旧"
    } else if lower.contains("kodachrome_25") {
        "极致细腻柯达克罗姆，曾是最细颗粒彩色反转片。适合极致画质需求"
    } else if lower.contains("velvia_50_artistic") {
        "增强版Velvia，更极致鲜艳。适合风光大片、艺术表达"
    } else if lower.contains("astia_100f") {
        "富士柔和反转片，淡彩低对比，皮肤过渡平滑。适合人像、柔和风光"
    } else if lower.contains("optima_200") {
        "爱克发暖调反转片，德系色彩风格。适合风光、旅行"
    } else if lower.contains("precisa_100") {
        "爱克发暖调反转片，通用型。适合风光、人像兼顾"
    } else if lower.contains("tri_x_400_artistic") {
        "增强版Tri-X，更强颗粒和对比。适合更粗犷的黑白表达"
    } else if lower.contains("plus_x_125") {
        "柯达中速黑白，中调丰富细腻。适合人像、风光通用"
    } else if lower.contains("hp5_plus_400_artistic") {
        "增强版HP5，颗粒更突出。适合粗颗粒黑白艺术创作"
    } else if lower.contains("fp4_plus_125") {
        "伊尔福中速黑白，细腻过渡。适合人像、风光、静物"
    } else if lower.contains("delta_400") {
        "伊尔福现代黑白，颗粒锐利清晰度好。适合风光、纪实、建筑"
    } else if lower.contains("delta_100") {
        "伊尔福超细腻现代黑白，可作对比度标准。适合高画质需求"
    } else if lower.contains("pan_f_plus_50") {
        "伊尔福极细腻低感卷，风光专用。适合大画幅、风光"
    } else if lower.contains("xp2_super") {
        "C41工艺黑白，彩色店也能冲。适合混合冲洗需求的用户"
    } else if lower.contains("sfx_200") {
        "红外效果黑白，独特质感让树叶发白。适合创意黑白艺术"
    } else if lower.contains("ortho_plus_80") {
        "正色片，高对比反差，红光下不感光。适合高反差艺术"
    } else if lower.contains("neopan_400") {
        "富士日系黑白，细腻灰阶。适合街拍、人像"
    } else if lower.contains("neopan_100") {
        "富士日系低感黑白，极致细腻。适合风光、静物"
    } else if lower.contains("apx_400") {
        "爱克发经典德系黑白，宽泛灰度。适合街拍、纪实"
    } else if lower.contains("apx_100") {
        "爱克发经典细腻黑白。适合人像、风光、静物"
    } else if lower.contains("polaroid_bw_667") {
        "宝丽来黑白即时显影，独特质感。适合即时街拍、创意"
    } else if lower.contains("polaroid_55_bw") {
        "宝丽来正负片，一张出正片+负片。适合追求极限画质的宝丽来用户"
    } else if lower.contains("orwo_un54") {
        "东德经典黑白，高对比。适合复古风格、艺术创作"
    } else if lower.contains("orwo_un64") {
        "东德低感黑白，细腻。适合人像、静物"
    } else if lower.contains("gr_street") {
        "街拍高感黑白，粗颗粒城市感。适合夜晚街拍、城市纪实"
    } else if lower.contains("scala_200") {
        "黑白反转片，高反差，可直接放映。适合高对比艺术"
    } else if lower.contains("sx70_color") {
        "SX-70经典暖调柔和，宝丽来宽幅。适合室内、创意静物"
    } else if lower.contains("i_type") {
        "现代宝丽来鲜艳色彩。适合聚会、创意、日常"
    } else if lower.contains("spectra_color") {
        "宽幅宝丽来偏冷调。适合风景、创意构图"
    } else if lower.contains("polaroid_100_color") || lower.contains("polaroid 100") {
        "老式宝丽来褪色怀旧。适合复古创意、文艺表达"
    } else if lower.contains("lomochrome_purple") {
        "紫调幻彩，经独特化学配方产生紫色偏色。适合创意艺术、超现实"
    } else if lower.contains("solaris_400") {
        "意式暖调复古褪色感。适合旅行怀旧、文艺街拍"
    } else if lower.contains("solaris_100") {
        "意式低感暖调柔和。适合人像、静物"
    } else {
        ""
    }
}

fn tone_label(p: &FilmPreset) -> &'static str {
    let s = p.name.to_lowercase();
    if s.contains("portra")||s.contains("gold")||s.contains("kodachrome") {"暖色调"}
    else if s.contains("superia")||s.contains("fujicolor") {"冷调·青绿"}
    else if s.contains("cinestill 800t") {"冷调·蓝钨丝"}
    else if s.contains("velvia") {"高饱和·绿"}
    else if s.contains("ektar") {"高饱和·暖"}
    else if s.contains("ektachrome") {"正片·冷"}
    else if s.contains("provia") {"正片·中性"}
    else if s.contains("tri-x")||s.contains("hp5") {"黑白·高反差"}
    else if s.contains("fp4")||s.contains("delta")||s.contains("pan"){"黑白·细颗粒"}
    else if s.contains("solaris")||s.contains("vista"){"暖调·复古"}
    else if s.contains("lomo") {"创意·Lomo"}
    else if s.contains("polaroid") {"暖调·拍立得"}
    else {"中性"}
}

fn layer_tag(lt: &LayerType) -> &'static str {
    match lt { LayerType::FilmBase{..}=>"[胶片]", LayerType::Color{..}=>"[色彩]", LayerType::Curves{..}=>"[曲线]", _=>"[--]" }
}

// ============================================================
struct FilmRustPro {
    files: Vec<PathBuf>, selected_idx: usize,
    last_dir: Option<PathBuf>,
    original_img: Option<DynamicImage>,
    original_tex: Option<egui::TextureHandle>, processed_tex: Option<egui::TextureHandle>,
    display_img_w: u32, display_img_h: u32,
    is_processing: bool, has_processed: bool,
    proc_result_rx: Option<mpsc::Receiver<ProcessResult>>,
    animating: bool, anim_start: Instant, anim_duration: f32,
    anim_src: Option<image::RgbaImage>, anim_dst: Option<image::RgbaImage>,
    processed_base: Option<RgbImage>,
    presets: Vec<FilmPreset>, style_idx: usize,
    layers: LayerStack, selected_layer: Option<usize>,
    status: String, status_ok: bool,
    dark_mode: bool,
    show_curves_overlay: bool, curve_drag: Option<usize>, curve_cx: [f32; 3],
    comparison_mode: bool, split_pos: f32,
}

struct ProcessResult { ok: bool, image: Option<DynamicImage>, error: Option<String> }

impl FilmRustPro {
    fn guide_msg(&self) -> &'static str {
        if self.files.is_empty() { "拖拽图片到窗口任意位置，或点「打开文件」开始" }
        else if self.original_img.is_none() { "点击图片列表切换照片" }
        else if !self.has_processed && !self.is_processing { "左侧选好胶片风格 → 右侧调整参数 → 点「开始显影」预览效果" }
        else if self.is_processing { "正在显影中，请稍候..." }
        else if self.has_processed { "可继续调参数重新显影，或点「导出」保存到本地" }
        else { "就绪" }
    }

    fn bg_top(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(28,32,38)} else {Color32::from_rgb(240,240,245)} }
    fn bg_bottom(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(22,26,32)} else {Color32::from_rgb(230,230,238)} }
    fn bg_panel(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(24,28,34)} else {Color32::from_rgb(245,245,250)} }
    fn bg_center(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(18,20,26)} else {Color32::from_rgb(250,250,252)} }
    fn bg_layer(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(32,36,42)} else {Color32::from_rgb(235,238,242)} }
    fn bg_layer_sel(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(50,60,70)} else {Color32::from_rgb(200,210,225)} }
    fn text_accent(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(200,180,140)} else {Color32::from_rgb(140,100,40)} }
    fn text_ok(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(140,200,140)} else {Color32::from_rgb(40,140,40)} }
    fn text_err(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(240,140,140)} else {Color32::from_rgb(200,40,40)} }
    fn text_dim(&self) -> Color32 { if self.dark_mode {Color32::from_rgb(140,150,160)} else {Color32::from_rgb(120,120,130)} }

    fn new(_cc: &eframe::CreationContext) -> Self {
        let presets = get_all_presets();
        let sid = presets.first().map(|p| p.id.clone()).unwrap_or_default();
        let (def_w, def_t, def_s) = presets.first().map(|p| (p.default_warmth, p.default_tint, p.default_saturation)).unwrap_or((0.0, 0.0, 1.0));
        let mut layers = LayerStack::new();
        layers.add(Layer::new("胶片基底".into(), LayerType::FilmBase { stock_id: sid, strength: 100.0, grain: 100.0, auto_levels: true }));
        layers.add(Layer::new("色彩".into(), LayerType::Color { warmth: def_w, tint: def_t, saturation: def_s }));
        layers.add(Layer::new("曲线".into(), LayerType::Curves { contrast: 0.0, highlights: 0.0, shadows: 0.0 }));
        Self {
            files: vec![], selected_idx: 0, last_dir: None,
            original_img: None, original_tex: None, processed_tex: None,
            display_img_w:0, display_img_h:0, is_processing:false, has_processed:false,
            proc_result_rx:None, animating:false, anim_start:Instant::now(), anim_duration:1.5,
            anim_src:None, anim_dst:None, processed_base:None,
            presets, style_idx:0, layers, selected_layer:Some(0),
            status:"就绪".into(), status_ok:true, dark_mode:true,
            show_curves_overlay:false, curve_drag:None, curve_cx:[0.25,0.5,0.75],
            comparison_mode:false, split_pos:0.5,
        }
    }

    fn current_preset(&self) -> Option<&FilmPreset> { self.presets.get(self.style_idx) }
    fn film_base(&self) -> Option<&Layer> { self.layers.layers.iter().find(|l| matches!(l.layer_type, LayerType::FilmBase{..})) }

    /// 切换到指定索引的预设，并应用默认校色（warmth/tint/saturation）到 Color 层
    fn set_preset_index(&mut self, idx: usize) {
        self.style_idx = idx;
        if let Some(p) = self.presets.get(idx) {
            if let Some(l) = self.layers.layers.iter_mut().find(|l| matches!(l.layer_type, LayerType::FilmBase{..})) {
                if let LayerType::FilmBase{stock_id,..} = &mut l.layer_type { *stock_id = p.id.clone(); }
            }
            for l in &mut self.layers.layers {
                if let LayerType::Color{ warmth, tint, saturation } = &mut l.layer_type {
                    *warmth = p.default_warmth;
                    *tint = p.default_tint;
                    *saturation = p.default_saturation;
                }
            }
        }
    }

    fn film_base_params(&self) -> (String, f32, f32, bool) {
        match self.film_base() {
            Some(l) => match &l.layer_type { LayerType::FilmBase{stock_id,strength,grain,auto_levels}=>(stock_id.clone(),*strength,*grain,*auto_levels), _ => (String::new(),100.0,100.0,true) },
            None => (String::new(),100.0,100.0,true)
        }
    }

    fn load_image_for_display(&mut self, ctx: &egui::Context) {
        if let Some(p) = self.files.get(self.selected_idx) {
            match image::open(p) {
                Ok(img) => {
                    let (w,h)=(img.width(),img.height());
                    let s = if w>h {800.0/w as f32} else {600.0/h as f32};
                    let scaled = img.resize((w as f32*s) as u32,(h as f32*s) as u32,image::imageops::FilterType::Lanczos3);
                    let rgba=scaled.to_rgba8(); let (rw,rh)=(rgba.width() as _,rgba.height() as _);
                    self.original_tex=Some(ctx.load_texture("orig",ColorImage::from_rgba_unmultiplied([rw,rh],rgba.as_raw()),egui::TextureOptions::LINEAR));
                    self.original_img=Some(scaled); self.display_img_w=rgba.width(); self.display_img_h=rgba.height();
                    self.has_processed=false; self.processed_tex=None; self.processed_base=None; self.comparison_mode=false;
                    self.last_dir = p.parent().map(|d| d.to_path_buf());
                    self.status=format!("已加载: {} ({}x{})",p.file_name().unwrap_or_default().to_string_lossy(),w,h); self.status_ok=true;
                }
                Err(e)=>{self.status=format!("加载失败: {}",e);self.status_ok=false;}
            }
        }
    }

    fn do_process(img: &DynamicImage, stock_id: &str, _strength: f32, grain: f32, auto_levels: bool) -> ProcessResult {
        match find_filmr_stock(stock_id) {
            Ok(stock) => {
                let config = SimulationConfig {
                    exposure_time:1.0, auto_levels, white_balance_mode:filmr::WhiteBalanceMode::Off,
                    enable_grain:grain>5.0, motion_blur_amount:0.0, object_motion_amount:0.0,
                    light_leak:filmr::light_leak::LightLeakConfig{enabled:false,leaks:vec![]},
                    warmth:0.0, saturation:1.0, ..Default::default()
                };
                match apply_film(&img.to_rgb8(), &stock, &config) {
                    Ok(r) => ProcessResult{ok:true, image:Some(DynamicImage::ImageRgb8(r)), error:None},
                    Err(e) => ProcessResult{ok:false, image:None, error:Some(e.to_string())},
                }
            }
            Err(e) => ProcessResult{ok:false, image:None, error:Some(e.to_string())},
        }
    }

    fn trigger_develop(&mut self, ctx: &egui::Context) {
        if self.is_processing || self.original_img.is_none() { return; }
        let img = self.original_img.clone().unwrap();
        let (sid, strength, grain, auto_levels) = self.film_base_params();
        self.is_processing = true; self.animating = false; self.comparison_mode = false;
        self.anim_src = Some(img.to_rgba8());
        let (tx, rx) = mpsc::channel(); self.proc_result_rx = Some(rx);
        std::thread::spawn(move || {
            let r = Self::do_process(&img, &sid, strength, grain, auto_levels);
            let _ = tx.send(r);
        });
        ctx.request_repaint();
    }

    fn check_process_result(&mut self, _ctx: &egui::Context) {
        if let Some(rx) = &self.proc_result_rx {
            if let Ok(r) = rx.try_recv() {
                self.is_processing = false; self.proc_result_rx = None;
                if r.ok {
                    if let Some(img) = r.image {
                        self.processed_base = Some(img.to_rgb8());
                        self.anim_dst = Some(img.to_rgba8());
                        self.animating = true; self.anim_start = Instant::now();
                        self.has_processed = true;
                        self.status = "处理完成 — 可调整图层参数实时预览".into(); self.status_ok = true;
                    }
                } else {
                    self.status = format!("处理失败: {}", r.error.unwrap_or_default()); self.status_ok = false;
                }
            }
        }
    }

    fn render_developing_frame(&self, t: f32) -> Option<image::RgbaImage> {
        let src = self.anim_src.as_ref()?; let dst = self.anim_dst.as_ref()?;
        let mut out = src.clone();
        let eased = if t<0.5 {2.0*t*t} else {-1.0+(4.0-2.0*t)*t};
        for (s,(d,o)) in src.pixels().zip(dst.pixels().zip(out.pixels_mut())) {
            o[0]=lerp_u8(s[0],d[0],eased); o[1]=lerp_u8(s[1],d[1],eased); o[2]=lerp_u8(s[2],d[2],eased);
        }
        Some(out)
    }

    fn do_export_one(&mut self, path: &PathBuf, fmt: ExportFormat) -> bool {
        let full = match image::open(path) { Ok(f) => f, Err(e) => { self.status = format!("打开失败: {}", e); self.status_ok = false; return false; } };
        let (sid,s,grain,auto_levels) = self.film_base_params();
        let r = Self::do_process(&full, &sid, s, grain, auto_levels);
        if !r.ok { self.status = format!("处理失败: {}", r.error.unwrap_or_default()); self.status_ok = false; return false; }
        let proc = r.image.unwrap();
        let rgb = proc.to_rgb8();
        let comp = self.layers.composite(&rgb);
        let stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let dir = self.last_dir.as_ref().cloned().unwrap_or_else(|| {
            path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."))
        });
        let ext = if matches!(fmt, ExportFormat::Jpeg) { ".jpg" } else { ".png" };
        let out = dir.join(format!("{}_filmrust{}", stem, ext));
        let ok = match fmt {
            ExportFormat::Jpeg => {
                let jpg = DynamicImage::ImageRgb8(comp).to_rgb8();
                let mut buf = Vec::new();
                if DynamicImage::ImageRgb8(jpg).write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Jpeg).is_ok() {
                    let final_data = preserve_jpeg_exif(path, buf);
                    std::fs::write(&out, final_data).is_ok()
                } else { false }
            }
            ExportFormat::Png => comp.save(&out).is_ok(),
        };
        if ok { self.status = format!("已导出: {}", out.display()); self.status_ok = true; } else { self.status = format!("导出失败: {}", out.display()); self.status_ok = false; }
        ok
    }
}

fn preserve_jpeg_exif(input_path: &Path, output_data: Vec<u8>) -> Vec<u8> {
    use img_parts::jpeg::Jpeg;
    use img_parts::Bytes;
    std::fs::read(input_path).ok().and_then(|input_bytes| {
        Jpeg::from_bytes(Bytes::from(input_bytes)).ok().and_then(|input_jpeg| {
            input_jpeg.segments().iter().find(|s| s.marker() == 0xE1).cloned().and_then(|exif_seg| {
                Jpeg::from_bytes(Bytes::from(output_data.clone())).ok().map(|mut output_jpeg| {
                    output_jpeg.segments_mut().insert(1, exif_seg);
                    output_jpeg.encoder().bytes().to_vec()
                })
            })
        })
    }).unwrap_or(output_data)
}

#[derive(Clone, Copy, PartialEq)]
enum ExportFormat { Jpeg, Png }

fn lerp_u8(a:u8,b:u8,t:f32)->u8 {(a as f32+(b as f32 - a as f32)*t).clamp(0.0,255.0) as u8}

// ============================================================
impl eframe::App for FilmRustPro {
    fn ui(&mut self, ui: &mut Ui, _frame: &mut eframe::Frame) {
        // 拖拽导入
        let dropped: Vec<PathBuf> = ui.ctx().input(|i| i.raw.dropped_files.iter().filter_map(|df| df.path.clone()).collect());
        if !dropped.is_empty() {
            for p in dropped { if !self.files.contains(&p) { self.files.push(p); } }
            self.selected_idx = self.files.len().saturating_sub(1);
            self.load_image_for_display(ui.ctx());
        }

        self.check_process_result(ui.ctx());

        if self.has_processed || self.animating {
            let base: Option<RgbImage> = if self.animating {
                let t = (self.anim_start.elapsed().as_secs_f32()/self.anim_duration).min(1.0);
                if let Some(frame) = self.render_developing_frame(t) {
                    if t>=1.0 { self.animating = false; }
                    Some(DynamicImage::ImageRgba8(frame).to_rgb8())
                } else { None }
            } else { self.processed_base.clone() };
            if let Some(rgb) = base {
                let comp = self.layers.composite(&rgb);
                let rgba = DynamicImage::ImageRgb8(comp).to_rgba8();
                let (rw,rh) = (rgba.width() as _, rgba.height() as _);
                self.processed_tex = Some(ui.ctx().load_texture("live",
                    ColorImage::from_rgba_unmultiplied([rw,rh],rgba.as_raw()), egui::TextureOptions::LINEAR));
            }
            ui.ctx().request_repaint();
        }

        let cr=CornerRadius::same(12u8); let pad:i8=12;

        if self.show_curves_overlay { self.render_curves_overlay(ui); }

        egui::Panel::top("tb").frame(Frame{corner_radius:cr,fill:self.bg_top(),inner_margin:egui::Margin::same(pad),..Default::default()})
        .show_inside(ui,|ui|{ ui.horizontal(|ui|{
            ui.label(egui::RichText::new(WATERMARK).size(18.0).color(self.text_accent()));
            ui.separator();
            if ui.button("打开文件").on_hover_text("选择图片文件（支持多选），也可直接拖拽到窗口").clicked(){ if let Some(ps)=FileDialog::new().add_filter("图片",&["png","jpg","jpeg","tiff","tif","bmp","webp"]).pick_files(){self.files=ps;if !self.files.is_empty(){self.selected_idx=0;self.load_image_for_display(ui.ctx());}}}
            if !self.files.is_empty() {
                ui.menu_button("导出", |ui| {
                    if ui.button("导出当前 — JPG (高质量)").on_hover_text("以高质量 JPG 导出当前照片，保留 EXIF 信息").clicked() { if let Some(p)=self.files.get(self.selected_idx).cloned(){self.do_export_one(&p,ExportFormat::Jpeg);} ui.close(); }
                    if ui.button("导出当前 — PNG (无损)").on_hover_text("以无损 PNG 导出当前照片").clicked() { if let Some(p)=self.files.get(self.selected_idx).cloned(){self.do_export_one(&p,ExportFormat::Png);} ui.close(); }
                    ui.separator();
                    if ui.button("导出全部 — JPG (高质量)").on_hover_text("批量导出所有图片为 JPG").clicked() { let paths:Vec<_>=self.files.clone(); for p in &paths{self.do_export_one(p,ExportFormat::Jpeg);} self.status=format!("已导出 {} 张图片",paths.len()); self.status_ok=true; ui.close(); }
                    if ui.button("导出全部 — PNG (无损)").on_hover_text("批量导出所有图片为 PNG").clicked() { let paths:Vec<_>=self.files.clone(); for p in &paths{self.do_export_one(p,ExportFormat::Png);} self.status=format!("已导出 {} 张图片",paths.len()); self.status_ok=true; ui.close(); }
                }).response.on_hover_text("默认输出到原图目录，或点「导出到...」指定位置");
                if ui.small_button("导出到...").on_hover_text("选择自定义输出目录，设置后以后默认记住此位置").clicked(){ if let Some(dir)=FileDialog::new().pick_folder(){self.last_dir=Some(dir);self.status=format!("导出目录: {}",self.last_dir.as_ref().unwrap().display());self.status_ok=true;} }
                let dir_label=self.last_dir.as_ref().map(|d|d.file_name().unwrap_or_default().to_string_lossy().to_string()).unwrap_or_else(||"原图目录".into());
                ui.label(egui::RichText::new(format!("→ {}",dir_label)).size(11.0).color(self.text_dim()));
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                if ui.button(if self.dark_mode{"浅色"}else{"深色"}).on_hover_text("切换深色/浅色主题").clicked(){self.dark_mode = !self.dark_mode;}
            });
        });});

        egui::Panel::bottom("sb").frame(Frame{corner_radius:cr,fill:self.bg_bottom(),inner_margin:egui::Margin::symmetric(pad,8),..Default::default()})
        .show_inside(ui,|ui|{ ui.horizontal(|ui|{
            let guide = self.guide_msg();
            ui.label(egui::RichText::new(&self.status).size(13.0).color(if self.status_ok{self.text_ok()}else{self.text_err()}));
            ui.label(egui::RichText::new(guide).size(12.0).color(self.text_dim()));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                ui.label(egui::RichText::new("星TAP软件 2026 | csb603@qq.com").size(11.0).color(self.text_dim()));
            });
        });});

        egui::Panel::left("fp").resizable(true).default_size(250.0).min_size(200.0)
            .frame(Frame{corner_radius:cr,fill:self.bg_panel(),inner_margin:egui::Margin::same(pad),..Default::default()})
            .show_inside(ui,|ui|{self.render_file_panel(ui);});

        egui::Panel::right("lp").resizable(true).default_size(270.0).min_size(230.0)
            .frame(Frame{corner_radius:cr,fill:self.bg_panel(),inner_margin:egui::Margin::same(pad),..Default::default()})
            .show_inside(ui,|ui|{self.render_layer_panel(ui);});

        CentralPanel::default().frame(Frame{corner_radius:cr,fill:self.bg_center(),inner_margin:egui::Margin::same(pad),..Default::default()})
            .show_inside(ui,|ui|{self.render_preview(ui);});
    }
}

// ============================================================
impl FilmRustPro {
    fn render_file_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("文件列表").size(16.0).color(self.text_accent()));
        ui.horizontal(|ui|{
            if ui.button("添加图片").on_hover_text("选择更多图片添加到列表，支持批量处理").clicked(){ if let Some(ps)=FileDialog::new().add_filter("图片",&["png","jpg","jpeg","tiff","tif","bmp","webp"]).pick_files(){for p in ps{if !self.files.contains(&p){self.files.push(p);}}if self.files.len()==1{self.selected_idx=0;self.load_image_for_display(ui.ctx());}}}
            if ui.button("清空").on_hover_text("清空图片列表，重新开始").clicked(){self.files.clear();self.selected_idx=0;}
        });
        ui.add_space(6.0);
        let mut to_rem: Option<usize> = None; let mut to_sel: Option<usize> = None;
        egui::ScrollArea::vertical().max_height(280.0).show(ui,|ui|{
            for i in 0..self.files.len() {
                let name=self.files[i].file_name().unwrap_or_default().to_string_lossy();
                let is_sel=self.selected_idx==i;
                Frame::NONE.fill(if is_sel{self.bg_layer_sel()}else{self.bg_layer()}).corner_radius(CornerRadius::same(6u8)).inner_margin(egui::Margin::symmetric(8,4)).show(ui,|ui|{ui.horizontal(|ui|{
                    if ui.selectable_label(is_sel,name).on_hover_text(if is_sel{"当前选中的照片"}else{"切换到此照片"}).clicked(){to_sel=Some(i);}
                    if ui.small_button("删除").on_hover_text("从列表中移除此图片").clicked(){to_rem=Some(i);}
                });});
            }
        });
        if let Some(i)=to_sel{self.selected_idx=i;self.load_image_for_display(ui.ctx());}
        if let Some(i)=to_rem{self.files.remove(i);if self.selected_idx>=self.files.len(){self.selected_idx=self.files.len().saturating_sub(1);}}

        ui.add_space(8.0); ui.separator();
        ui.heading(egui::RichText::new("胶片选择").size(16.0).color(self.text_accent()));
        let cn = self.current_preset().map(|p|p.name.clone()).unwrap_or_default();
        // 常用胶片快速选择
        let popular = ["Ultramax 400", "Pro 400H", "Natura 1600", "Portra 400", "Gold 200"];
        ui.horizontal_wrapped(|ui| {
            for name in &popular {
                let is_active = cn == *name;
                if ui.selectable_label(is_active, egui::RichText::new(*name).size(12.0)).clicked() {
                    if let Some(idx) = self.presets.iter().position(|p| p.name == *name) {
                        self.set_preset_index(idx);
                    }
                }
            }
        });
        ui.add_space(2.0);
        let mut clicked_idx: Option<usize> = None;
        egui::ComboBox::from_id_salt("stock").width(210.0).selected_text(&cn).show_ui(ui,|ui|{
            let h = (self.presets.len() as f32 * 24.0 + 8.0).min(320.0);
            egui::ScrollArea::vertical().max_height(h).show(ui, |ui| {
                for (i,p) in self.presets.iter().enumerate() {
                    if ui.selectable_label(false,&p.name).clicked() {
                        clicked_idx = Some(i);
                    }
                }
            });
        });
        if let Some(i) = clicked_idx { self.set_preset_index(i); }
        if let Some(p)=self.current_preset() {
            ui.add_space(4.0);
            let tc=tone_color(p); let tl=tone_label(p);
            ui.label(egui::RichText::new(format!("色调: {}",tl)).size(14.0).color(tc));
            ui.label(egui::RichText::new(&p.description).size(12.0).color(self.text_dim()));
            let usage = film_usage_desc(&p.id);
            if !usage.is_empty() {
                ui.label(egui::RichText::new(usage).size(11.0).color(self.text_dim())).on_hover_text("基于真实胶片的使用经验和网络资料整理");
            }
            ui.horizontal_wrapped(|ui|{for t in &p.tags{ui.label(egui::RichText::new(format!("[{}]",t)).size(11.0).color(self.text_dim()));}});
        }
    }
}

// ============================================================
impl FilmRustPro {
    fn render_layer_panel(&mut self, ui: &mut egui::Ui) {
        ui.heading(egui::RichText::new("图层").size(16.0).color(self.text_accent()));
        ui.label(egui::RichText::new("叠层顺序: 色彩 → 曲线 → 基底").size(10.0).color(self.text_dim()));
        ui.add_space(2.0); ui.separator();
        let nl = self.layers.layers.len();
        let display_order: &[usize] = if nl == 3 { &[1, 2, 0] } else { &[0, 1, 2, 3, 4, 5, 6] };
        egui::ScrollArea::vertical().max_height(380.0).show(ui,|ui|{
            for &i in display_order {
                if i >= nl { continue; }
                let is_sel=self.selected_layer==Some(i);
                let vis=self.layers.layers[i].visible;
                let lt=self.layers.layers[i].layer_type.clone();
                let nm=self.layers.layers[i].name.clone();
                Frame::NONE.fill(if is_sel{self.bg_layer_sel()}else{self.bg_layer()}).corner_radius(CornerRadius::same(8u8)).inner_margin(egui::Margin::symmetric(8,6)).show(ui,|ui|{
                    ui.horizontal(|ui|{
                        if ui.selectable_label(false,if vis{"可见"}else{"隐藏"}).clicked(){self.layers.layers[i].visible = !vis;}
                        if ui.selectable_label(is_sel,format!("{} {}",layer_tag(&lt),nm)).clicked(){self.selected_layer=Some(i);}
                    });
                    if !matches!(lt, LayerType::FilmBase{..}) {
                        ui.horizontal(|ui|{
                            let mode=&mut self.layers.layers[i].blend_mode;
                            egui::ComboBox::from_id_salt(format!("bm_{}",i)).width(90.0).selected_text(mode.label()).show_ui(ui,|ui|{
                                for m in BlendMode::ALL { if ui.selectable_label(*m==*mode,m.label()).clicked(){*mode = *m;} }
                            });
                            ui.add(egui::Slider::new(&mut self.layers.layers[i].opacity,0.0..=1.0).text("透明度"));
                        });
                    }
                });
            }
        });
        ui.add_space(6.0);
        if let Some(i)=self.selected_layer { self.render_layer_properties(ui,i); }
    }

    fn render_layer_properties(&mut self, ui: &mut egui::Ui, idx: usize) {
        let accent=self.text_accent(); let td=self.text_dim();
        ui.separator();
        ui.label(egui::RichText::new("层属性").size(14.0).color(accent));
        let mut new_warmth_tint: Option<(f32, f32, f32)> = None;
        let layer=&mut self.layers.layers[idx];
        match &mut layer.layer_type {
            LayerType::FilmBase{stock_id,strength,grain,auto_levels}=>{
                ui.label("胶片类型:");
                let cn=self.presets.iter().find(|p|p.id==*stock_id).map(|p|p.name.clone()).unwrap_or_default();
                let preset_names: Vec<String> = self.presets.iter().map(|p| p.name.clone()).collect();
                let mut clicked_idx: Option<usize> = None;
                egui::ComboBox::from_id_salt("stockp").width(200.0).selected_text(&cn).show_ui(ui,|ui|{
                    let h = (preset_names.len() as f32 * 24.0 + 8.0).min(320.0);
                    egui::ScrollArea::vertical().max_height(h).show(ui, |ui| {
                        for (i, name) in preset_names.iter().enumerate(){
                            if ui.selectable_label(false, name).clicked(){
                                clicked_idx = Some(i);
                            }
                        }
                    });
                });
                if let Some(i) = clicked_idx {
                    *stock_id = self.presets[i].id.clone();
                    self.style_idx = i;
                    new_warmth_tint = Some((self.presets[i].default_warmth, self.presets[i].default_tint, self.presets[i].default_saturation));
                }
                if let Some(p)=self.presets.get(self.style_idx) {
                    ui.add_space(2.0);
                    ui.label(egui::RichText::new(format!("色调: {}",tone_label(p))).size(13.0).color(tone_color(p)));
                    ui.label(egui::RichText::new(&p.description).size(12.0).color(td));
                }
                ui.add_space(4.0);
                ui.add(egui::Slider::new(strength,0.0..=150.0).text("强度").suffix("%")).on_hover_text("越大胶片味越浓。100%=真实胶片效果，150%=强化版");
                ui.add(egui::Slider::new(grain,0.0..=200.0).text("颗粒").suffix("%")).on_hover_text("模拟真实银盐颗粒。100%=原生颗粒量，0=无颗粒");
                ui.checkbox(auto_levels,"自动色阶");
            }
            LayerType::Color{warmth,tint,saturation}=>{
                ui.add(egui::Slider::new(warmth,-1.0..=1.0).text("色温")).on_hover_text("向右→画面偏黄偏暖，向左→画面偏蓝偏冷。调节量适中就行，别拉太猛");
                ui.add(egui::Slider::new(tint,-1.0..=1.0).text("色调")).on_hover_text("向右→画面偏品红，向左→画面偏绿。微调肤色和天空很管用");
                ui.add(egui::Slider::new(saturation,0.0..=2.0).text("饱和度")).on_hover_text("1.0=跟原图一样，大于1更鲜艳，小于1更清淡。不会死黑死白，过渡自然");
                ui.add_space(2.0);
                if ui.button("↺ 复位默认").on_hover_text("重置为该胶卷的扫片校色默认值").clicked() {
                    if let Some(p) = self.presets.get(self.style_idx) {
                        *warmth = p.default_warmth;
                        *tint = p.default_tint;
                        *saturation = p.default_saturation;
                    }
                }
                ui.add_space(2.0);
                ui.label(egui::RichText::new("调节后点击「开始显影」即可生效").size(11.0).color(td));
            }
            LayerType::Curves{contrast,highlights,shadows}=>{
                ui.add(egui::Slider::new(contrast,-1.0..=1.0).text("中间调")).on_hover_text("正=中间调变柔(反差低) 负=中间调变硬(反差高)");
                ui.add(egui::Slider::new(highlights,-1.0..=1.0).text("高光")).on_hover_text("正=高光提亮 负=高光压缩 · 锚点往上拉变亮，往下拉变暗");
                ui.add(egui::Slider::new(shadows,-1.0..=1.0).text("阴影")).on_hover_text("正=阴影提亮(暗部细节) 负=阴影加深 · 锚点往上拉变亮，往下拉变暗");
                ui.add_space(4.0);
                if ui.button("打开曲线可视化面板").on_hover_text("弹出可拖拽的曲线调整窗口，支持拖拽锚点微调").clicked(){self.show_curves_overlay=true;}
            }
            _=>{}
        }
        // 切换预设后更新 Color 层默认校色
        if let Some((w, t, s)) = new_warmth_tint {
            for l in &mut self.layers.layers {
                if let LayerType::Color{ warmth, tint, saturation } = &mut l.layer_type {
                    *warmth = w;
                    *tint = t;
                    *saturation = s;
                }
            }
        }
        ui.add_space(8.0);
        ui.horizontal(|ui|{
            if ui.button(egui::RichText::new("开始显影").size(16.0).color(accent)).on_hover_text("将胶片风格 + 色温/色调/饱和度 + 曲线全部写入像素生成效果预览").clicked(){self.trigger_develop(ui.ctx());}
            if self.has_processed && self.original_tex.is_some() {
                let cmp_lbl = if self.comparison_mode { "▌对比中" } else { "▌对比" };
                if ui.button(egui::RichText::new(cmp_lbl).size(14.0)).on_hover_text(if self.comparison_mode{"关闭对比模式，退出左右分割对比"}else{"打开对比模式：左=原图 右=处理后，拖拽分割线切换显示区域"}).clicked(){self.comparison_mode = !self.comparison_mode;}
            }
        });
    }
}

// ============================================================
// 曲线浮动面板 — 居中 · 半透明 · Catmull-Rom · 渐变着色 · X/Y 拖拽
// ============================================================
impl FilmRustPro {
    fn render_curves_overlay(&mut self, ui: &mut egui::Ui) {
        let bg = if self.dark_mode { Color32::from_rgba_unmultiplied(18,22,30,15) } else { Color32::from_rgba_unmultiplied(240,240,248,15) };
        let sr = ui.ctx().content_rect();
        let ww = 560.0_f32; let wh = 480.0_f32;
        let pos = [(sr.center().x - ww/2.0).max(0.0), (sr.center().y - wh/2.0).max(0.0)];
        let win = Window::new("曲线调整")
            .collapsible(false).resizable(true)
            .default_size([ww, wh]).min_size([380.0, 340.0])
            .default_pos(pos)
            .frame(Frame{fill:bg,corner_radius:CornerRadius::same(12u8),inner_margin:egui::Margin::same(14),..Default::default()});
        let ci = self.selected_layer.filter(|&i|matches!(self.layers.layers.get(i).map(|l|&l.layer_type),Some(LayerType::Curves{..})));
        let mut close = false;
        win.show(ui.ctx(), |ui| {
            ui.horizontal(|ui|{
                ui.heading(egui::RichText::new("曲线调整").size(16.0).color(self.text_accent()));
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{
                    if ui.button("关闭").clicked(){close=true;}
                });
            });
            ui.label(egui::RichText::new("上下拖拽=亮度 左右拖拽=范围 | ↑=亮 ↓=暗").size(11.0).color(self.text_dim()));
            if let Some(idx) = ci {
                let gcol = self.text_dim(); let ac = self.text_accent();
                if let LayerType::Curves{contrast,highlights,shadows}=&mut self.layers.layers[idx].layer_type {
                    let cx = self.curve_cx;
                    let y0 = (0.25 - *shadows * 0.25).clamp(0.0, 1.0);
                    let y1 = (0.50 - *contrast * 0.25).clamp(0.0, 1.0);
                    let y2 = (0.75 + *highlights * 0.25).clamp(0.0, 1.0);
                    let cs = ui.available_size().x.min(480.0);
                    let (rect, resp) = ui.allocate_exact_size(vec2(cs, cs), egui::Sense::click_and_drag());
                    let p = ui.painter(); let tl = rect.min; let w = rect.width(); let h = rect.height();
                    p.rect_filled(rect, CornerRadius::same(6u8), Color32::from_rgba_unmultiplied(0,0,0,15));
                    p.rect_stroke(rect, CornerRadius::same(6u8), (0.8, gcol), egui::StrokeKind::Inside);
                    for i in 0..=4 {
                        let x = tl.x + w * i as f32 / 4.0;
                        let y = tl.y + h * i as f32 / 4.0;
                        p.line_segment([pos2(x,tl.y), pos2(x,tl.y+h)], (0.3, gcol));
                        p.line_segment([pos2(tl.x,y), pos2(tl.x+w,y)], (0.3, gcol));
                    }
                    p.line_segment([pos2(tl.x,tl.y+h), pos2(tl.x+w,tl.y)], (0.6, Color32::GRAY));

                    let cp_px = |t:f32,y:f32| pos2(tl.x + t*w, tl.y + (1.0-y)*h);
                    let cps = [cp_px(cx[0], y0), cp_px(cx[1], y1), cp_px(cx[2], y2)];
                    let all = [(0.0_f32, 0.0_f32), (cx[0], y0), (cx[1], y1), (cx[2], y2), (1.0, 1.0)];
                    let curve: Vec<(f32, f32)> = (0..=200).map(|i| {
                        let x = i as f32 / 200.0;
                        (x, catmull_rom_curve_bis(x, &all).clamp(0.0, 1.0))
                    }).collect();

                    let brt = Color32::from_rgb(255, 220, 60);
                    let drk = Color32::from_rgb(50, 90, 220);
                    for wnd in curve.windows(2) {
                        let (x1, y1v) = wnd[0]; let (x2, y2v) = wnd[1];
                        let mid_y = (y1v + y2v) * 0.5;
                        let mid_x = (x1 + x2) * 0.5;
                        let bias = (mid_y - mid_x).clamp(-0.3, 0.3);
                        let seg_col = if bias >= 0.0 {
                            let t = (bias / 0.3).min(1.0);
                            Color32::from_rgb(
                                ((ac.r() as f32) + ((brt.r() as f32) - (ac.r() as f32)) * t) as u8,
                                ((ac.g() as f32) + ((brt.g() as f32) - (ac.g() as f32)) * t) as u8,
                                ((ac.b() as f32) + ((brt.b() as f32) - (ac.b() as f32)) * t) as u8,
                            )
                        } else {
                            let t = (-bias / 0.3).min(1.0);
                            Color32::from_rgb(
                                ((ac.r() as f32) + ((drk.r() as f32) - (ac.r() as f32)) * t) as u8,
                                ((ac.g() as f32) + ((drk.g() as f32) - (ac.g() as f32)) * t) as u8,
                                ((ac.b() as f32) + ((drk.b() as f32) - (ac.b() as f32)) * t) as u8,
                            )
                        };
                        p.line_segment([cp_px(x1, y1v), cp_px(x2, y2v)], (2.8, seg_col));
                    }

                    let mut di = self.curve_drag;
                    if resp.drag_started() {
                        if let Some(mp) = resp.interact_pointer_pos() {
                            let mut best = None; let mut bd = 30.0_f32;
                            for (j, cp) in cps.iter().enumerate() {
                                let d = mp.distance(*cp);
                                if d < bd { bd = d; best = Some(j); }
                            }
                            di = best; self.curve_drag = di;
                        }
                    }
                    if resp.drag_stopped() { di = None; self.curve_drag = None; }
                    if let Some(dj) = di {
                        if let Some(mp) = resp.interact_pointer_pos() {
                            let nx = ((mp.x - tl.x) / w).clamp(0.01, 0.99);
                            let ny = (1.0 - (mp.y - tl.y) / h).clamp(0.0, 1.0);
                            let xr = match dj { 0=>(0.02,0.40), 1=>(0.22,0.78), 2=>(0.60,0.98), _=>(0.0,1.0) };
                            self.curve_cx[dj] = nx.clamp(xr.0, xr.1);
                            match dj {
                                0 => *shadows = ((0.25 - ny) / 0.25).clamp(-1.0, 1.0),
                                1 => *contrast = ((0.50 - ny) / 0.25).clamp(-1.0, 1.0),
                                2 => *highlights = ((ny - 0.75) / 0.25).clamp(-1.0, 1.0),
                                _ => {}
                            }
                        }
                    }

                    for (j, cp) in cps.iter().enumerate() {
                        let diag_y = tl.y + (1.0 - cx[j]) * h;
                        p.line_segment([*cp, pos2(cp.x, diag_y)], (0.8, Color32::from_rgba_unmultiplied(255,255,255,30)));
                    }

                    for (j, cp) in cps.iter().enumerate() {
                        let is_d = di == Some(j);
                        let r = if is_d { 9.0 } else { 7.0 };
                        let cy = [y0, y1, y2][j];
                        let cpbias = (cy - cx[j]).clamp(-0.25, 0.25);
                        let pt_col = if is_d { Color32::WHITE } else {
                            let t = (cpbias / 0.25).clamp(-1.0, 1.0);
                            if t >= 0.0 {
                                Color32::from_rgb(
                                    ((ac.r() as f32) + ((brt.r() as f32) - (ac.r() as f32)) * t) as u8,
                                    ((ac.g() as f32) + ((brt.g() as f32) - (ac.g() as f32)) * t) as u8,
                                    ((ac.b() as f32) + ((brt.b() as f32) - (ac.b() as f32)) * t) as u8,
                                )
                            } else {
                                let t = t.abs();
                                Color32::from_rgb(
                                    ((ac.r() as f32) + ((drk.r() as f32) - (ac.r() as f32)) * t) as u8,
                                    ((ac.g() as f32) + ((drk.g() as f32) - (ac.g() as f32)) * t) as u8,
                                    ((ac.b() as f32) + ((drk.b() as f32) - (ac.b() as f32)) * t) as u8,
                                )
                            }
                        };
                        p.circle_filled(*cp, r, pt_col);
                        if is_d { p.circle_stroke(*cp, r+1.0, (1.5, ac)); }
                        p.text(pos2(cp.x+8.0, cp.y-8.0), egui::Align2::LEFT_TOP, ["阴影","中间调","高光"][j], egui::FontId::proportional(11.0), pt_col);
                    }

                    ui.add_space(8.0);
                    ui.add(egui::Slider::new(contrast, -1.0..=1.0).text("中间调"));
                    ui.add(egui::Slider::new(highlights, -1.0..=1.0).text("高光"));
                    ui.add(egui::Slider::new(shadows, -1.0..=1.0).text("阴影"));
                }
            } else { ui.label("请在右侧图层面板选中「曲线」层"); }
        });
        if close { self.show_curves_overlay = false; }
    }
}

fn catmull_rom_bis(p0: f32, p1: f32, p2: f32, p3: f32, t: f32) -> f32 {
    let t2 = t * t; let t3 = t2 * t;
    0.5 * (2.0 * p1 + (p2 - p0) * t + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * t2 + (3.0 * p1 - p0 - 3.0 * p2 + p3) * t3)
}

fn catmull_rom_curve_bis(x: f32, pts: &[(f32, f32); 5]) -> f32 {
    for i in 0..4 {
        if x >= pts[i].0 && x <= pts[i+1].0 {
            let seg = pts[i+1].0 - pts[i].0;
            let t = if seg > 0.0 { (x - pts[i].0) / seg } else { 0.0 };
            let p0 = if i == 0 { pts[0].1 } else { pts[i-1].1 };
            let p3 = if i >= 3 { pts[4].1 } else { pts[i+2].1 };
            return catmull_rom_bis(p0, pts[i].1, pts[i+1].1, p3, t);
        }
    }
    x
}

// ============================================================
impl FilmRustPro {
    fn render_preview(&mut self, ui: &mut egui::Ui) {
        if self.original_tex.is_none() {
            ui.vertical_centered(|ui|{
                ui.add_space(40.0);
                ui.label(egui::RichText::new("[FR] FilmRust Studio").size(32.0).color(self.text_accent()));
                ui.label(egui::RichText::new("物理级胶片模拟工具 — 把数字照片变成经典胶片质感").size(16.0).color(self.text_dim()));
                ui.add_space(6.0);
                ui.horizontal_centered(|ui|{
                    ui.label(egui::RichText::new("三步出片：拖入照片 → 选胶片风格 → 调参数导出").size(14.0).color(self.text_dim()));
                });
                ui.add_space(20.0);
                Frame::NONE.fill(self.bg_panel()).corner_radius(CornerRadius::same(10u8)).inner_margin(egui::Margin::symmetric(32,20)).show(ui,|ui|{
                    ui.vertical_centered(|ui|{
                        ui.label(egui::RichText::new("拖拽图片到窗口任意位置开始").size(15.0).color(self.text_dim())).on_hover_text("支持 JPG / PNG / TIFF / BMP / WebP，也支持批量拖入");
                        ui.add_space(10.0);
                        if ui.button(egui::RichText::new("打开图片文件").size(16.0)).on_hover_text("选择一张或多张图片开始").clicked(){
                            if let Some(ps)=FileDialog::new().add_filter("图片",&["png","jpg","jpeg","tiff","tif","bmp","webp"]).pick_files(){self.files=ps;if !self.files.is_empty(){self.selected_idx=0;self.load_image_for_display(ui.ctx());}}
                        }
                    });
                });
                ui.add_space(12.0);
                ui.label(egui::RichText::new("涵盖 Kodak · Fujifilm · Ilford · CineStill · Polaroid · Agfa 等 60 种经典胶片风格").size(13.0).color(self.text_dim()));
                ui.add_space(4.0);
                ui.label(egui::RichText::new("每条风格基于真实胶片的感光特性曲线，物理级色彩映射").size(12.0).color(self.text_dim()));
                ui.add_space(40.0);
                ui.label(egui::RichText::new("星TAP软件 2026 | csb603@qq.com").size(11.0).color(self.text_dim()));
            });
            return;
        }
        if self.comparison_mode && self.has_processed && self.original_tex.is_some() && self.processed_tex.is_some() {
            self.render_comparison_view(ui);
            return;
        }
        let tex = if self.has_processed { self.processed_tex.as_ref() } else { self.original_tex.as_ref() };
        if let Some(tex)=tex {
            let avail=ui.available_size(); let iw=self.display_img_w as f32; let ih=self.display_img_h as f32;
            let s=(avail.x/iw).min(avail.y/ih).min(1.0); let sz=vec2(iw*s,ih*s);
            ui.vertical_centered(|ui|{
                ui.add_space(8.0);
                Frame::NONE.corner_radius(CornerRadius::same(12u8)).fill(self.bg_center()).show(ui,|ui|{ui.image((tex.id(),sz));});
                if self.is_processing { ui.add_space(8.0); ui.spinner(); ui.label(egui::RichText::new("显影中...").size(14.0).color(self.text_accent())); }
                else if self.animating { let el=self.anim_start.elapsed().as_secs_f32(); let pct=((el/self.anim_duration).min(1.0)*100.0)as u32; ui.add_space(8.0); ui.add(egui::ProgressBar::new(el/self.anim_duration).desired_width(sz.x.min(400.0)).text(format!("显影中... {}%",pct))); }
            });
        }
    }

    fn render_comparison_view(&mut self, ui: &mut egui::Ui) {
        let tex_orig = self.original_tex.as_ref().unwrap();
        let tex_proc = self.processed_tex.as_ref().unwrap();
        let avail = ui.available_size();
        let iw = self.display_img_w as f32;
        let ih = self.display_img_h as f32;
        let s = (avail.x / iw).min(avail.y / ih).min(1.0);
        let image_size = vec2(iw * s, ih * s);
        let bg = self.bg_center();

        ui.vertical_centered(|ui| {
            ui.add_space(8.0);
            Frame::NONE.corner_radius(CornerRadius::same(12u8)).fill(bg).show(ui, |ui| {
                let (rect, response) = ui.allocate_exact_size(image_size, egui::Sense::click_and_drag());
                let split_x = rect.min.x + rect.width() * self.split_pos;

                let left_rect = egui::Rect::from_min_max(rect.min, egui::pos2(split_x, rect.max.y));
                let p = ui.painter();
                p.with_clip_rect(left_rect).image(tex_orig.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                let right_rect = egui::Rect::from_min_max(egui::pos2(split_x, rect.min.y), rect.max);
                p.with_clip_rect(right_rect).image(tex_proc.id(), rect, egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)), egui::Color32::WHITE);

                let label_y = rect.min.y + 16.0;
                let label_font = egui::FontId::proportional(13.0);
                p.text(egui::pos2(left_rect.min.x + 12.0, label_y), egui::Align2::LEFT_TOP, "原图", label_font.clone(), egui::Color32::WHITE.gamma_multiply(0.9));
                p.text(egui::pos2(right_rect.max.x - 12.0, label_y), egui::Align2::RIGHT_TOP, "处理后", label_font, egui::Color32::WHITE.gamma_multiply(0.9));

                let line_full = [egui::pos2(split_x, rect.min.y), egui::pos2(split_x, rect.max.y)];
                let line_color = egui::Color32::from_rgb(255, 255, 255);
                p.line_segment(line_full, (2.0, line_color));

                let handle_pos = egui::pos2(split_x, rect.center().y);
                p.circle_filled(handle_pos, 10.0, line_color);
                p.circle_stroke(handle_pos, 10.0, (2.0, egui::Color32::from_rgb(40, 40, 40)));

                let tri_color = egui::Color32::from_rgb(60, 60, 60);
                let tri_h = 5.0; let tri_w = 4.0;
                let ltri = [egui::pos2(handle_pos.x - 4.0, handle_pos.y), egui::pos2(handle_pos.x - 4.0 - tri_w, handle_pos.y - tri_h), egui::pos2(handle_pos.x - 4.0 - tri_w, handle_pos.y + tri_h)];
                p.add(egui::Shape::convex_polygon(ltri.to_vec(), tri_color, egui::Stroke::NONE));
                let rtri = [egui::pos2(handle_pos.x + 4.0, handle_pos.y), egui::pos2(handle_pos.x + 4.0 + tri_w, handle_pos.y - tri_h), egui::pos2(handle_pos.x + 4.0 + tri_w, handle_pos.y + tri_h)];
                p.add(egui::Shape::convex_polygon(rtri.to_vec(), tri_color, egui::Stroke::NONE));

                if response.dragged() {
                    if let Some(mp) = response.interact_pointer_pos() {
                        self.split_pos = ((mp.x - rect.min.x) / rect.width()).clamp(0.03, 0.97);
                        ui.ctx().request_repaint();
                    }
                }
            });
        });
    }
}

fn main()->Result<(),eframe::Error>{
    let icon=load_app_icon();
    eframe::run_native("FilmRust Studio Pro",
        eframe::NativeOptions{viewport:egui::ViewportBuilder::default().with_inner_size([1200.0,800.0]).with_icon(icon),..Default::default()},
        Box::new(|cc|{setup_chinese_fonts(&cc.egui_ctx);Ok(Box::new(FilmRustPro::new(cc)))}))
}