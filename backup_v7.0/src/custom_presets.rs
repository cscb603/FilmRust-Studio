/// 自定义胶片预设 - 从 Python 版反推 + 网络研究
///
/// 这些预设使用 filmr 的 RGB 简化通道模型（无 layer_stack），
/// 通过精心调参的 SegmentedCurve + color_matrix 还原胶片特征。
use filmr::film::{FilmStock, FilmType, ReciprocityFailure, SegmentedCurve};
use filmr::spectral::FilmSpectralParams;
use filmr::GrainModel;
use std::rc::Rc;

/// Kodak Ultramax 400 — 暖调高饱和消费级卷
///
/// 来源: Kodak E-7023 技术文档 + Python 反推
/// 特征: 对比鲜明·暖调浓郁·高饱和高颗粒·消费级风格
pub fn kodak_ultramax_400() -> FilmStock {
    FilmStock {
        manufacturer: "Kodak".to_string(),
        name: "Ultramax 400".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 400.0,
        r_curve: SegmentedCurve { d_min: 0.14, d_max: 2.9, gamma: 0.65, shoulder_point: 0.8, exposure_offset: 0.12 },
        g_curve: SegmentedCurve { d_min: 0.16, d_max: 2.9, gamma: 0.70, shoulder_point: 0.8, exposure_offset: 0.12 },
        b_curve: SegmentedCurve { d_min: 0.19, d_max: 2.9, gamma: 0.75, shoulder_point: 0.8, exposure_offset: 0.12 },
        // 高饱和暖调：强色分离，R通道纯净暖调，G/B稍压缩
        color_matrix: [
            [1.15, -0.08, -0.07],
            [-0.08, 1.12, -0.04],
            [-0.04, -0.08, 1.12],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000120, sigma_read: 0.005, monochrome: false,
            blur_radius: 0.5, roughness: 0.45, color_correlation: 0.8,
            shadow_noise: 0.001, highlight_coarseness: 0.08,
        },
        resolution_lp_mm: 120.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.15,
        halation_threshold: 0.85,
        halation_sigma: 0.014,
        halation_tint: [1.0, 0.72, 0.52],
        layer_stack: None,
    }
}

/// Fujicolor Pro 400H — 日系人像专业卷
///
/// 来源: Fuji X Weekly 配方 + 摄影师评测
/// 特征: 粉调高光·冷蓝阴影·低饱和·肤色柔和·高宽容度
pub fn fuji_pro_400h() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Pro 400H".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 400.0,
        r_curve: SegmentedCurve { d_min: 0.18, d_max: 2.6, gamma: 0.60, shoulder_point: 0.82, exposure_offset: 0.085 },
        g_curve: SegmentedCurve { d_min: 0.20, d_max: 2.6, gamma: 0.58, shoulder_point: 0.82, exposure_offset: 0.085 },
        b_curve: SegmentedCurve { d_min: 0.22, d_max: 2.6, gamma: 0.65, shoulder_point: 0.80, exposure_offset: 0.085 },
        // 粉蓝双色调：R→G正耦合=粉肤色，R→B正耦合=暖高光，G通道压低=粉彩效果
        color_matrix: [
            [1.04, -0.02, -0.02],
            [0.04, 0.94, 0.02],
            [0.03, -0.03, 1.00],
        ],
        spectral_params: FilmSpectralParams::new_portra(),
        grain_model: GrainModel {
            alpha: 0.000090, sigma_read: 0.004, monochrome: false,
            blur_radius: 0.45, roughness: 0.40, color_correlation: 0.85,
            shadow_noise: 0.001, highlight_coarseness: 0.04,
        },
        resolution_lp_mm: 125.0,
        vignette_strength: 0.45,
        reciprocity: ReciprocityFailure { beta: 0.05 },
        halation_strength: 0.18,
        halation_threshold: 0.82,
        halation_sigma: 0.015,
        halation_tint: [1.0, 0.78, 0.62],
        layer_stack: None,
    }
}

/// Fujicolor Natura 1600 — 高速月光卷
///
/// 来源: i50mm 评测 + 摄影师样片
/// 特征: 暖调浓郁·青绿阴影·颗粒控制出色·高宽容度
pub fn fuji_natura_1600() -> FilmStock {
    FilmStock {
        manufacturer: "Fujifilm".to_string(),
        name: "Natura 1600".to_string(),
        film_type: FilmType::ColorNegative,
        iso: 1600.0,
        r_curve: SegmentedCurve { d_min: 0.16, d_max: 2.7, gamma: 0.60, shoulder_point: 0.78, exposure_offset: 0.20 },
        g_curve: SegmentedCurve { d_min: 0.18, d_max: 2.7, gamma: 0.65, shoulder_point: 0.78, exposure_offset: 0.20 },
        b_curve: SegmentedCurve { d_min: 0.21, d_max: 2.7, gamma: 0.70, shoulder_point: 0.76, exposure_offset: 0.20 },
        // 暖绿调和：R通道高增益=暖调，G通道稍强=青绿阴影
        color_matrix: [
            [1.10, -0.05, -0.05],
            [-0.03, 1.08, -0.05],
            [-0.03, -0.06, 1.09],
        ],
        spectral_params: FilmSpectralParams::new_color_negative_standard(),
        grain_model: GrainModel {
            alpha: 0.000250, sigma_read: 0.008, monochrome: false,
            blur_radius: 0.6, roughness: 0.55, color_correlation: 0.75,
            shadow_noise: 0.002, highlight_coarseness: 0.10,
        },
        resolution_lp_mm: 95.0,
        vignette_strength: 0.5,
        reciprocity: ReciprocityFailure { beta: 0.08 },
        halation_strength: 0.22,
        halation_threshold: 0.80,
        halation_sigma: 0.016,
        halation_tint: [1.0, 0.70, 0.55],
        layer_stack: None,
    }
}

/// 获取所有自定义胶片
pub fn get_custom_stocks() -> Vec<Rc<FilmStock>> {
    vec![
        Rc::new(kodak_ultramax_400()),
        Rc::new(fuji_pro_400h()),
        Rc::new(fuji_natura_1600()),
    ]
}
