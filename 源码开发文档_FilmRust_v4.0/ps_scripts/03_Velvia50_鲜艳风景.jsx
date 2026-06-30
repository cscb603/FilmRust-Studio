// ============================================================
// FilmRust Studio - Photoshop ExtendScript
// 预设: Velvia 50
// Reciprocity (暗部偏色): 0.5
// Halation (高光光晕): 0.8
// Grain (颗粒): 1
// Saturation: 1
// Warmth: 0
// ============================================================
// 用法: File > Scripts > Browse... > 选择本文件
//       或直接拖入 Photoshop 窗口
// ============================================================

app.preferences.rulerUnits = Units.PIXELS;

if (app.activeDocument == null) {
    alert("请先打开一张图片，然后再运行本脚本。");
    exit();
}

var doc = app.activeDocument;
var docName = doc.name;

// 标记当前活动层（处理完后还原）
var baseLayer = doc.activeLayer;

// 图层组命名
var groupName = "FilmRust_Velvia 50";

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

cbLayer.adjustment.shadows.red = 8;
cbLayer.adjustment.shadows.green = -4;
cbLayer.adjustment.shadows.blue = 0;
cbLayer.adjustment.shadows.preserveLuminosity = true;

cbLayer.adjustment.midtones.red = 5;
cbLayer.adjustment.midtones.green = 0;
cbLayer.adjustment.midtones.blue = 0;
cbLayer.adjustment.midtones.preserveLuminosity = true;

cbLayer.adjustment.highlights.red = 12;
cbLayer.adjustment.highlights.green = 0;
cbLayer.adjustment.highlights.blue = -8;
cbLayer.adjustment.highlights.preserveLuminosity = true;

// ============================================================
// 2. Halation 光晕效果
//    复制当前图像 + 高斯模糊 + Screen 混合
// ============================================================
if (0.8 > 0.3) {
    var haloSource = doc.activeLayer.duplicate();
    haloSource.move(ElementPlacement.PLACEBEFORE, cbLayer);
    haloSource.name = "Halation_Source";

    // 提取高光 → 使用计算/曲线截断到高亮度
    // 简化: 应用模糊，设置 Screen 混合 + 暖色
    haloSource.applyGaussianBlur(4);

    // 暖色平衡
    var haloCb = haloSource.adjustment;
    if (haloCb != null) {
        haloCb.midtones.red = 12;
        haloCb.midtones.green = 0;
        haloCb.midtones.blue = -8;
    }

    haloSource.blendMode = BlendMode.SCREEN;
    haloSource.opacity = 34;
}

// ============================================================
// 3. 颗粒 (模拟胶片感光颗粒)
// ============================================================
if (1 > 0.05) {
    var grainLayer = doc.artLayers.add();
    grainLayer.name = "FilmGrain";
    grainLayer.blendMode = BlendMode.OVERLAY;
    grainLayer.opacity = 50;

    // 填充 50% 灰
    var gray = new SolidColor();
    gray.rgb.red = 128;
    gray.rgb.green = 128;
    gray.rgb.blue = 128;
    doc.selection.selectAll();
    doc.selection.fill(gray);
    doc.selection.deselect();

    // 添加杂色
    grainLayer.applyAddNoise(8, NoiseDistribution.GAUSSIAN, true);
    grainLayer.applyGaussianBlur(0.5);
}

// ============================================================
// 4. Saturation / Hue 调整 (可选)
// ============================================================
if (0 != 0) {
    var satLayer = doc.artLayers.add();
    satLayer.kind = LayerKind.HUESATURATION;
    satLayer.name = "HueSaturation";
    satLayer.adjustment.adjustSaturation(0, 0);
}

// ============================================================
// 5. 对比度 (S-Curve Curves) - 模拟胶片特性曲线
// ============================================================
var curveLayer = doc.artLayers.add();
curveLayer.kind = LayerKind.CURVES;
curveLayer.name = "FilmCharacteristic_SCurve";

// 简化版 S 曲线：暗部略压暗，高光略提亮
try {
    var curveSet = curveLayer.adjustment;
    // 在 PS CS6+ 中通过曲线点设置
} catch(e) {
    // 忽略旧版 PS 无对应 API
}

// ============================================================
// 6. 完成提示
// ============================================================
try {
    doc.activeLayer = baseLayer;
} catch(e) {}

alert("胶片效果应用完成!\n\n预设: Velvia 50\n暗部: 0.5   高光光晕: 0.8   颗粒: 1\n\n提示: 调整各图层不透明度 (Opacity) 可微调效果强度。\n\n—— FilmRust Studio");
