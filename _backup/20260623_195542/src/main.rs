use anyhow::{self, Context};
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Parser, Debug)]
#[command(name = "filmrust", version)]
struct Cli {
    #[arg(long = "input")]
    input: Option<PathBuf>,

    #[arg(long = "output")]
    output: Option<PathBuf>,

    #[arg(long = "style")]
    style: Option<String>,

    #[arg(long = "strength", default_value_t = 100)]
    strength: i32,

    #[arg(long = "grain", default_value_t = 100)]
    grain: i32,

    #[arg(long = "warmth", default_value_t = 0.0)]
    warmth: f32,

    #[arg(long = "tint", default_value_t = 0.0)]
    tint: f32,

    #[arg(long = "saturation", default_value_t = 1.0)]
    saturation: f32,

    #[arg(long = "auto")]
    auto: bool,

    #[arg(long = "analyze")]
    analyze: Option<PathBuf>,

    #[arg(long = "json-output")]
    json_output: Option<PathBuf>,

    #[arg(long = "list-styles")]
    list_styles: bool,
}

#[derive(Serialize, Deserialize)]
struct AnalyzeResult {
    ok: bool,
    recommended: String,
    recommended_name: String,
    reason: String,
    analysis: AnalysisData,
    top3: Vec<StyleScore>,
}

#[derive(Serialize, Deserialize)]
struct AnalysisData {
    brightness: f64,
    color_temp: f64,
    saturation_est: f64,
    dark_ratio: f64,
    skin_tone_ratio: f64,
    tags: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct StyleScore {
    key: String,
    name: String,
    desc: String,
    score: f64,
}

// ============================================================
//  胶片预设: 60 种，按常用度分组排序
//  分组: 常用 | 人像 | 风光 | 黑白 | 宝丽来 | 特殊效果
// ============================================================
#[derive(Debug, Clone, Copy)]
struct StyleInfo {
    key: &'static str,
    name: &'static str,
    desc: &'static str,
    group: &'static str,
    is_bw: bool,
    is_warm: i8,
    saturation: u8,
    is_portrait: bool,
    is_landscape: bool,
    is_night_ok: bool,
    is_instant: bool,
}

static FILM_STYLES: &[StyleInfo] = &[
    // ========== ★ 常用 (Common) ==========
    StyleInfo { key:"kodak_ultramax_400", name:"Kodak Ultramax 400", desc:"暖调高饱和·消费级卷王", group:"⭐ 常用",
        is_bw:false, is_warm:2, saturation:8, is_portrait:false, is_landscape:true, is_night_ok:true, is_instant:false },
    StyleInfo { key:"fuji_pro_400h", name:"Fujicolor Pro 400H", desc:"日系人像王·冷蓝阴影·暖粉高光", group:"⭐ 常用",
        is_bw:false, is_warm:-1, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"fuji_natura_1600", name:"Fujicolor Natura 1600", desc:"月光卷·暖调·青绿阴影", group:"⭐ 常用",
        is_bw:false, is_warm:2, saturation:7, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"kodak_portra_400", name:"Kodak Portra 400", desc:"人像首选，柔和肤色", group:"⭐ 常用",
        is_bw:false, is_warm:2, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"kodak_gold_200", name:"Kodak Gold 200", desc:"日常阳光感，90年代温暖回忆", group:"⭐ 常用",
        is_bw:false, is_warm:3, saturation:7, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"kodak_ektar_100", name:"Kodak Ektar 100", desc:"超细腻颜色，风光利器", group:"⭐ 常用",
        is_bw:false, is_warm:1, saturation:8, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"kodak_tri_x_400", name:"Kodak Tri-X 400", desc:"传奇黑白，高对比颗粒质感", group:"⭐ 常用",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"fujifilm_superia_400", name:"Fujifilm Superia 400", desc:"通用彩色，自然饱和日用", group:"⭐ 常用",
        is_bw:false, is_warm:1, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"fujifilm_provia_100f", name:"Fujifilm Provia 100F", desc:"专业反转片，中性真实色彩", group:"⭐ 常用",
        is_bw:false, is_warm:0, saturation:7, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"fujifilm_velvia_50", name:"Fujifilm Velvia 50", desc:"风光首席，超高饱和鲜艳", group:"⭐ 常用",
        is_bw:false, is_warm:-2, saturation:10, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"cinestill_800t", name:"CineStill 800T", desc:"夜景/钨丝灯，电影感青橙调", group:"⭐ 常用",
        is_bw:false, is_warm:-1, saturation:7, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"cinestill_50d", name:"CineStill 50D", desc:"日光型电影胶片，电影感街拍", group:"⭐ 常用",
        is_bw:false, is_warm:0, saturation:7, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ilford_hp5_plus_400", name:"Ilford HP5 Plus 400", desc:"黑白经典，高对比颗粒感", group:"⭐ 常用",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"standard_daylight", name:"Standard Daylight", desc:"中性基准，去胶片化原色", group:"⭐ 常用",
        is_bw:false, is_warm:0, saturation:5, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"lomography_color_chrome", name:"Lomography Color Chrome", desc:"LOMO艺术，高对比偏色", group:"⭐ 常用",
        is_bw:false, is_warm:1, saturation:9, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"polaroid_600_color", name:"Polaroid 600 Color", desc:"宝丽来即时感，怀旧偏暖", group:"⭐ 常用",
        is_bw:false, is_warm:2, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:true },

    // ========== 人像 (Portrait) ==========
    StyleInfo { key:"kodak_portra_160", name:"Kodak Portra 160", desc:"低感人像，更细腻的肤色", group:"人像",
        is_bw:false, is_warm:2, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"kodak_portra_800", name:"Kodak Portra 800", desc:"弱光人像，温暖颗粒感", group:"人像",
        is_bw:false, is_warm:2, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"kodak_portra_400_artistic", name:"Kodak Portra 400 Artistic", desc:"艺术版，增强色彩分离", group:"人像",
        is_bw:false, is_warm:2, saturation:7, is_portrait:true, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"fujifilm_superia_200", name:"Fujifilm Superia 200", desc:"暖调人像，日系清新", group:"人像",
        is_bw:false, is_warm:1, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"fujifilm_superia_100", name:"Fujifilm Superia 100", desc:"低感人像，细腻柔和", group:"人像",
        is_bw:false, is_warm:1, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"agfa_vista_400", name:"Agfa Vista 400", desc:"德系暖调，浓郁色彩人像", group:"人像",
        is_bw:false, is_warm:2, saturation:7, is_portrait:true, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"agfa_vista_200", name:"Agfa Vista 200", desc:"德系暖调，日常人像", group:"人像",
        is_bw:false, is_warm:2, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"agfa_vista_100", name:"Agfa Vista 100", desc:"德系低感人像", group:"人像",
        is_bw:false, is_warm:1, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"lucky_color_200", name:"Lucky Color 200", desc:"国产乐凯，暖调怀旧", group:"人像",
        is_bw:false, is_warm:2, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },

    // ========== 风光 (Landscape) ==========
    StyleInfo { key:"kodak_ektachrome_100", name:"Kodak Ektachrome 100", desc:"经典反转片，暖调风光", group:"风光",
        is_bw:false, is_warm:1, saturation:8, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"kodak_ektachrome_100vs", name:"Kodak Ektachrome 100 VS", desc:"超鲜艳反转片，极致色彩", group:"风光",
        is_bw:false, is_warm:0, saturation:10, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"kodak_kodachrome_64", name:"Kodak Kodachrome 64", desc:"经典柯达克罗姆，暖调浓郁", group:"风光",
        is_bw:false, is_warm:1, saturation:8, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"kodak_kodachrome_25", name:"Kodak Kodachrome 25", desc:"极致细腻柯达克罗姆", group:"风光",
        is_bw:false, is_warm:1, saturation:7, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"fujifilm_velvia_50_artistic", name:"Fujifilm Velvia 50 Artistic", desc:"增强版Velvia，极致鲜艳", group:"风光",
        is_bw:false, is_warm:-2, saturation:10, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"fujifilm_astia_100f", name:"Fujifilm Astia 100F", desc:"柔和反转片，淡彩风光", group:"风光",
        is_bw:false, is_warm:0, saturation:5, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"agfa_optima_200", name:"Agfa Optima 200", desc:"暖调风光，德系反转片", group:"风光",
        is_bw:false, is_warm:1, saturation:7, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"agfa_precisa_100", name:"Agfa Precisa 100", desc:"暖调反转片，风光人像通用", group:"风光",
        is_bw:false, is_warm:1, saturation:6, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },

    // ========== 黑白 (Black & White) ==========
    StyleInfo { key:"kodak_tri_x_400_artistic", name:"Kodak Tri-X 400 Artistic", desc:"增强版，更强颗粒对比", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"kodak_plus_x_125", name:"Kodak Plus-X 125", desc:"细腻黑白，中调丰富", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ilford_hp5_plus_400_artistic", name:"Ilford HP5 Plus 400 Artistic", desc:"增强版HP5，更强颗粒", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"ilford_fp4_plus_125", name:"Ilford FP4 Plus 125", desc:"中速黑白，细腻过渡", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ilford_delta_400", name:"Ilford Delta 400 Professional", desc:"现代黑白，颗粒锐利", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"ilford_delta_100", name:"Ilford Delta 100 Professional", desc:"超细腻现代黑白", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ilford_pan_f_plus_50", name:"Ilford Pan F Plus 50", desc:"极细腻低感黑白，风光专用", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ilford_xp2_super_400", name:"Ilford XP2 Super 400", desc:"C41工艺黑白，冲印方便", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"ilford_sfx_200", name:"Ilford SFX 200", desc:"红外效果黑白，独特质感", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ilford_ortho_plus_80", name:"Ilford Ortho Plus 80", desc:"正色片，高对比反差", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"fujifilm_neopan_400", name:"Fujifilm Neopan 400", desc:"日系黑白，细腻灰阶", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"fujifilm_neopan_100", name:"Fujifilm Neopan 100", desc:"日系低感黑白", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"agfa_apx_400", name:"Agfa APX 400", desc:"经典德系黑白", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"agfa_apx_100", name:"Agfa APX 100", desc:"经典德系细腻黑白", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"polaroid_bw_667", name:"Polaroid B&W 667", desc:"宝丽来黑白，即时显影质感", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"polaroid_55_bw", name:"Polaroid 55 B&W", desc:"宝丽来正负片，极致黑白", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"orwo_un54", name:"Orwo UN54", desc:"东德经典黑白，高对比", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"orwo_un64", name:"Orwo UN64", desc:"东德低感黑白，细腻", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ricoh_gr_street", name:"Ricoh GR Street Night", desc:"街拍高感黑白，粗颗粒", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"agfa_scala_200", name:"Agfa Scala 200", desc:"黑白反转片，高反差", group:"黑白",
        is_bw:true, is_warm:0, saturation:0, is_portrait:false, is_landscape:false, is_night_ok:true, is_instant:false },

    // ========== 宝丽来 (Instant) ==========
    StyleInfo { key:"polaroid_sx70_color", name:"Polaroid SX-70 Color", desc:"经典SX-70，暖调柔和", group:"宝丽来",
        is_bw:false, is_warm:2, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:true },
    StyleInfo { key:"polaroid_i_type_color", name:"Polaroid i-Type Color", desc:"现代宝丽来，鲜艳色彩", group:"宝丽来",
        is_bw:false, is_warm:1, saturation:7, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:true },
    StyleInfo { key:"polaroid_spectra_color", name:"Polaroid Spectra Color", desc:"宽幅宝丽来，偏冷调", group:"宝丽来",
        is_bw:false, is_warm:0, saturation:6, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:true },
    StyleInfo { key:"polaroid_100_color", name:"Polaroid 100 Color", desc:"老式宝丽来100，褪色怀旧", group:"宝丽来",
        is_bw:false, is_warm:1, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:true },

    // ========== 特殊效果 (Special) ==========
    StyleInfo { key:"lomography_lomochrome_purple", name:"Lomography Lomochrome Purple", desc:"紫色幻彩，独特艺术效果", group:"特殊效果",
        is_bw:false, is_warm:0, saturation:9, is_portrait:false, is_landscape:true, is_night_ok:false, is_instant:false },
    StyleInfo { key:"ferrania_solaris_400", name:"Ferrania Solaris 400", desc:"意式暖调，复古褪色感", group:"特殊效果",
        is_bw:false, is_warm:3, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:true, is_instant:false },
    StyleInfo { key:"ferrania_solaris_100", name:"Ferrania Solaris 100", desc:"意式低感，暖调柔和", group:"特殊效果",
        is_bw:false, is_warm:2, saturation:5, is_portrait:true, is_landscape:false, is_night_ok:false, is_instant:false },
];

// ============================================================
//  查找 filmr stock：将我们的 key 正确匹配到 filmr 的 stock.name
// ============================================================
fn find_filmr_stock_by_key<'a>(presets: &'a [std::rc::Rc<filmr::FilmStock>], key: &str) -> Option<&'a filmr::FilmStock> {
    let search = key.replace("_", " ").to_lowercase();
    let manufacturers = ["kodak", "fujifilm", "fuji", "ilford", "polaroid", "agfa",
                         "lomo", "lomography", "cinestill", "ferrania", "orwo",
                         "ricoh", "lucky", "standard", "vintage"];

    let clean_words: Vec<&str> = search.split(' ')
        .filter(|w| w.len() > 1 && !manufacturers.contains(w))
        .collect();
    let clean_nospace: String = clean_words.join("")
        .chars().filter(|c| !c.is_whitespace()).collect();

    presets.iter().find(|s| {
        let sn = s.name.to_lowercase();
        let sn_nospace: String = sn.chars().filter(|c| !c.is_whitespace()).collect();

        sn == search || sn.contains(&search) || search.contains(&sn)
        || sn_nospace.contains(&clean_nospace) || clean_nospace.contains(&sn_nospace)
        || clean_words.iter().all(|w| sn.contains(w))
    }).map(|rc| rc.as_ref())
}

fn get_style_by_key(key: &str) -> Option<&'static StyleInfo> {
    FILM_STYLES.iter().find(|s| s.key == key)
}

// ============================================================
//  图片加载：普通格式 + RAW (dcraw)
// ============================================================
fn is_raw_format(path: &Path) -> bool {
    let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    matches!(ext.as_str(), "cr2" | "cr3" | "nef" | "arw" | "dng" | "raf" | "rw2" | "orf" | "sr2" | "srw" | "x3f" | "3fr" | "mef" | "mos" | "pef")
}

fn find_dcraw() -> Option<PathBuf> {
    let candidates = [
        PathBuf::from("dcraw.exe"),
        PathBuf::from("dcraw"),
    ];
    for c in &candidates {
        if c.exists() { return Some(c.clone()); }
    }
    if let Ok(output) = Command::new("where").arg("dcraw").output() {
        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let path = stdout.lines().next().map(|l| l.trim().to_string()).unwrap_or_default();
            if !path.is_empty() && Path::new(&path).exists() {
                return Some(PathBuf::from(path));
            }
        }
    }
    if let Ok(paths) = std::env::var("PATH") {
        for dir in paths.split(';') {
            for name in &["dcraw.exe", "dcraw"] {
                let full = Path::new(dir).join(name);
                if full.exists() { return Some(full); }
            }
        }
    }
    None
}

fn convert_raw_with_dcraw(raw_path: &Path) -> anyhow::Result<image::DynamicImage> {
    let dcraw = find_dcraw()
        .ok_or_else(|| anyhow::anyhow!(
            "RAW 文件需要 dcraw 工具转换。\n下载地址: https://www.dechifro.org/dcraw/\n\
             将 dcraw.exe 放入程序目录或添加到 PATH 后重试。"))?;

    if !raw_path.exists() {
        return Err(anyhow::anyhow!("RAW 文件不存在: {:?}", raw_path));
    }

    let out_dir = raw_path.parent().unwrap_or(Path::new("."));
    let stem = raw_path.file_stem().and_then(|s| s.to_str()).unwrap_or("raw");
    let tiff_path = out_dir.join(format!("{}_filmrust.tiff", stem));

    let _ = fs::remove_file(&tiff_path);

    let output = Command::new(&dcraw)
        .arg("-T")
        .arg("-w")
        .arg("-q")
        .arg("3")
        .arg(raw_path)
        .output()
        .with_context(|| format!("执行 dcraw 失败: {:?}", dcraw))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("dcraw 转换失败: {}", stderr.trim()));
    }

    let dcraw_out = out_dir.join(format!("{}.tiff", stem));
    let final_path = if dcraw_out.exists() { dcraw_out } else { tiff_path.clone() };

    for _ in 0..30 {
        if final_path.exists() { break; }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let img = image::open(&final_path)
        .with_context(|| "读取 dcraw 输出的 TIFF 失败")?;

    let _ = fs::remove_file(&final_path);

    Ok(img)
}

fn load_image(path: &Path) -> anyhow::Result<image::DynamicImage> {
    if is_raw_format(path) {
        convert_raw_with_dcraw(path)
    } else {
        image::open(path).with_context(|| format!("无法读取图片: {:?}", path))
    }
}

// ============================================================
//  图片分析：针对全部 60 种风格智能评分推荐
// ============================================================
fn analyze_image(image_path: &Path) -> AnalyzeResult {
    let img = match load_image(image_path) {
        Ok(img) => img.to_rgb8(),
        Err(_) => {
            return AnalyzeResult {
                ok: false,
                recommended: "kodak_portra_400".to_string(),
                recommended_name: "Kodak Portra 400".to_string(),
                reason: "读取失败".to_string(),
                analysis: AnalysisData {
                    brightness: 0.0, color_temp: 0.0, saturation_est: 0.0,
                    dark_ratio: 0.0, skin_tone_ratio: 0.0, tags: Vec::new(),
                }, top3: Vec::new(),
            };
        }
    };

    let (w, h) = img.dimensions();
    let total = (w * h) as f64;

    let mut r_sum: u64 = 0;
    let mut g_sum: u64 = 0;
    let mut b_sum: u64 = 0;
    let mut dark_count: u64 = 0;
    let mut skin_count: u64 = 0;

    let mut r_var: f64 = 0.0;
    let mut g_var: f64 = 0.0;
    let mut b_var: f64 = 0.0;

    for pixel in img.pixels() {
        let r = pixel[0] as f64;
        let g = pixel[1] as f64;
        let b = pixel[2] as f64;

        r_sum += r as u64;
        g_sum += g as u64;
        b_sum += b as u64;

        let lum = 0.299 * r + 0.587 * g + 0.114 * b;
        if lum < 50.0 { dark_count += 1; }

        if r > 80.0 && r < 230.0 && g > 30.0 && g < 200.0 && b > 20.0 && b < 180.0
            && r > g && r > b && (r - g).abs() < 60.0
        {
            skin_count += 1;
        }

        r_var += r * r;
        g_var += g * g;
        b_var += b * b;
    }

    let r_mean = r_sum as f64 / total;
    let g_mean = g_sum as f64 / total;
    let b_mean = b_sum as f64 / total;

    let brightness = 0.299 * r_mean + 0.587 * g_mean + 0.114 * b_mean;
    let color_temp = r_mean - b_mean;
    let dark_ratio = dark_count as f64 / total;

    r_var = (r_var / total - r_mean * r_mean).sqrt();
    g_var = (g_var / total - g_mean * g_mean).sqrt();
    b_var = (b_var / total - b_mean * b_mean).sqrt();
    let saturation_est = ((r_var * r_var + g_var * g_var + b_var * b_var) / 3.0).sqrt();

    let skin_tone_ratio = skin_count as f64 / total;

    let is_night = brightness < 65.0 || dark_ratio > 0.4;
    let is_low_light = brightness < 100.0 && !is_night;
    let is_bright = brightness > 160.0;
    let is_warm = color_temp > 8.0;
    let is_cool = color_temp < -5.0;
    let is_flat = saturation_est < 30.0;

    let mut tags = Vec::new();
    if is_night { tags.push("弱光/夜景".to_string()); }
    else if is_low_light { tags.push("一般室内".to_string()); }
    else if is_bright { tags.push("明亮日拍".to_string()); }
    if is_warm { tags.push("原片偏暖".to_string()); }
    else if is_cool { tags.push("原片偏冷".to_string()); }
    if is_flat { tags.push("低对比/灰".to_string()); }
    if skin_tone_ratio > 0.08 { tags.push("含皮肤/人像".to_string()); }

    // 评分：针对每种风格计算匹配度
    let mut scores: HashMap<&str, f64> = HashMap::new();
    for style in FILM_STYLES {
        let mut score = 50.0;

        let sat_diff = (style.saturation as f64 - saturation_est / 12.0).abs();
        score -= sat_diff * 1.5;

        if is_night && style.is_night_ok { score += 25.0; }
        else if is_night { score -= 15.0; }

        if is_low_light {
            if style.is_night_ok { score += 10.0; }
            if style.saturation >= 5 { score += 5.0; }
        }

        if is_bright && (style.is_landscape || style.saturation >= 7) { score += 15.0; }

        if is_warm && style.is_warm < 0 { score += 12.0; }
        else if is_warm && style.is_warm > 0 { score -= 8.0; }
        else if is_cool && style.is_warm > 0 { score += 10.0; }
        else if is_cool && style.is_warm < 0 { score -= 6.0; }

        if skin_tone_ratio > 0.08 && style.is_portrait { score += 20.0; }
        if skin_tone_ratio > 0.15 && style.is_portrait { score += 10.0; }
        if skin_tone_ratio > 0.08 && style.is_bw { score -= 12.0; }
        if skin_tone_ratio < 0.02 && style.is_landscape { score += 12.0; }

        if is_flat && style.saturation <= 5 { score += 8.0; }
        else if is_flat && style.saturation >= 8 { score -= 8.0; }

        if style.is_bw && is_flat { score += 10.0; }
        if style.is_bw && !is_flat && saturation_est > 50.0 { score -= 10.0; }

        // 宝丽来：需要更自然的输入
        if style.is_instant && !is_night { score -= 3.0; }

        // 基础流行度：常用组额外加分
        if style.group == "⭐ 常用" { score += 5.0; }

        scores.insert(style.key, score.max(0.0));
    }

    let mut sorted_scores: Vec<(&str, f64)> = scores.into_iter().collect();
    sorted_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    let best_key = sorted_scores[0].0;
    let best_style = get_style_by_key(best_key).unwrap();

    let analysis = AnalysisData {
        brightness: (brightness * 10.0).round() / 10.0,
        color_temp: (color_temp * 10.0).round() / 10.0,
        saturation_est: (saturation_est * 10.0).round() / 10.0,
        dark_ratio: (dark_ratio * 100.0).round() / 100.0,
        skin_tone_ratio: (skin_tone_ratio * 100.0).round() / 100.0,
        tags,
    };

    let info_parts: Vec<String> = vec![
        format!("亮度{:.0}", brightness),
        format!("温差{:.0}", color_temp),
        format!("饱和{:.0}", saturation_est),
    ];
    let reason = format!("{} · 推荐 {}", info_parts.join(" "), best_style.name);

    let top3: Vec<StyleScore> = sorted_scores[..3.min(sorted_scores.len())]
        .iter()
        .map(|(key, score)| {
            let style = get_style_by_key(key).unwrap();
            StyleScore {
                key: key.to_string(),
                name: style.name.to_string(),
                desc: style.desc.to_string(),
                score: (score * 10.0).round() / 10.0,
            }
        })
        .collect();

    AnalyzeResult {
        ok: true,
        recommended: best_key.to_string(),
        recommended_name: best_style.name.to_string(),
        reason,
        analysis,
        top3,
    }
}

// ============================================================
//  图像处理：应用胶片效果
// ============================================================
#[allow(clippy::too_many_arguments)]
fn process_image(input_path: &Path, output_path: &Path, style_name: &str, _strength: i32, grain: i32, warmth: f32, tint: f32, saturation: f32) -> anyhow::Result<()> {
    let mut rgb = load_image(input_path)?
        .to_rgb8();

    let grain_factor = grain as f32 / 100.0;

    let presets = filmrust::get_all_filmr_stocks();
    let stock = find_filmr_stock_by_key(&presets, style_name)
        .or_else(|| presets.first().map(|rc| rc.as_ref()))
        .ok_or_else(|| anyhow::anyhow!("找不到胶片预设: {}", style_name))?;

    // 查找预设默认扫片校色值
    let (def_warmth, def_tint, def_sat) = filmrust::find_preset(style_name)
        .map(|p| (p.default_warmth, p.default_tint, p.default_saturation))
        .unwrap_or((0.0, 0.0, 1.0));

    // 以预设默认校色为基准，用户传参为相对微调
    let effective_warmth = def_warmth + warmth;
    let effective_tint = def_tint + tint;
    let effective_saturation = (def_sat * saturation).clamp(0.0, 2.0);

    let config = filmr::SimulationConfig {
        exposure_time: 1.0,
        auto_levels: true,
        white_balance_mode: filmr::WhiteBalanceMode::Off,
        enable_grain: grain_factor > 0.05,
        motion_blur_amount: 0.0,
        object_motion_amount: 0.0,
        light_leak: filmr::light_leak::LightLeakConfig {
            enabled: false,
            leaks: Vec::new(),
        },
        saturation: effective_saturation,
        warmth: effective_warmth,
        ..Default::default()
    };

    let processed = filmr::process_image(&rgb, stock, &config);
    rgb = processed;

    if effective_tint.abs() > 0.005 {
        rgb = filmrust::apply_tint_to_rgb(&rgb, effective_tint);
    }

    rgb.save(output_path)
        .with_context(|| format!("保存图片失败: {:?}", output_path))?;

    Ok(())
}

// ============================================================
//  主入口
// ============================================================
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    if cli.list_styles {
        let mut current_group = "";
        for style in FILM_STYLES {
            if style.group != current_group {
                current_group = style.group;
                println!("\n── {} ──", style.group);
            }
            println!("  {:30} {} — {}", style.key, style.name, style.desc);
        }
        return Ok(());
    }

    if let Some(analyze_path) = &cli.analyze {
        if !analyze_path.exists() {
            return Err(anyhow::anyhow!("文件不存在: {:?}", analyze_path));
        }
        let result = analyze_image(analyze_path);
        let json = serde_json::to_string(&result)
            .with_context(|| "序列化分析结果失败")?;

        if let Some(json_output) = &cli.json_output {
            fs::write(json_output, &json)
                .with_context(|| format!("写入 JSON 失败: {:?}", json_output))?;
        }

        println!("{}", json);
        return Ok(());
    }

    let input_path = cli.input
        .ok_or_else(|| anyhow::anyhow!("必须提供 --input"))?;

    if !input_path.exists() {
        return Err(anyhow::anyhow!("文件不存在: {:?}", input_path));
    }

    let output_path = cli.output
        .unwrap_or_else(|| input_path.parent().unwrap_or(Path::new("."))
            .join(format!("{}_film.jpg",
                input_path.file_stem().unwrap_or_default().to_string_lossy())));

    let style_name = match (&cli.auto, &cli.style) {
        (true, _) | (false, None) => {
            let analysis = analyze_image(&input_path);
            if cli.auto {
                println!("[分析] {}", analysis.reason);
            }
            analysis.recommended
        }
        (false, Some(s)) => s.clone(),
    };

    if is_raw_format(&input_path) {
        println!("[信息] RAW 输入: {:?} → dcraw 转换中...", input_path);
    }

    process_image(&input_path, &output_path, &style_name, cli.strength, cli.grain, cli.warmth, cli.tint, cli.saturation)
        .with_context(|| format!("处理图片失败: {:?}", input_path))?;

    println!("[OK] 已生成: {:?} (风格: {})", output_path, style_name);

    Ok(())
}
