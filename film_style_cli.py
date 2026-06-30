"""
智能胶片调色系统 - 零依赖分发版
用法:
  film_style_cli.exe --input photo.jpg --style kodak_portra_400 --output out.jpg
  film_style_cli.exe --analyze photo.jpg
  film_style_cli.py --list-styles
"""
import argparse
import cv2
import numpy as np
from pathlib import Path
import json
import sys
import os

# ============================================================
# 工具函数
# ============================================================
_SUPPORTED_READ = {".jpg", ".jpeg", ".png", ".bmp", ".tif", ".tiff",
                    ".webp", ".ppm", ".pgm", ".dib"}
_SUPPORTED_WRITE = {".jpg", ".jpeg", ".png", ".bmp", ".tif", ".tiff", ".webp"}


def imread_unicode(path):
    try:
        p = str(path)
        if not os.path.isfile(p):
            return None
        data = np.fromfile(p, dtype=np.uint8)
        if data.size == 0:
            return None
        return cv2.imdecode(data, cv2.IMREAD_COLOR)
    except Exception:
        return None


def imwrite_unicode(path, img, quality=95):
    try:
        ext = Path(path).suffix.lower() or ".jpg"
        if ext not in _SUPPORTED_WRITE:
            ext = ".jpg"

        if ext in (".jpg", ".jpeg"):
            params = [cv2.IMWRITE_JPEG_QUALITY, int(quality)]
        elif ext == ".png":
            params = [cv2.IMWRITE_PNG_COMPRESSION, max(0, 9 - int(quality / 12))]
        elif ext in (".tif", ".tiff"):
            params = [cv2.IMWRITE_TIFF_COMPRESSION, 1]
        elif ext == ".webp":
            params = [cv2.IMWRITE_WEBP_QUALITY, int(quality)]
        else:
            params = []

        ok, buf = cv2.imencode(ext, img, params)
        if not ok:
            ok, buf = cv2.imencode(".jpg", img, [cv2.IMWRITE_JPEG_QUALITY, int(quality)])
        if ok:
            parent = Path(path).parent
            if str(parent) and not parent.exists():
                parent.mkdir(parents=True, exist_ok=True)
            buf.tofile(str(path))
            return True
        return False
    except Exception:
        return False


def check_format_supported(path):
    ext = Path(path).suffix.lower()
    if ext in _SUPPORTED_READ:
        return True, None
    if not ext:
        return False, "文件没有扩展名，无法识别格式"
    return False, f"格式 {ext} 不直接支持，建议先用 PS/画图另存为 JPG/PNG"

# ============================================================
# 核心: 胶片响应曲线生成
# ============================================================
def build_response_curve(
    shoulder=0.0,
    toe=0.0,
    midpoint=0.5,
    gamma=1.0,
    warm_bias=0.0,
    cool_bias=0.0,
    green_tint=0.0
):
    x = np.linspace(0.0, 1.0, 256, dtype=np.float32)
    s_curve = x.copy()
    if shoulder > 0:
        s_curve = 1.0 - np.power(1.0 - s_curve, 1.0 + shoulder)
    if toe > 0:
        s_curve = np.power(s_curve, 1.0 / (1.0 + toe))
    if gamma != 1.0:
        s_curve = np.power(s_curve, gamma)
    s_curve = np.clip(s_curve + midpoint, 0, 1)

    r_curve = s_curve.copy()
    g_curve = s_curve.copy()
    b_curve = s_curve.copy()
    if warm_bias > 0:
        shadow_warm = warm_bias * np.power(1.0 - x, 1.5)
        r_curve = np.clip(r_curve + shadow_warm, 0, 1)
    if cool_bias > 0:
        shadow_cool = cool_bias * np.power(1.0 - x, 1.5)
        b_curve = np.clip(b_curve + shadow_cool, 0, 1)
    if green_tint != 0:
        g_tint = green_tint * np.power(x, 0.5)
        g_curve = np.clip(g_curve + g_tint, 0, 1)

    r_lut = np.clip(r_curve * 255, 0, 255).astype(np.uint8)
    g_lut = np.clip(g_curve * 255, 0, 255).astype(np.uint8)
    b_lut = np.clip(b_curve * 255, 0, 255).astype(np.uint8)
    return r_lut, g_lut, b_lut

# ============================================================
# Zone System 9 分区色调映射
# ============================================================
def apply_zone_system(img_rgb, zone_params, strength=1.0):
    h, w = img_rgb.shape[:2]
    img_f = img_rgb.astype(np.float32)
    luminance = (0.2126 * img_f[:, :, 0] +
                 0.7152 * img_f[:, :, 1] +
                 0.0722 * img_f[:, :, 2])
    zone_centers = np.array([14, 42, 70, 99, 127, 156, 184, 213, 241],
                            dtype=np.float32)
    zone_width = 32.0

    bright_adj = np.zeros((h, w), dtype=np.float32)
    r_adj = np.zeros((h, w), dtype=np.float32)
    g_adj = np.zeros((h, w), dtype=np.float32)
    b_adj = np.zeros((h, w), dtype=np.float32)
    weight_sum = np.zeros((h, w), dtype=np.float32) + 1e-6

    for zi in range(9):
        params = zone_params[zi]
        diff = luminance - zone_centers[zi]
        mask = np.exp(-(diff * diff) / (zone_width * zone_width))
        br = params.get("bright", 0.0)
        rsh = params.get("r", 0.0)
        gsh = params.get("g", 0.0)
        bsh = params.get("b", 0.0)
        bright_adj += mask * br
        r_adj += mask * rsh
        g_adj += mask * gsh
        b_adj += mask * bsh
        weight_sum += mask

    bright_adj /= weight_sum
    r_adj /= weight_sum
    g_adj /= weight_sum
    b_adj /= weight_sum

    out = np.zeros_like(img_f)
    out[:, :, 0] = np.clip(img_f[:, :, 0] + bright_adj * strength + r_adj * strength, 0, 255)
    out[:, :, 1] = np.clip(img_f[:, :, 1] + bright_adj * strength + g_adj * strength, 0, 255)
    out[:, :, 2] = np.clip(img_f[:, :, 2] + bright_adj * strength + b_adj * strength, 0, 255)
    return out.astype(np.uint8)

# ============================================================
# 色彩混合矩阵（3x3）
# ============================================================
def apply_color_matrix(img, matrix):
    img_f = img.astype(np.float32)
    out = np.matmul(img_f, matrix.T)
    return np.clip(out, 0, 255).astype(np.uint8)

# ============================================================
# 胶片颗粒
# ============================================================
def apply_film_grain(img, strength=1.0, color_grain=0.3):
    if strength <= 0:
        return img
    h, w = img.shape[:2]
    img_f = img.astype(np.float32)
    brightness = cv2.cvtColor(img, cv2.COLOR_RGB2GRAY).astype(np.float32) / 255.0
    grain_mask = 0.3 + 1.7 * np.power(1.0 - brightness, 1.2)

    noise_coarse = np.random.normal(0, 25, (h // 4, w // 4)).astype(np.float32)
    noise_coarse = cv2.resize(noise_coarse, (w, h), interpolation=cv2.INTER_LINEAR)
    noise_fine = np.random.normal(0, 8, (h, w)).astype(np.float32)
    combined = (noise_coarse * 0.6 + noise_fine * 0.4) * grain_mask * strength

    r_factor = 1.0 + combined / 255.0
    g_factor = 1.0 + combined / 255.0
    b_factor = 1.0 + combined / 255.0

    img_out = np.zeros_like(img_f)
    img_out[:, :, 0] = np.clip(img_f[:, :, 0] * r_factor, 0, 255)
    img_out[:, :, 1] = np.clip(img_f[:, :, 1] * g_factor, 0, 255)
    img_out[:, :, 2] = np.clip(img_f[:, :, 2] * b_factor, 0, 255)
    return img_out.astype(np.uint8)

# ============================================================
# 暗角
# ============================================================
def apply_vignette(img, strength=0.3):
    if strength <= 0:
        return img
    h, w = img.shape[:2]
    y, x = np.ogrid[:h, :w]
    cx, cy = w / 2.0, h / 2.0
    dist = np.sqrt((x - cx) ** 2 + (y - cy) ** 2)
    max_d = np.sqrt(cx ** 2 + cy ** 2)
    vig = 1.0 - np.power(dist / max_d, 2.2) * strength
    vig = np.clip(vig, 0.2, 1.0).astype(np.float32)
    return np.clip(img.astype(np.float32) * vig[:, :, np.newaxis], 0, 255).astype(np.uint8)


def apply_halation(img, strength=0.15, threshold=0.85, tint=(1.0, 0.70, 0.50)):
    if strength <= 0:
        return img
    h, w = img.shape[:2]
    gray = cv2.cvtColor(img, cv2.COLOR_RGB2GRAY).astype(np.float32)
    bright_mask = np.clip((gray - threshold * 255.0) / 25.0, 0, 1)
    k = max(3, int(min(h, w) * 0.025))
    if k % 2 == 0:
        k += 1
    bright_mask = cv2.GaussianBlur(bright_mask, (k, k), 0)
    glow = np.zeros_like(img, dtype=np.float32)
    glow[:, :, 0] = bright_mask * tint[0] * strength * 60
    glow[:, :, 1] = bright_mask * tint[1] * strength * 35
    glow[:, :, 2] = bright_mask * tint[2] * strength * 20
    result = img.astype(np.float32) + glow
    return np.clip(result, 0, 255).astype(np.uint8)


def apply_color_crossover(img, shadow_tint, highlight_tint, strength=1.0):
    if strength <= 0:
        return img
    gray = cv2.cvtColor(img, cv2.COLOR_RGB2GRAY).astype(np.float32)
    shadow_w = np.clip(1.0 - gray / 128.0, 0, 1)
    highlight_w = np.clip(gray / 128.0 - 1.0, 0, 1)
    shadow_w = np.power(shadow_w, 1.5)
    highlight_w = np.power(highlight_w, 1.5)
    result = img.astype(np.float32)
    for c in range(3):
        result[:, :, c] += shadow_w * shadow_tint[c] * strength * 35
        result[:, :, c] += highlight_w * highlight_tint[c] * strength * 35
    return np.clip(result, 0, 255).astype(np.uint8)


# ============================================================
# 9 分区参数助手
# ============================================================
def _Z(p):
    return [{"bright": p[i][0], "r": p[i][1], "g": p[i][2], "b": p[i][3]}
            for i in range(9)]

# ============================================================
# 胶片风格定义
# ============================================================
FILM_STYLES = {
    "kodak_portra_400": {
        "name": "Kodak Portra 400",
        "desc": "人像首选 | 柔和肤色·低对比·暖阴影",
        "curve": {"shoulder": 0.35, "toe": 0.20, "midpoint": 0.02, "gamma": 1.02,
                  "warm_bias": 0.04, "cool_bias": 0.00, "green_tint": -0.015},
        "zones": _Z([[4, 4, 1, -3], [3, 3, 1, -2], [2, 2, 1, -1],
                     [0, 1, 0, 0], [0, 1, 0, -1], [0, 1, 0, -1],
                     [1, 1, 0, -1], [2, 2, 0, -1], [3, 3, 0, 0]]),
        "matrix": np.array([[0.97, 0.02, 0.01], [0.01, 0.96, 0.03],
                            [0.00, 0.02, 0.98]], dtype=np.float32),
        "saturation": 0.92, "grain": 0.55, "vignette": 0.08,
    },
    "fuji_provia_100f": {
        "name": "Fuji Provia 100F",
        "desc": "专业反转片 | 中性色彩·锐利清晰",
        "curve": {"shoulder": 0.25, "toe": 0.15, "midpoint": 0.00, "gamma": 0.97,
                  "warm_bias": 0.02, "cool_bias": 0.01, "green_tint": 0.005},
        "zones": _Z([[-1, -1, 0, 1], [-1, 0, 0, 0], [0, 0, 0, 0],
                     [0, 0, 1, 0], [1, 0, 1, 0], [1, 0, 1, 0],
                     [2, 0, 0, 0], [3, 1, 0, 0], [2, 1, 0, 0]]),
        "matrix": np.array([[0.99, 0.01, 0.00], [0.00, 1.00, 0.00],
                            [0.01, 0.00, 0.99]], dtype=np.float32),
        "saturation": 1.08, "grain": 0.35, "vignette": 0.04,
    },
    "fuji_velvia_50": {
        "name": "Fuji Velvia 50",
        "desc": "风景首选 | 高饱和·翠绿浓郁·暖阴影",
        "curve": {"shoulder": 0.45, "toe": 0.10, "midpoint": 0.01, "gamma": 0.93,
                  "warm_bias": 0.05, "cool_bias": 0.02, "green_tint": 0.04},
        "zones": _Z([[-3, 2, -1, -2], [-2, 2, -1, -2], [-1, 1, 2, -2],
                     [0, 0, 3, -2], [0, 0, 3, -2], [1, 0, 2, -1],
                     [2, 1, 1, -1], [3, 3, 0, -2], [3, 3, 0, -2]]),
        "matrix": np.array([[1.02, 0.02, -0.02], [-0.02, 1.06, 0.00],
                            [0.00, 0.00, 1.00]], dtype=np.float32),
        "saturation": 1.35, "grain": 0.30, "vignette": 0.10,
    },
    "kodak_ektachrome_100": {
        "name": "Kodak Ektachrome 100",
        "desc": "暖色反转片 | 日落·户外人像",
        "curve": {"shoulder": 0.30, "toe": 0.12, "midpoint": 0.02, "gamma": 0.96,
                  "warm_bias": 0.08, "cool_bias": 0.00, "green_tint": -0.02},
        "zones": _Z([[3, 5, 1, -4], [3, 4, 1, -3], [2, 3, 1, -2],
                     [1, 2, 0, -1], [0, 1, 0, 0], [0, 1, 0, 0],
                     [1, 2, 0, -1], [2, 3, -1, -1], [3, 4, -2, -1]]),
        "matrix": np.array([[1.03, 0.02, 0.00], [0.00, 0.98, 0.02],
                            [0.00, 0.01, 0.99]], dtype=np.float32),
        "saturation": 1.18, "grain": 0.40, "vignette": 0.08,
    },
    "cinestill_800t": {
        "name": "CineStill 800T",
        "desc": "夜景钨丝灯 | 青蓝阴影·暖色高光",
        "curve": {"shoulder": 0.40, "toe": 0.05, "midpoint": -0.01, "gamma": 0.95,
                  "warm_bias": 0.00, "cool_bias": 0.08, "green_tint": -0.01},
        "zones": _Z([[-2, -3, 2, 4], [-1, -3, 2, 3], [0, -3, 2, 3],
                     [0, -2, 1, 2], [0, -1, 0, 1], [1, 0, 0, 0],
                     [2, 2, 0, -1], [3, 4, 1, -2], [3, 5, 0, -2]]),
        "matrix": np.array([[1.05, 0.00, 0.00], [0.00, 0.95, 0.05],
                            [0.05, -0.02, 1.00]], dtype=np.float32),
        "saturation": 0.95, "grain": 1.0, "vignette": 0.18,
    },
    "cinestill_50d": {
        "name": "CineStill 50D",
        "desc": "日光电影片 | 中性街拍·电影感",
        "curve": {"shoulder": 0.32, "toe": 0.10, "midpoint": 0.01, "gamma": 0.98,
                  "warm_bias": 0.03, "cool_bias": 0.01, "green_tint": 0.00},
        "zones": _Z([[-2, -1, 0, 1], [-1, -1, 0, 1], [0, 0, 0, 0],
                     [0, 1, 0, -1], [1, 1, 0, -1], [1, 1, 0, 0],
                     [2, 2, 0, -1], [3, 3, -1, -1], [3, 3, -1, -1]]),
        "matrix": np.array([[1.00, 0.01, 0.00], [0.00, 0.99, 0.01],
                            [0.01, 0.00, 1.00]], dtype=np.float32),
        "saturation": 1.10, "grain": 0.55, "vignette": 0.12,
    },
    "kodak_gold_200": {
        "name": "Kodak Gold 200",
        "desc": "90年代回忆 | 黄调·温暖阳光",
        "curve": {"shoulder": 0.28, "toe": 0.15, "midpoint": 0.02, "gamma": 1.00,
                  "warm_bias": 0.06, "cool_bias": 0.00, "green_tint": 0.025},
        "zones": _Z([[3, 3, 2, -3], [2, 3, 2, -3], [2, 2, 1, -2],
                     [1, 2, 1, -1], [0, 1, 0, -1], [1, 1, 0, -1],
                     [2, 2, 0, -2], [3, 3, 0, -2], [3, 4, -1, -2]]),
        "matrix": np.array([[1.02, 0.02, 0.00], [0.01, 0.98, 0.01],
                            [0.00, 0.01, 0.97]], dtype=np.float32),
        "saturation": 1.08, "grain": 0.55, "vignette": 0.10,
    },
    "kodak_ultramax_400": {
        "name": "Kodak Ultramax 400",
        "desc": "暖调高饱和·浓郁色彩·日常卷王 — 旅行/家庭/街拍",
        "curve": {"shoulder": 0.30, "toe": 0.08, "midpoint": 0.02, "gamma": 0.97,
                  "warm_bias": 0.08, "cool_bias": 0.00, "green_tint": 0.015},
        "zones": _Z([[4, 4, 1, -4], [3, 3, 1, -3], [2, 2, 1, -2],
                     [1, 1, 1, -1], [0, 1, 0, -1], [1, 1, 0, -1],
                     [2, 2, 0, -2], [3, 3, 0, -2], [4, 4, -1, -2]]),
        "matrix": np.array([[1.05, 0.01, 0.00], [0.00, 1.00, 0.00],
                            [0.00, 0.00, 0.96]], dtype=np.float32),
        "saturation": 1.25, "grain": 0.65, "vignette": 0.10,
        "halation_strength": 0.14, "halation_threshold": 0.86,
        "halation_tint": (1.0, 0.72, 0.52),
        "shadow_tint": (0.0, 0.0, 0.03), "highlight_tint": (0.03, 0.0, -0.01),
    },
    "fujifilm_superia_400": {
        "name": "Fuji Superia 400",
        "desc": "日常通用 | 自然饱和·真实绿",
        "curve": {"shoulder": 0.25, "toe": 0.12, "midpoint": 0.01, "gamma": 1.00,
                  "warm_bias": 0.03, "cool_bias": 0.00, "green_tint": 0.035},
        "zones": _Z([[2, 1, 2, -2], [1, 0, 2, -1], [0, 0, 1, -1],
                     [0, 0, 2, -1], [0, 0, 2, -1], [1, 0, 1, 0],
                     [1, 1, 0, 0], [2, 2, 0, -1], [2, 2, -1, -1]]),
        "matrix": np.array([[0.99, 0.01, 0.00], [0.00, 1.02, 0.02],
                            [0.01, 0.00, 0.99]], dtype=np.float32),
        "saturation": 1.12, "grain": 0.55, "vignette": 0.08,
    },
    "ilford_hp5": {
        "name": "Ilford HP5 Plus",
        "desc": "经典黑白 | 中高对比·显著颗粒",
        "curve": {"shoulder": 0.50, "toe": 0.05, "midpoint": 0.00, "gamma": 0.92,
                  "warm_bias": 0.0, "cool_bias": 0.0, "green_tint": 0.0},
        "zones": _Z([[-4, 0, 0, 0], [-3, 0, 0, 0], [-2, 0, 0, 0],
                     [-1, 0, 0, 0], [0, 0, 0, 0], [1, 0, 0, 0],
                     [2, 0, 0, 0], [3, 0, 0, 0], [4, 0, 0, 0]]),
        "matrix": None, "saturation": 0.0, "contrast": 1.18,
        "grain": 1.10, "vignette": 0.08, "grayscale": True,
    },
    "kodak_tri_x_400": {
        "name": "Kodak Tri-X 400",
        "desc": "传奇黑白 | 高对比·纪实感",
        "curve": {"shoulder": 0.42, "toe": 0.03, "midpoint": -0.005, "gamma": 0.90,
                  "warm_bias": 0.0, "cool_bias": 0.0, "green_tint": 0.0},
        "zones": _Z([[-5, 0, 0, 0], [-4, 0, 0, 0], [-3, 0, 0, 0],
                     [-1, 0, 0, 0], [0, 0, 0, 0], [2, 0, 0, 0],
                     [3, 0, 0, 0], [4, 0, 0, 0], [5, 0, 0, 0]]),
        "matrix": None, "saturation": 0.0, "contrast": 1.25,
        "grain": 0.95, "vignette": 0.06, "grayscale": True,
    },
    "ilford_delta_3200": {
        "name": "Ilford Delta 3200",
        "desc": "高感黑白 | 极粗颗粒·夜景",
        "curve": {"shoulder": 0.35, "toe": 0.08, "midpoint": 0.00, "gamma": 0.95,
                  "warm_bias": 0.0, "cool_bias": 0.0, "green_tint": 0.0},
        "zones": _Z([[-3, 0, 0, 0], [-2, 0, 0, 0], [-1, 0, 0, 0],
                     [0, 0, 0, 0], [0, 0, 0, 0], [1, 0, 0, 0],
                     [2, 0, 0, 0], [3, 0, 0, 0], [3, 0, 0, 0]]),
        "matrix": None, "saturation": 0.0, "contrast": 1.15,
        "grain": 1.50, "vignette": 0.14, "grayscale": True,
    },
    "lomo_lc_a": {
        "name": "Lomo LC-A",
        "desc": "LOMO风格 | 暗角浓郁·偏色·高饱和",
        "curve": {"shoulder": 0.38, "toe": 0.08, "midpoint": 0.015, "gamma": 0.95,
                  "warm_bias": 0.05, "cool_bias": 0.00, "green_tint": -0.02},
        "zones": _Z([[5, 6, -2, -3], [4, 5, -2, -3], [3, 4, -1, -2],
                     [1, 2, 0, -1], [0, 1, 0, -1], [1, 1, 0, -1],
                     [3, 3, 0, -2], [4, 4, -1, -2], [5, 5, -2, -2]]),
        "matrix": np.array([[1.06, 0.00, 0.00], [0.00, 0.98, 0.02],
                            [0.04, 0.00, 0.96]], dtype=np.float32),
        "saturation": 1.30, "grain": 0.85, "vignette": 0.35,
    },
    "polaroid_600": {
        "name": "Polaroid 600",
        "desc": "宝丽来即影 | 冷绿调·柔和怀旧",
        "curve": {"shoulder": 0.28, "toe": 0.18, "midpoint": 0.01, "gamma": 1.03,
                  "warm_bias": 0.00, "cool_bias": 0.03, "green_tint": 0.04},
        "zones": _Z([[2, -2, 3, 1], [2, -2, 3, 1], [1, -1, 2, 1],
                     [1, -1, 1, 1], [0, 0, 1, 0], [1, 0, 1, 0],
                     [2, 1, 0, -1], [2, 2, -1, -1], [3, 2, -1, -1]]),
        "matrix": np.array([[0.97, 0.02, 0.01], [0.01, 1.01, 0.02],
                            [0.00, 0.02, 1.00]], dtype=np.float32),
        "saturation": 0.88, "grain": 0.70, "vignette": 0.18,
    },
    "vintage_70s": {
        "name": "Vintage 70s",
        "desc": "70年代复古 | 褐黄褪色·低饱和",
        "curve": {"shoulder": 0.30, "toe": 0.22, "midpoint": 0.00, "gamma": 1.08,
                  "warm_bias": 0.08, "cool_bias": 0.00, "green_tint": 0.02},
        "zones": _Z([[2, 2, 1, -3], [2, 2, 0, -3], [1, 2, 0, -2],
                     [0, 2, 0, -1], [0, 1, 0, -1], [1, 1, 0, -1],
                     [2, 2, -1, -1], [3, 3, -1, -2], [3, 3, -2, -2]]),
        "matrix": np.array([[1.00, 0.03, 0.02], [0.00, 0.95, 0.03],
                            [0.00, 0.02, 0.92]], dtype=np.float32),
        "saturation": 0.78, "grain": 0.75, "vignette": 0.15,
    },
    "fuji_pro_400h": {
        "name": "Fujicolor Pro 400H",
        "desc": "日系人像卷·冷蓝阴影·暖粉高光 — 人像/婚礼/生活",
        "curve": {"shoulder": 0.32, "toe": 0.18, "midpoint": 0.02, "gamma": 1.00,
                  "warm_bias": 0.06, "cool_bias": 0.04, "green_tint": -0.01},
        "zones": _Z([[2, -2, 1, 2], [2, -1, 0, 2], [1, 0, 0, 1],
                     [0, 0, 0, 0], [0, 0, 0, -1], [0, 0, 0, -1],
                     [1, 0, 0, -1], [1, 1, 0, 0], [2, 1, 0, 0]]),
        "matrix": np.array([[0.98, 0.04, 0.00], [0.00, 0.96, 0.02],
                            [0.02, -0.01, 0.99]], dtype=np.float32),
        "saturation": 0.92, "grain": 0.45, "vignette": 0.08,
        "halation_strength": 0.18, "halation_threshold": 0.82,
        "halation_tint": (1.0, 0.75, 0.60),
        "shadow_tint": (0.0, 0.01, 0.05), "highlight_tint": (0.05, 0.01, -0.02),
    },
    "fuji_natura_1600": {
        "name": "Fujicolor Natura 1600",
        "desc": "高速月光卷·暖调浓郁·青绿阴影 — 夜景/室内/街拍",
        "curve": {"shoulder": 0.40, "toe": 0.10, "midpoint": 0.01, "gamma": 0.95,
                  "warm_bias": 0.08, "cool_bias": 0.02, "green_tint": 0.04},
        "zones": _Z([[3, -1, 1, 2], [2, -1, 1, 2], [1, 0, 1, 1],
                     [1, 0, 0, 1], [0, 0, 0, 0], [0, 0, 0, 0],
                     [1, 0, 0, -1], [2, 1, 0, -1], [3, 2, 0, -1]]),
        "matrix": np.array([[1.03, 0.02, 0.00], [0.00, 0.98, 0.02],
                            [0.01, 0.00, 0.97]], dtype=np.float32),
        "saturation": 1.10, "grain": 1.0, "vignette": 0.15,
        "halation_strength": 0.22, "halation_threshold": 0.80,
        "halation_tint": (1.0, 0.70, 0.55),
        "shadow_tint": (0.0, 0.02, 0.04), "highlight_tint": (0.04, 0.00, -0.01),
    },
}

# ============================================================
# 风格应用
# ============================================================
def apply_film_style(image_path, style_name, strength=100, output_path=None, grain_scale=100):
    if style_name not in FILM_STYLES:
        print(f"[ERROR] Unknown style: {style_name}", file=sys.stderr)
        return None

    style = FILM_STYLES[style_name]
    img = imread_unicode(image_path)
    if img is None:
        print(f"[ERROR] 无法读取图片: {image_path}", file=sys.stderr)
        return None

    img = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    img_f = img.astype(np.float32)
    factor = strength / 100.0

    if style.get("grayscale"):
        gray = cv2.cvtColor(img, cv2.COLOR_RGB2GRAY)
        img = cv2.cvtColor(gray, cv2.COLOR_GRAY2RGB)
        img_f = img.astype(np.float32)

    curve_params = style["curve"]
    r_lut, g_lut, b_lut = build_response_curve(
        shoulder=curve_params["shoulder"] * factor,
        toe=curve_params["toe"] * factor,
        midpoint=curve_params["midpoint"] * factor,
        gamma=1.0 + (curve_params["gamma"] - 1.0) * factor,
        warm_bias=curve_params["warm_bias"] * factor,
        cool_bias=curve_params["cool_bias"] * factor,
        green_tint=curve_params["green_tint"] * factor,
    )

    processed = np.zeros_like(img_f)
    processed[:, :, 0] = cv2.LUT(img_f[:, :, 0].astype(np.uint8), r_lut).astype(np.float32)
    processed[:, :, 1] = cv2.LUT(img_f[:, :, 1].astype(np.uint8), g_lut).astype(np.float32)
    processed[:, :, 2] = cv2.LUT(img_f[:, :, 2].astype(np.uint8), b_lut).astype(np.float32)

    zone_params = style.get("zones")
    if zone_params is not None:
        zone_uint8 = np.clip(processed, 0, 255).astype(np.uint8)
        processed = apply_zone_system(zone_uint8, zone_params, strength=factor).astype(np.float32)

    matrix = style.get("matrix")
    if matrix is not None:
        identity = np.eye(3, dtype=np.float32)
        blended = identity * (1.0 - factor) + matrix * factor
        processed = np.matmul(processed, blended.T)
        processed = np.clip(processed, 0, 255)

    sat = style.get("saturation", 1.0)
    if sat != 1.0:
        sat_factor = 1.0 + (sat - 1.0) * factor
        hsv = cv2.cvtColor(processed.astype(np.uint8), cv2.COLOR_RGB2HSV).astype(np.float32)
        hsv[:, :, 1] = np.clip(hsv[:, :, 1] * sat_factor, 0, 255)
        processed = cv2.cvtColor(hsv.astype(np.uint8), cv2.COLOR_HSV2RGB).astype(np.float32)

    contrast = style.get("contrast", 1.0)
    if contrast != 1.0:
        c_factor = 1.0 + (contrast - 1.0) * factor
        processed = np.clip(128.0 + (processed - 128.0) * c_factor, 0, 255)

    if factor < 1.0:
        processed = img_f * (1.0 - factor) + processed * factor

    processed = np.clip(processed, 0, 255).astype(np.uint8)

    grain_scale_factor = grain_scale / 100.0
    grain = style.get("grain", 0.5) * factor * grain_scale_factor
    if grain > 0:
        processed = apply_film_grain(processed, strength=grain, color_grain=0.25)

    vignette = style.get("vignette", 0.0) * factor
    if vignette > 0:
        processed = apply_vignette(processed, strength=vignette)

    hl = style.get("halation_strength", 0.0) * factor
    if hl > 0:
        processed = apply_halation(
            processed,
            strength=hl,
            threshold=style.get("halation_threshold", 0.85),
            tint=style.get("halation_tint", (1.0, 0.70, 0.50)),
        )

    sh_tint = style.get("shadow_tint", (0.0, 0.0, 0.0))
    hl_tint = style.get("highlight_tint", (0.0, 0.0, 0.0))
    if any(x != 0 for x in sh_tint) or any(x != 0 for x in hl_tint):
        processed = apply_color_crossover(
            processed, shadow_tint=sh_tint, highlight_tint=hl_tint, strength=factor
        )

    if output_path is None:
        output_path = Path(image_path).stem + f"_film_{style_name}.jpg"

    imwrite_unicode(output_path, cv2.cvtColor(processed, cv2.COLOR_RGB2BGR), 95)
    return output_path

# ============================================================
# 图片分析
# ============================================================
def _find_cascade():
    candidates = []
    try:
        if hasattr(cv2, "data") and hasattr(cv2.data, "haarcascades"):
            candidates.append(os.path.join(cv2.data.haarcascades,
                                            "haarcascade_frontalface_default.xml"))
    except Exception:
        pass
    try:
        candidates.append(os.path.join(os.path.dirname(cv2.__file__), "data",
                                        "haarcascade_frontalface_default.xml"))
    except Exception:
        pass
    for attr in ("_MEIPASS",):
        if hasattr(sys, attr):
            base = getattr(sys, attr)
            candidates.append(os.path.join(base, "cv2", "data",
                                            "haarcascade_frontalface_default.xml"))
            candidates.append(os.path.join(base, "cv2_data",
                                            "haarcascade_frontalface_default.xml"))
    try:
        here = Path(__file__).resolve().parent
        candidates.append(str(here / "cv2" / "data" / "haarcascade_frontalface_default.xml"))
        candidates.append(str(here / "cv2_data" / "haarcascade_frontalface_default.xml"))
    except Exception:
        pass
    try:
        if sys.argv and sys.argv[0]:
            exe_dir = Path(sys.argv[0]).resolve().parent
            candidates.append(str(exe_dir / "cv2_data" / "haarcascade_frontalface_default.xml"))
    except Exception:
        pass
    for p in candidates:
        try:
            if p and os.path.isfile(p):
                return p
        except Exception:
            continue
    return None


def detect_faces_ratio(img):
    """返回人脸区域占图片总面积的比例(0~1)。仅判断有无显著人脸，不关心具体个数。"""
    try:
        cascade_path = _find_cascade()
        if not cascade_path:
            return 0.0
        h, w = img.shape[:2]
        min_face = max(60, int(min(h, w) * 0.08))
        gray = cv2.cvtColor(img, cv2.COLOR_BGR2GRAY)
        faces = cv2.CascadeClassifier(cascade_path).detectMultiScale(
            gray, 1.1, 8, minSize=(min_face, min_face))
        if len(faces) == 0:
            return 0.0
        total_area = sum(fw * fh for (_, _, fw, fh) in faces)
        return min(total_area / (h * w), 1.0)
    except Exception:
        return 0.0


def analyze_image(image_path):
    supported, hint = check_format_supported(image_path)
    if not supported:
        pass
    img = imread_unicode(image_path)
    if img is None:
        return {"ok": False, "recommended": "kodak_portra_400",
                "reason": "读取失败", "analysis": {}}

    img_rgb = cv2.cvtColor(img, cv2.COLOR_BGR2RGB)
    brightness = float(np.mean(img_rgb))
    r_mean = float(np.mean(img_rgb[:, :, 0]))
    b_mean = float(np.mean(img_rgb[:, :, 2]))
    color_temp = r_mean - b_mean
    face_ratio = detect_faces_ratio(img)
    has_faces = face_ratio > 0.005
    is_night = brightness < 75
    is_low_light = brightness < 110

    analysis = {"brightness": round(brightness, 1),
                "color_temp": round(color_temp, 1),
                "face_ratio": round(face_ratio, 4), "tags": []}
    tags = []
    if has_faces:
        tags.append("人像")
    if is_night:
        tags.append("夜景")
    elif is_low_light:
        tags.append("弱光")
    if color_temp > 10:
        tags.append("原片偏暖")
    elif color_temp < -5:
        tags.append("原片偏冷")
    analysis["tags"] = tags

    scores = {k: 0.0 for k in FILM_STYLES}
    if has_faces:
        if is_low_light:
            scores["ilford_hp5"] += 30
            scores["kodak_portra_400"] += 25
            scores["fuji_natura_1600"] += 22
            scores["kodak_tri_x_400"] += 20
        else:
            scores["kodak_portra_400"] += 35
            scores["fuji_pro_400h"] += 28
            scores["kodak_gold_200"] += 20
            scores["fuji_provia_100f"] += 15
            scores["polaroid_600"] += 12
    elif is_night:
        scores["cinestill_800t"] += 35
        scores["fuji_natura_1600"] += 30
        scores["ilford_delta_3200"] += 25
        scores["kodak_ultramax_400"] += 18
        scores["lomo_lc_a"] += 15
    else:
        if color_temp < 0:
            scores["fuji_velvia_50"] += 25
        scores["kodak_gold_200"] += 22
        scores["kodak_ultramax_400"] += 20
        scores["fujifilm_superia_400"] += 18
        scores["cinestill_50d"] += 15
        if color_temp > 10:
            scores["fuji_pro_400h"] += 10

    sorted_scores = sorted(scores.items(), key=lambda x: -x[1])
    best_key = sorted_scores[0][0]
    parts = []
    if has_faces:
        parts.append("人像")
    if is_low_light:
        parts.append("弱光")
    else:
        parts.append("正常光照")
    reason = f"{' · '.join(parts)} · 推荐 {FILM_STYLES[best_key]['name']}"

    return {
        "ok": True,
        "recommended": best_key,
        "recommended_name": FILM_STYLES[best_key]["name"],
        "reason": reason,
        "analysis": analysis,
        "top3": [{"key": k, "name": FILM_STYLES[k]["name"],
                  "desc": FILM_STYLES[k]["desc"], "score": round(s, 1)}
                 for k, s in sorted_scores[:3]],
    }

# ============================================================
# GUI
# ============================================================
def run_gui(input_path=None):
    import tkinter as tk
    from tkinter import ttk, filedialog, messagebox

    root = tk.Tk()
    root.title("胶片调色")
    root.geometry("640x560")

    file_frame = ttk.LabelFrame(root, text="图片")
    file_frame.pack(fill="x", padx=10, pady=5)
    path_var = tk.StringVar(value=input_path or "")
    ttk.Entry(file_frame, textvariable=path_var, width=55).pack(side="left", padx=5, pady=5)
    ttk.Button(file_frame, text="浏览",
               command=lambda: path_var.set(filedialog.askopenfilename(
                   filetypes=[("图片", "*.jpg *.jpeg *.png *.bmp *.tiff *.tif")]))).pack(side="left", padx=5)

    info_frame = ttk.LabelFrame(root, text="分析")
    info_frame.pack(fill="x", padx=10, pady=5)
    info_text = tk.Text(info_frame, height=4, width=75, wrap="word")
    info_text.pack(padx=5, pady=5)

    def do_analyze():
        p = path_var.get().strip()
        if not p or not os.path.exists(p):
            return
        result = analyze_image(p)
        info_text.config(state="normal")
        info_text.delete("1.0", "end")
        if result.get("ok"):
            a = result["analysis"]
            info_text.insert("end",
                f"内容: {', '.join(a.get('tags', [])) or '通用场景'}\n")
            info_text.insert("end",
                f"亮度: {a['brightness']}   色温偏移: {a['color_temp']}\n")
            info_text.insert("end", f"推荐: {result['recommended_name']} — ")
            for i, t in enumerate(result.get("top3", [])):
                info_text.insert("end", f"{i+1}.{t['name']} ")
        else:
            info_text.insert("end", f"分析失败: {result.get('error', '未知')}")
        info_text.config(state="disabled")

    ttk.Button(info_frame, text="分析图片", command=do_analyze).pack(pady=2)

    style_frame = ttk.LabelFrame(root, text="胶片风格")
    style_frame.pack(fill="x", padx=10, pady=5)

    style_names = [f"{FILM_STYLES[k]['name']} — {FILM_STYLES[k]['desc']}" for k in FILM_STYLES]
    style_keys = list(FILM_STYLES.keys())
    style_combo = ttk.Combobox(style_frame, values=style_names, width=75, state="readonly")
    style_combo.current(0)
    style_combo.pack(padx=5, pady=5)

    str_frame = ttk.LabelFrame(root, text="强度（越大胶片味越浓）")
    str_frame.pack(fill="x", padx=10, pady=5)
    strength_var = tk.IntVar(value=100)
    slider = ttk.Scale(str_frame, from_=0, to=150, variable=strength_var, orient="horizontal")
    slider.pack(fill="x", padx=10, pady=5)
    val_label = ttk.Label(str_frame, textvariable=tk.StringVar(value="100%"))
    val_label.pack(pady=2)

    def update_val(*args):
        val_label.config(text=f"{strength_var.get()}%")
    strength_var.trace("w", update_val)

    grain_frame = ttk.LabelFrame(root, text="颗粒度（0=无颗粒，200=重颗粒）")
    grain_frame.pack(fill="x", padx=10, pady=5)
    grain_var = tk.IntVar(value=100)
    grain_slider = ttk.Scale(grain_frame, from_=0, to=200, variable=grain_var, orient="horizontal")
    grain_slider.pack(fill="x", padx=10, pady=5)
    grain_label = ttk.Label(grain_frame, textvariable=tk.StringVar(value="100%"))
    grain_label.pack(pady=2)

    def update_grain(*args):
        grain_label.config(text=f"{grain_var.get()}%")
    grain_var.trace("w", update_grain)

    def do_process():
        p = path_var.get().strip()
        if not p or not os.path.exists(p):
            messagebox.showwarning("提示", "请先选择图片")
            return
        style_key = style_keys[style_combo.current()]
        out = str(Path(p).parent / (Path(p).stem + f"_film_{style_key}.jpg"))
        result = apply_film_style(p, style_key, strength_var.get(), out, grain_var.get())
        if result:
            messagebox.showinfo("完成", f"已输出: {result}")
        else:
            messagebox.showerror("失败", "处理失败")

    btn_frame = ttk.Frame(root)
    btn_frame.pack(fill="x", padx=10, pady=10)
    ttk.Button(btn_frame, text="处理", command=do_process).pack(side="left", padx=5)
    ttk.Button(btn_frame, text="退出", command=root.destroy).pack(side="right", padx=5)

    if input_path and os.path.exists(input_path):
        root.after(100, do_analyze)

    root.mainloop()

# ============================================================
# 主入口
# ============================================================
def main():
    parser = argparse.ArgumentParser(description="胶片调色系统")
    parser.add_argument("--input", help="输入图片路径（支持中文、空格）")
    parser.add_argument("--output", help="输出图片路径")
    parser.add_argument("--style", choices=list(FILM_STYLES.keys()), help="指定胶片风格")
    parser.add_argument("--strength", type=int, default=100, help="强度 0-150")
    parser.add_argument("--grain", type=int, default=100, help="颗粒度 0-200（默认 100）")
    parser.add_argument("--auto", action="store_true", help="自动分析推荐风格")
    parser.add_argument("--analyze", help="仅分析图片（输出 JSON）")
    parser.add_argument("--json-output", help="将 --analyze 结果写入该文件")
    parser.add_argument("--list-styles", action="store_true", help="列出所有风格")
    parser.add_argument("--gui", action="store_true", help="启动 GUI")
    parser.add_argument("--ps-mode", action="store_true", help="PS 脚本调用（保留兼容）")
    args = parser.parse_args()

    if args.gui:
        run_gui(args.input)
        return

    if args.list_styles:
        for k, v in FILM_STYLES.items():
            print(f"  {k}: {v['name']} — {v['desc']}")
        return

    if args.analyze:
        report = analyze_image(args.analyze)
        report_json = json.dumps(report, ensure_ascii=False, indent=2)
        if args.json_output:
            try:
                out = Path(args.json_output)
                if str(out.parent) and not out.parent.exists():
                    out.parent.mkdir(parents=True, exist_ok=True)
                out.write_text(report_json, encoding="utf-8")
            except Exception as e:
                print(f"[ERROR] write json: {e}", file=sys.stderr)
                sys.exit(1)
        print(report_json)
        return

    if not args.input:
        print("[ERROR] 必须提供 --input", file=sys.stderr)
        sys.exit(1)

    input_path = args.input

    supported, hint = check_format_supported(input_path)
    if not supported:
        print(f"[WARN] {hint}", file=sys.stderr)

    if not os.path.isfile(input_path):
        print(f"[ERROR] 文件不存在: {input_path}", file=sys.stderr)
        sys.exit(1)

    output_path = args.output or (Path(input_path).stem + "_film.jpg")

    if args.auto or not args.style:
        report = analyze_image(input_path)
        style = report["recommended"]
    else:
        style = args.style

    result = apply_film_style(input_path, style, args.strength, output_path, args.grain)
    if not result:
        print(f"[ERROR] 处理失败，请检查输入文件是否为有效图片", file=sys.stderr)
        sys.exit(1)

    print(f"[OK] 已生成: {output_path}  (风格: {FILM_STYLES[style]['name']})")

if __name__ == "__main__":
    main()
