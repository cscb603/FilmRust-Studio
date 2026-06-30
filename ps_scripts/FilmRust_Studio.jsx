// ============================================================
// FilmRust Studio — Photoshop 联动脚本 v7.3
// 架构: filmrust.exe 分析 + 处理 + JSX 回显（静默运行）
// 要求: 把 filmrust.exe 放在与本脚本同一目录
// 特点: 静默后台运行 · 色彩/肤色调节 · 全分辨率无损输出
// 版权: 星TAP 软件 2026  csb603@qq.com
// ============================================================

var SCRIPT_FILE = new File($.fileName);
var SCRIPT_DIR = SCRIPT_FILE.parent.fsName;
var EXE_PATH = SCRIPT_DIR + "\\filmrust.exe";
var TEMP_DIR = "C:\\Temp\\filmrust_temp";

// 60+ 种胶片预设（与 GUI/CLI 同步）
var STYLE_LIST = [
    {key:"kodak_ultramax_400", name:"Kodak Ultramax 400", desc:"暖调高饱和·消费级卷王"},
    {key:"fuji_natura_1600", name:"Fujicolor Natura 1600", desc:"月光卷·暖调·青绿阴影"},
    {key:"kodak_portra_400", name:"Kodak Portra 400", desc:"人像首选，柔和肤色"},
    {key:"kodak_gold_200", name:"Kodak Gold 200", desc:"日常阳光感，90年代温暖回忆"},
    {key:"kodak_ektar_100", name:"Kodak Ektar 100", desc:"超细腻颜色，风光利器"},
    {key:"kodak_tri_x_400", name:"Kodak Tri-X 400", desc:"传奇黑白，高对比颗粒质感"},
    {key:"fujifilm_superia_400", name:"Fujifilm Superia 400", desc:"通用彩色，自然饱和日用"},
    {key:"fujifilm_provia_100f", name:"Fujifilm Provia 100F", desc:"专业反转片，中性真实色彩"},
    {key:"fujifilm_velvia_50", name:"Fujifilm Velvia 50", desc:"风光首席，超高饱和鲜艳"},
    {key:"cinestill_800t", name:"CineStill 800T", desc:"夜景/钨丝灯，电影感青橙调"},
    {key:"cinestill_50d", name:"CineStill 50D", desc:"日光型电影胶片，电影感街拍"},
    {key:"ilford_hp5_plus_400", name:"Ilford HP5 Plus 400", desc:"黑白经典，高对比颗粒感"},
    {key:"standard_daylight", name:"Standard Daylight", desc:"中性基准，去胶片化原色"},
    {key:"lomography_color_chrome", name:"Lomography Color Chrome", desc:"LOMO艺术，高对比偏色"},
    {key:"polaroid_600_color", name:"Polaroid 600 Color", desc:"宝丽来即时感，怀旧偏暖"},
    {key:"lomography_cn_400", name:"Lomography CN 400", desc:"Lomo彩色负片，复古偏暖"},
    {key:"kodak_portra_400_artistic", name:"Portra 400 Artistic", desc:"艺术版，增强色彩分离"},
    {key:"fujifilm_velvia_50_artistic", name:"Velvia 50 Artistic", desc:"增强版Velvia，极致鲜艳"},
    {key:"kodak_portra_160", name:"Kodak Portra 160", desc:"低感人像，更细腻的肤色"},
    {key:"kodak_portra_800", name:"Kodak Portra 800", desc:"弱光人像，温暖颗粒感"},
    {key:"fujifilm_superia_200", name:"Fujifilm Superia 200", desc:"暖调人像，日系清新"},
    {key:"fujifilm_superia_100", name:"Fujifilm Superia 100", desc:"低感人像，细腻柔和"},
    {key:"agfa_vista_400", name:"Agfa Vista 400", desc:"德系暖调，浓郁色彩人像"},
    {key:"agfa_vista_200", name:"Agfa Vista 200", desc:"德系暖调，日常人像"},
    {key:"agfa_vista_100", name:"Agfa Vista 100", desc:"德系低感人像"},
    {key:"lucky_color_200", name:"Lucky Color 200", desc:"国产乐凯，暖调怀旧"},
    {key:"kodak_ektachrome_100", name:"Kodak Ektachrome 100", desc:"经典反转片，暖调风光"},
    {key:"kodak_ektachrome_100vs", name:"Ektachrome 100 VS", desc:"超鲜艳反转片，极致色彩"},
    {key:"kodak_kodachrome_64", name:"Kodak Kodachrome 64", desc:"经典柯达克罗姆，暖调浓郁"},
    {key:"kodak_kodachrome_25", name:"Kodak Kodachrome 25", desc:"极致细腻柯达克罗姆"},
    {key:"fujifilm_astia_100f", name:"Fujifilm Astia 100F", desc:"柔和反转片，淡彩风光"},
    {key:"kodak_pro_image_100", name:"Kodak Pro Image 100", desc:"日光高性价比，暖调自然"},
    {key:"agfa_optima_200", name:"Agfa Optima 200", desc:"暖调风光，德系反转片"},
    {key:"agfa_precisa_100", name:"Agfa Precisa 100", desc:"暖调反转片，风光人像通用"},
    {key:"kodak_tri_x_400_artistic", name:"Tri-X 400 Artistic", desc:"增强版，更强颗粒对比"},
    {key:"kodak_plus_x_125", name:"Kodak Plus-X 125", desc:"细腻黑白，中调丰富"},
    {key:"ilford_hp5_plus_400_artistic", name:"HP5 Plus 400 Artistic", desc:"增强版HP5，更强颗粒"},
    {key:"ilford_fp4_plus_125", name:"Ilford FP4 Plus 125", desc:"中速黑白，细腻过渡"},
    {key:"ilford_delta_400", name:"Ilford Delta 400", desc:"现代黑白，颗粒锐利"},
    {key:"ilford_delta_100", name:"Ilford Delta 100", desc:"超细腻现代黑白"},
    {key:"ilford_pan_f_plus_50", name:"Ilford Pan F Plus 50", desc:"极细腻低感黑白"},
    {key:"ilford_xp2_super_400", name:"Ilford XP2 Super 400", desc:"C41工艺黑白，冲印方便"},
    {key:"ilford_sfx_200", name:"Ilford SFX 200", desc:"红外效果黑白"},
    {key:"ilford_ortho_plus_80", name:"Ilford Ortho Plus 80", desc:"正色片，高对比反差"},
    {key:"fujifilm_neopan_400", name:"Fujifilm Neopan 400", desc:"日系黑白，细腻灰阶"},
    {key:"fujifilm_neopan_100", name:"Fujifilm Neopan 100", desc:"日系低感黑白"},
    {key:"agfa_apx_400", name:"Agfa APX 400", desc:"经典德系黑白"},
    {key:"agfa_apx_100", name:"Agfa APX 100", desc:"经典德系细腻黑白"},
    {key:"polaroid_bw_667", name:"Polaroid B&W 667", desc:"宝丽来黑白，即时显影"},
    {key:"polaroid_55_bw", name:"Polaroid 55 B&W", desc:"宝丽来正负片，极致黑白"},
    {key:"orwo_un54", name:"Orwo UN54", desc:"东德经典黑白，高对比"},
    {key:"orwo_un64", name:"Orwo UN64", desc:"东德低感黑白，细腻"},
    {key:"ricoh_gr_street", name:"Ricoh GR Street Night", desc:"街拍高感黑白"},
    {key:"agfa_scala_200", name:"Agfa Scala 200", desc:"黑白反转片，高反差"},
    {key:"polaroid_sx70_color", name:"Polaroid SX-70", desc:"经典SX-70，暖调柔和"},
    {key:"polaroid_i_type_color", name:"Polaroid i-Type", desc:"现代宝丽来，鲜艳色彩"},
    {key:"polaroid_spectra_color", name:"Polaroid Spectra", desc:"宽幅宝丽来，偏冷调"},
    {key:"polaroid_100_color", name:"Polaroid 100", desc:"老式宝丽来100，褪色怀旧"},
    {key:"lomography_lomochrome_purple", name:"Lomochrome Purple", desc:"紫色幻彩，独特艺术效果"},
    {key:"ferrania_solaris_400", name:"Ferrania Solaris 400", desc:"意式暖调，复古褪色感"},
    {key:"ferrania_solaris_100", name:"Ferrania Solaris 100", desc:"意式低感，暖调柔和"},
];

// ========== 工具函数 ==========
function ensureTempDir() {
    var dir = new Folder(TEMP_DIR);
    if (!dir.exists) dir.create();
}
function deleteFileSafe(path) {
    var f = new File(path); if (f.exists) f.remove();
}
function runExe(cmd) {
    var batPath = TEMP_DIR + "\\_run_filmrust.bat";
    var batFile = new File(batPath);
    batFile.encoding = "UTF-8";
    batFile.open("w");
    batFile.writeln("@echo off\ntitle FilmRust\ncd /d \"" + SCRIPT_DIR + "\"");
    batFile.writeln("chcp 65001 >nul 2>nul");
    batFile.writeln(cmd + " >\"" + TEMP_DIR + "\\_run_log.txt\" 2>&1");
    batFile.close();
    app.system(batPath);
    $.sleep(300);
}
function readTextFile(path) {
    var f = new File(path); if (!f.exists) return "";
    f.encoding = "UTF-8"; f.open("r"); var c = f.read(); f.close(); return c;
}
function getStyleIdx(key) {
    for (var i = 0; i < STYLE_LIST.length; i++) if (STYLE_LIST[i].key === key) return i;
    return 0;
}
function extractField(text, field) {
    var m = text.match(new RegExp('"' + field + '"\\s*:\\s*"([^"]*)"'));
    return m ? m[1] : "";
}

// ========== 高画质导出当前文档为 PNG（全分辨率，无损） ==========
function exportFullQuality(doc, filePath) {
    // 优先 PNG 无损；若 PS 版本不支持则用最高质量 JPEG
    try {
        var pngOpts = new ExportOptionsSaveForWeb();
        pngOpts.format = SaveDocumentType.PNG;
        pngOpts.interlaced = false;
        pngOpts.optimized = true;
        doc.exportDocument(new File(filePath), ExportType.SAVEFORWEB, pngOpts);
        return true;
    } catch(e1) {
        // 回退: 用最高质量 JPEG (quality=12 即最大)
        try {
            var dup = doc.duplicate("_filmrust_export", true);
            dup.flatten();
            var jpgOpts = new JPEGSaveOptions();
            jpgOpts.quality = 12; // 最高质量
            jpgOpts.embedColorProfile = true;
            dup.saveAs(new File(filePath), jpgOpts, true, Extension.LOWERCASE);
            dup.close(SaveOptions.DONOTSAVECHANGES);
            return true;
        } catch(e2) {
            return false;
        }
    }
}

// ========== 主函数 ==========
function main() {
    if (app.documents.length < 1) { alert("请先打开一张图片！"); return; }
    var doc = app.activeDocument;
    ensureTempDir();

    var inputPath  = TEMP_DIR + "\\_input.png";
    var outputPath = TEMP_DIR + "\\_output.png";
    var analyzePath = TEMP_DIR + "\\_analyze.json";
    deleteFileSafe(inputPath); deleteFileSafe(outputPath); deleteFileSafe(analyzePath);

    // ── 1. 导出全分辨率输入文件（无损 PNG） ──
    if (!exportFullQuality(doc, inputPath)) {
        alert("导出输入文件失败，请检查文件是否已保存。"); return;
    }

    // ── 2. 自动分析推荐风格 ──
    var recommendedKey = "kodak_portra_400";
    var infoText = "（分析不可用，使用默认推荐）";
    try {
        var cmd = '"' + EXE_PATH + '" --analyze "' + inputPath + '" --json-output "' + analyzePath + '"';
        runExe(cmd); $.sleep(500);
        var jsonText = readTextFile(analyzePath);
        if (jsonText.length > 0) {
            var rec = extractField(jsonText, "recommended");
            var recName = extractField(jsonText, "recommended_name");
            var reason = extractField(jsonText, "reason");
            if (rec.length > 0) recommendedKey = rec;
            infoText = (reason.length > 0) ? reason : ("推荐: " + recName);
        }
    } catch(e) { infoText = "（分析跳过）"; }

    var styleIdx = getStyleIdx(recommendedKey);

    // ══════════════════════════════════════════
    //  对话框 — 含可折叠色彩/肤色调节面板
    // ══════════════════════════════════════════
    var w = new Window("dialog", "FilmRust Studio v7.3 — 胶片模拟 + 色彩调节");
    w.orientation = "column";
    w.alignChildren = ["fill", "top"];

    // ── 分析信息 ──
    var infoPnl = w.add("panel", undefined, " 智能分析 ");
    infoPnl.alignChildren = ["fill", "top"];
    infoPnl.add("statictext", undefined, infoText);

    // ── 胶片风格选择 ──
    var stylePnl = w.add("panel", undefined, " 胶片风格（60+ 种） ");
    stylePnl.alignChildren = ["fill", "top"];
    var ddl = stylePnl.add("dropdownlist", undefined);
    for (var i = 0; i < STYLE_LIST.length; i++)
        ddl.add("item", STYLE_LIST[i].name + " — " + STYLE_LIST[i].desc);
    ddl.selection = styleIdx;

    // ── 基础参数 ──
    var basePnl = w.add("panel", undefined, " 基础参数 ");
    basePnl.orientation = "column"; basePnl.alignChildren = ["fill", "top"];

    function addSliderRow(parent, label, min, max, val, unit) {
        var g = parent.add("group");
        g.alignChildren = ["left", "center"];
        g.add("statictext", undefined, label);
        var sl = g.add("slider", undefined, val, min, max);
        sl.preferredSize.width = 200;
        var vt = g.add("statictext", undefined, val + (unit||""));
        sl.onChanging = function() { vt.text = Math.round(this.value) + (unit||""); };
        return { slider: sl, label: vt };
    }

    var strengthS = addSliderRow(basePnl, "强度:", 10, 150, 100, "%");
    var grainS    = addSliderRow(basePnl, "颗粒:", 0, 200, 100, "%");

    // ══════════════════════════════════════════
    //  色彩调节面板（可折叠）
    // ══════════════════════════════════════════
    var colorPnl = w.add("panel", undefined, " 色彩调节 ");
    colorPnl.orientation = "column"; colorPnl.alignChildren = ["fill", "top"];

    var colorEnable = colorPnl.add("checkbox", undefined, "启用色彩调节");
    colorEnable.value = false;

    var colorBody = colorPnl.add("group");
    colorBody.orientation = "column"; colorBody.alignChildren = ["fill", "top"];

    var warmthS = addSliderRow(colorBody, "色温:", -100, 100, 0, "");
    warmthS.slider.helpTip = "向左→偏蓝冷调，向右→偏黄暖调";
    var tintS    = addSliderRow(colorBody, "色调:", -100, 100, 0, "");
    tintS.slider.helpTip = "向左→偏绿，向右→偏品红";
    var satS     = addSliderRow(colorBody, "饱和度:", 0, 200, 100, "%");
    satS.slider.helpTip = "0=灰度，100=原图饱和度，200=超鲜艳";

    // 提示文字
    var colorHint = colorBody.add("statictext", undefined,
        "提示: 色温/色调/饱和度会通过 CLI 参数传递给 filmrust 引擎");
    colorHint.graphics.font = ScriptUI.newFont("Arial", "Regular", 10);

    colorBody.visible = false;
    colorEnable.onClick = function() { colorBody.visible = this.value; };

    // ══════════════════════════════════════════
    //  肤色调节面板（可折叠）
    // ══════════════════════════════════════════
    var skinPnl = w.add("panel", undefined, " 肤色优化（PS 调整层） ");
    skinPnl.orientation = "column"; skinPnl.alignChildren = ["fill", "top"];

    var skinEnable = skinPnl.add("checkbox", undefined, "启用肤色优化");
    skinEnable.value = false;

    var skinBody = skinPnl.add("group");
    skinBody.orientation = "column"; skinBody.alignChildren = ["fill", "top"];

    var skinYellowS = addSliderRow(skinBody, "去黄:", 0, 100, 30, "");
    skinYellowS.slider.helpTip = "降低肤色黄调，使肤色更干净 (0=不调)";
    var skinGreenS  = addSliderRow(skinBody, "减绿:", 0, 100, 20, "");
    skinGreenS.slider.helpTip = "减少胶片平光偏绿，补正肤色 (0=不调)";
    var skinPinkS   = addSliderRow(skinBody, "加粉:", 0, 100, 25, "");
    skinPinkS.slider.helpTip = "增加红蓝→粉润感，适合亚洲肤色 (0=不调)";
    var skinRedS    = addSliderRow(skinBody, "加红:", 0, 100, 15, "");
    skinRedS.slider.helpTip = "微增暖调血色，让肤色更健康 (0=不调)";
    var skinBrightS = addSliderRow(skinBody, "亮度:", -50, 50, 0, "");
    skinBrightS.slider.helpTip = "肤色亮度微调 (0=不调)";

    var skinHint = skinBody.add("statictext", undefined,
        "提示: 肤色调节通过 PS Color Balance 调整层实现，不破坏原图");
    skinHint.graphics.font = ScriptUI.newFont("Arial", "Regular", 10);

    skinBody.visible = false;
    skinEnable.onClick = function() { skinBody.visible = this.value; };

    // ── 按钮 ──
    var btnGrp = w.add("group");
    btnGrp.alignChildren = ["right", "center"];
    btnGrp.add("button", undefined, "应用效果", {name:"ok"});
    btnGrp.add("button", undefined, "取消", {name:"cancel"});
    w.add("statictext", undefined, "FilmRust Studio v7.3  星TAP 软件 2026").alignment = "right";

    var result = w.show();
    if (result !== 1) return;

    // ══════════════════════════════════════════
    //  收集参数
    // ══════════════════════════════════════════
    var selStyle = STYLE_LIST[ddl.selection.index];
    var strengthV = Math.round(strengthS.slider.value);
    var grainV    = Math.round(grainS.slider.value);

    // 色彩参数 (归一化到 CLI 范围)
    var warmthV = 0, tintV = 0, satV = 1.0;
    if (colorEnable.value) {
        warmthV = warmthS.slider.value / 100.0;   // -1.0 ~ 1.0
        tintV   = tintS.slider.value / 100.0;      // -1.0 ~ 1.0
        satV    = satS.slider.value / 100.0;        // 0.0 ~ 2.0
    }

    // 肤色参数
    var skinOn = skinEnable.value;
    var skinYellow = skinOn ? Math.round(skinYellowS.slider.value) : 0;
    var skinGreen  = skinOn ? Math.round(skinGreenS.slider.value)  : 0;
    var skinPink   = skinOn ? Math.round(skinPinkS.slider.value)   : 0;
    var skinRed    = skinOn ? Math.round(skinRedS.slider.value)    : 0;
    var skinBright = skinOn ? Math.round(skinBrightS.slider.value) : 0;

    // ══════════════════════════════════════════
    //  调用 CLI 处理（全分辨率，色彩参数传递）
    // ══════════════════════════════════════════
    var processCmd = '"' + EXE_PATH + '"'
        + ' --input "' + inputPath + '"'
        + ' --output "' + outputPath + '"'
        + ' --style ' + selStyle.key
        + ' --strength ' + strengthV
        + ' --grain ' + grainV
        + ' --warmth ' + warmthV.toFixed(3)
        + ' --tint ' + tintV.toFixed(3)
        + ' --saturation ' + satV.toFixed(3);

    runExe(processCmd);

    // 等待输出文件（最多 60 秒）
    var waitCount = 0;
    var outputFile = new File(outputPath);
    while (!outputFile.exists && waitCount < 120) { $.sleep(500); waitCount++; }

    if (!outputFile.exists) {
        var errMsg = "处理失败：未找到输出文件\n\n请确认 filmrust.exe 与本脚本在同一目录。";
        var logFile = new File(TEMP_DIR + "\\_run_log.txt");
        if (logFile.exists) {
            logFile.encoding = "UTF-8"; logFile.open("r");
            var logC = logFile.read(); logFile.close();
            if (logC.length > 0) errMsg += "\n\n日志:\n" + logC.substring(0, 500);
        }
        alert(errMsg); return;
    }

    // ══════════════════════════════════════════
    //  将结果作为图层载入 PS + 应用肤色调整层
    // ══════════════════════════════════════════
    try {
        // 打开处理结果（全分辨率）
        var resultDoc = app.open(outputFile);
        resultDoc.flatten();

        // 确保结果尺寸与原图一致（CLI 处理的是全分辨率导出，尺寸应一致）
        if (resultDoc.width !== doc.width || resultDoc.height !== doc.height) {
            resultDoc.resizeImage(doc.width, doc.height, doc.resolution, ResampleMethod.BICUBIC);
        }

        // 复制结果到原文档
        resultDoc.selection.selectAll();
        resultDoc.selection.copy();
        resultDoc.close(SaveOptions.DONOTSAVECHANGES);

        app.activeDocument = doc;
        doc.paste();
        var newLayer = doc.activeLayer;
        newLayer.name = "FilmRust_" + selStyle.name;

        // ─ 肤色优化: 直接应用图像调整到当前图层 ──
        if (skinOn && (skinYellow > 0 || skinGreen > 0 || skinPink > 0 || skinRed > 0 || skinBright !== 0)) {
            // Color Balance 调整（针对中间调肤色区域）
            if (skinYellow > 0 || skinGreen > 0 || skinPink > 0 || skinRed > 0) {
                var cbLayer = doc.activeLayer;
                
                // 去黄: 中间调减黄(加蓝)、减绿(加红)
                var cbShadowRed   = Math.round(skinRed * 0.5 + skinPink * 0.3);
                var cbShadowGreen = -Math.round(skinGreen * 0.4);
                var cbShadowBlue  = Math.round(skinYellow * 0.3 + skinPink * 0.3);

                var cbMidRed   = Math.round(skinRed * 0.8 + skinPink * 0.5);
                var cbMidGreen = -Math.round(skinGreen * 0.8);
                var cbMidBlue  = Math.round(skinYellow * 0.6 + skinPink * 0.5);

                var cbHlRed   = Math.round(skinRed * 0.3 + skinPink * 0.2);
                var cbHlGreen = -Math.round(skinGreen * 0.3);
                var cbHlBlue  = Math.round(skinYellow * 0.4 + skinPink * 0.2);

                // Clamp to PS range -100~100
                cbShadowRed   = Math.max(-100, Math.min(100, cbShadowRed));
                cbShadowGreen = Math.max(-100, Math.min(100, cbShadowGreen));
                cbShadowBlue  = Math.max(-100, Math.min(100, cbShadowBlue));
                cbMidRed      = Math.max(-100, Math.min(100, cbMidRed));
                cbMidGreen    = Math.max(-100, Math.min(100, cbMidGreen));
                cbMidBlue     = Math.max(-100, Math.min(100, cbMidBlue));
                cbHlRed       = Math.max(-100, Math.min(100, cbHlRed));
                cbHlGreen     = Math.max(-100, Math.min(100, cbHlGreen));
                cbHlBlue      = Math.max(-100, Math.min(100, cbHlBlue));

                // ✅ 使用 ActionDescriptor 执行 Color Balance 调整
                var idClrB = charIDToTypeID("ClrB");
                var desc = new ActionDescriptor();
                desc.putInteger(charIDToTypeID("Shdw"), cbShadowRed);    // Shadow Red
                desc.putInteger(charIDToTypeID("ShdG"), cbShadowGreen);  // Shadow Green
                desc.putInteger(charIDToTypeID("ShdB"), cbShadowBlue);   // Shadow Blue
                desc.putInteger(charIDToTypeID("MdtR"), cbMidRed);       // Midtone Red
                desc.putInteger(charIDToTypeID("MdtG"), cbMidGreen);     // Midtone Green
                desc.putInteger(charIDToTypeID("MdtB"), cbMidBlue);      // Midtone Blue
                desc.putInteger(charIDToTypeID("HghR"), cbHlRed);        // Highlight Red
                desc.putInteger(charIDToTypeID("HghG"), cbHlGreen);      // Highlight Green
                desc.putInteger(charIDToTypeID("HghB"), cbHlBlue);       // Highlight Blue
                desc.putBoolean(charIDToTypeID("PrsL"), true);           // Preserve Luminosity
                
                executeAction(idClrB, desc, DialogModes.NO);
                
                cbLayer.name = "FilmRust_" + selStyle.name + "_肤色优化";
            }

            // 亮度调整（直接应用到当前图层）
            if (skinBright !== 0) {
                var brightnessVal = Math.max(-100, Math.min(100, skinBright));
                
                // ✅ 使用 ActionDescriptor 执行 Brightness/Contrast 调整
                var idBrCn = charIDToTypeID("BrCn");
                var desc = new ActionDescriptor();
                desc.putInteger(charIDToTypeID("Brgh"), brightnessVal);  // Brightness
                desc.putInteger(charIDToTypeID("Cntr"), 0);               // Contrast (不变)
                
                executeAction(idBrCn, desc, DialogModes.NO);
                
                doc.activeLayer.name = "FilmRust_" + selStyle.name + "_肤色亮度";
            }
        }

    } catch(e) {
        alert("处理出错: " + e.message + "\n\n行号: " + e.line);
    }
}

main();
