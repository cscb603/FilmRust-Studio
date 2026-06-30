// ============================================================
// FilmRust Studio Pro — Photoshop 专业版 v5.9
// 版权: 星TAP 软件 2026  csb603@qq.com
// ============================================================

var SCRIPT_FILE = new File($.fileName);
var SCRIPT_DIR = SCRIPT_FILE.parent.fsName;
var EXE_PATH = SCRIPT_DIR + "\\filmrust.exe";
var TEMP_DIR = "C:\\Temp\\filmrust_temp";

var STYLE_LIST = [
    {key:"kodak_portra_400", name:"Kodak Portra 400", desc:"人像首选，柔和肤色", tone:"暖色调"},
    {key:"kodak_gold_200", name:"Kodak Gold 200", desc:"日常阳光感，90年代温暖回忆", tone:"暖色调"},
    {key:"kodak_ektar_100", name:"Kodak Ektar 100", desc:"超细腻颜色，风光利器", tone:"高饱和·暖"},
    {key:"kodak_tri_x_400", name:"Kodak Tri-X 400", desc:"传奇黑白，高对比颗粒质感", tone:"黑白·高反差"},
    {key:"fujifilm_superia_400", name:"Fujifilm Superia 400", desc:"通用彩色，自然饱和日用", tone:"冷调·青绿"},
    {key:"fujifilm_provia_100f", name:"Fujifilm Provia 100F", desc:"专业反转片，中性真实色彩", tone:"正片·中性"},
    {key:"fujifilm_velvia_50", name:"Fujifilm Velvia 50", desc:"风光首席，超高饱和鲜艳", tone:"高饱和·绿"},
    {key:"cinestill_800t", name:"CineStill 800T", desc:"夜景/钨丝灯，电影感青橙调", tone:"冷调·蓝钨丝"},
    {key:"cinestill_50d", name:"CineStill 50D", desc:"日光型电影胶片，电影感街拍", tone:"冷调·柔和"},
    {key:"ilford_hp5_plus_400", name:"Ilford HP5 Plus 400", desc:"黑白经典，高对比颗粒感", tone:"黑白·高反差"},
    {key:"standard_daylight", name:"Standard Daylight", desc:"中性基准，去胶片化原色", tone:"中性"},
    {key:"lomography_color_chrome", name:"Lomography Color Chrome", desc:"LOMO艺术，高对比偏色", tone:"创意·Lomo"},
    {key:"polaroid_600_color", name:"Polaroid 600 Color", desc:"宝丽来即时感，怀旧偏暖", tone:"暖调·拍立得"},
    {key:"kodak_portra_160", name:"Kodak Portra 160", desc:"低感人像，更细腻的肤色", tone:"暖色调"},
    {key:"kodak_portra_800", name:"Kodak Portra 800", desc:"弱光人像，温暖颗粒感", tone:"暖色调"},
    {key:"kodak_portra_400_artistic", name:"Kodak Portra 400 Artistic", desc:"艺术版，增强色彩分离", tone:"暖色调"},
    {key:"fujifilm_superia_200", name:"Fujifilm Superia 200", desc:"暖调人像，日系清新", tone:"冷调·青绿"},
    {key:"fujifilm_superia_100", name:"Fujifilm Superia 100", desc:"低感人像，细腻柔和", tone:"冷调·青绿"},
    {key:"agfa_vista_400", name:"Agfa Vista 400", desc:"德系暖调，浓郁色彩人像", tone:"暖调·复古"},
    {key:"agfa_vista_200", name:"Agfa Vista 200", desc:"德系暖调，日常人像", tone:"暖调·复古"},
    {key:"agfa_vista_100", name:"Agfa Vista 100", desc:"德系低感人像", tone:"暖调·复古"},
    {key:"lucky_color_200", name:"Lucky Color 200", desc:"国产乐凯，暖调怀旧", tone:"暖色调"},
    {key:"kodak_ektachrome_100", name:"Kodak Ektachrome 100", desc:"经典反转片，暖调风光", tone:"正片·冷"},
    {key:"kodak_ektachrome_100vs", name:"Kodak Ektachrome 100 VS", desc:"超鲜艳反转片，极致色彩", tone:"正片·冷"},
    {key:"kodak_kodachrome_64", name:"Kodak Kodachrome 64", desc:"经典柯达克罗姆，暖调浓郁", tone:"暖色调"},
    {key:"kodak_kodachrome_25", name:"Kodak Kodachrome 25", desc:"极致细腻柯达克罗姆", tone:"暖色调"},
    {key:"fujifilm_velvia_50_artistic", name:"Fujifilm Velvia 50 Artistic", desc:"增强版Velvia，极致鲜艳", tone:"高饱和·绿"},
    {key:"fujifilm_astia_100f", name:"Fujifilm Astia 100F", desc:"柔和反转片，淡彩风光", tone:"正片·中性"},
    {key:"agfa_optima_200", name:"Agfa Optima 200", desc:"暖调风光，德系反转片", tone:"暖色调"},
    {key:"agfa_precisa_100", name:"Agfa Precisa 100", desc:"暖调反转片，风光人像通用", tone:"暖色调"},
    {key:"kodak_tri_x_400_artistic", name:"Kodak Tri-X 400 Artistic", desc:"增强版，更强颗粒对比", tone:"黑白·高反差"},
    {key:"kodak_plus_x_125", name:"Kodak Plus-X 125", desc:"细腻黑白，中调丰富", tone:"黑白·细颗粒"},
    {key:"ilford_hp5_plus_400_artistic", name:"Ilford HP5 Plus 400 Artistic", desc:"增强版HP5，更强颗粒", tone:"黑白·高反差"},
    {key:"ilford_fp4_plus_125", name:"Ilford FP4 Plus 125", desc:"中速黑白，细腻过渡", tone:"黑白·细颗粒"},
    {key:"ilford_delta_400", name:"Ilford Delta 400 Professional", desc:"现代黑白，颗粒锐利", tone:"黑白·细颗粒"},
    {key:"ilford_delta_100", name:"Ilford Delta 100 Professional", desc:"超细腻现代黑白", tone:"黑白·细颗粒"},
    {key:"ilford_pan_f_plus_50", name:"Ilford Pan F Plus 50", desc:"极细腻低感黑白，风光专用", tone:"黑白·细颗粒"},
    {key:"ilford_xp2_super_400", name:"Ilford XP2 Super 400", desc:"C41工艺黑白，冲印方便", tone:"黑白·细颗粒"},
    {key:"ilford_sfx_200", name:"Ilford SFX 200", desc:"红外效果黑白，独特质感", tone:"黑白·细颗粒"},
    {key:"ilford_ortho_plus_80", name:"Ilford Ortho Plus 80", desc:"正色片，高对比反差", tone:"黑白·高反差"},
    {key:"fujifilm_neopan_400", name:"Fujifilm Neopan 400", desc:"日系黑白，细腻灰阶", tone:"黑白·细颗粒"},
    {key:"fujifilm_neopan_100", name:"Fujifilm Neopan 100", desc:"日系低感黑白", tone:"黑白·细颗粒"},
    {key:"agfa_apx_400", name:"Agfa APX 400", desc:"经典德系黑白", tone:"黑白·高反差"},
    {key:"agfa_apx_100", name:"Agfa APX 100", desc:"经典德系细腻黑白", tone:"黑白·细颗粒"},
    {key:"polaroid_bw_667", name:"Polaroid B&W 667", desc:"宝丽来黑白，即时显影质感", tone:"黑白·细颗粒"},
    {key:"polaroid_55_bw", name:"Polaroid 55 B&W", desc:"宝丽来正负片，极致黑白", tone:"黑白·细颗粒"},
    {key:"orwo_un54", name:"Orwo UN54", desc:"东德经典黑白，高对比", tone:"黑白·高反差"},
    {key:"orwo_un64", name:"Orwo UN64", desc:"东德低感黑白，细腻", tone:"黑白·细颗粒"},
    {key:"ricoh_gr_street", name:"Ricoh GR Street Night", desc:"街拍高感黑白，粗颗粒", tone:"黑白·高反差"},
    {key:"agfa_scala_200", name:"Agfa Scala 200", desc:"黑白反转片，高反差", tone:"黑白·高反差"},
    {key:"polaroid_sx70_color", name:"Polaroid SX-70 Color", desc:"经典SX-70，暖调柔和", tone:"暖调·拍立得"},
    {key:"polaroid_i_type_color", name:"Polaroid i-Type Color", desc:"现代宝丽来，鲜艳色彩", tone:"暖调·拍立得"},
    {key:"polaroid_spectra_color", name:"Polaroid Spectra Color", desc:"宽幅宝丽来，偏冷调", tone:"暖调·拍立得"},
    {key:"polaroid_100_color", name:"Polaroid 100 Color", desc:"老式宝丽来100，褪色怀旧", tone:"暖调·拍立得"},
    {key:"lomography_lomochrome_purple", name:"Lomography Lomochrome Purple", desc:"紫色幻彩，独特艺术效果", tone:"创意·Lomo"},
    {key:"ferrania_solaris_400", name:"Ferrania Solaris 400", desc:"意式暖调，复古褪色感", tone:"暖调·复古"},
    {key:"ferrania_solaris_100", name:"Ferrania Solaris 100", desc:"意式低感，暖调柔和", tone:"暖调·复古"},
];

function ensureTempDir() {
    var d = new Folder(TEMP_DIR);
    if (!d.exists) d.create();
}

function deleteFileSafe(path) {
    var f = new File(path);
    if (f.exists) f.remove();
}

function runExe(cmd) {
    var batPath = TEMP_DIR + "\\_run_filmrust.bat";
    var batFile = new File(batPath);
    batFile.encoding = "UTF-8";
    batFile.open("w");
    batFile.writeln("@echo off");
    batFile.writeln("title FilmRust");
    batFile.writeln("cd /d \"" + SCRIPT_DIR + "\"");
    batFile.writeln("chcp 65001 >nul 2>nul");
    batFile.writeln(cmd + " >\"" + TEMP_DIR + "\\_run_log.txt\" 2>&1");
    batFile.close();
    app.system(batPath);
    $.sleep(500);
}

function readTextFile(path) {
    var f = new File(path);
    if (!f.exists) return "";
    f.encoding = "UTF-8";
    f.open("r");
    var c = f.read();
    f.close();
    return c;
}

function getStyleIndexByKey(key) {
    for (var i = 0; i < STYLE_LIST.length; i++) {
        if (STYLE_LIST[i].key === key) return i;
    }
    return 0;
}

function extractJsonField(text, field) {
    var pattern = '"' + field + '"\\s*:\\s*"([^"]*)"';
    var re = new RegExp(pattern);
    var m = text.match(re);
    if (m) return m[1];
    return "";
}

function waitForFile(path, timeoutMs) {
    var waited = 0;
    var interval = 200;
    var maxLoops = Math.floor(timeoutMs / interval);
    for (var i = 0; i < maxLoops; i++) {
        if (new File(path).exists) return true;
        $.sleep(interval);
        waited += interval;
    }
    return new File(path).exists;
}

// ============================================================
// 切换面板可见：隐藏/显示对应分组控件
// ============================================================
function togglePanelVisibility(chk, panelGroups) {
    var vis = chk.value;
    for (var i = 0; i < panelGroups.length; i++) {
        panelGroups[i].visible = vis;
    }
}

// ============================================================
function main() {
    if (app.documents.length < 1) {
        alert("请先打开一张图片！");
        return;
    }

    var doc = app.activeDocument;
    ensureTempDir();

    var inputPath = TEMP_DIR + "\\_input.jpg";
    var outputPath = TEMP_DIR + "\\_output.jpg";
    var analyzePath = TEMP_DIR + "\\_analyze.json";

    deleteFileSafe(inputPath);
    deleteFileSafe(outputPath);
    deleteFileSafe(analyzePath);

    var recommendedKey = "kodak_portra_400";
    var infoText = "分析尚未完成，将使用默认推荐";
    var analysisTags = "";

    try {
        var dupDoc = doc.duplicate("filmrust_analyze", true);
        dupDoc.flatten();
        var jpgOpts = new JPEGSaveOptions();
        jpgOpts.quality = 10;
        dupDoc.saveAs(new File(inputPath), jpgOpts, true, Extension.LOWERCASE);
        dupDoc.close(SaveOptions.DONOTSAVECHANGES);

        var cmd = '"' + EXE_PATH + '" --analyze "' + inputPath + '" --json-output "' + analyzePath + '"';
        runExe(cmd);
        waitForFile(analyzePath, 10000);

        var jsonText = readTextFile(analyzePath);
        if (jsonText.length > 0) {
            var rec = extractJsonField(jsonText, "recommended");
            var recName = extractJsonField(jsonText, "recommended_name");
            var reasonText = extractJsonField(jsonText, "reason");
            if (rec.length > 0) recommendedKey = rec;
            if (reasonText.length > 0) infoText = reasonText;
            else if (recName.length > 0) infoText = "推荐 " + recName;

            var tagsMatch = jsonText.match(/"tags"\s*:\s*\[([^\]]*)\]/);
            if (tagsMatch) {
                var rawTags = tagsMatch[1].replace(/"/g, "");
                if (rawTags.length > 0) analysisTags = "图片特征: " + rawTags;
            }
        }
    } catch (e) {
        infoText = "分析跳过 — 将使用默认推荐";
    }

    var styleIdx = getStyleIndexByKey(recommendedKey);

    // ====== UI ======
    var w = new Window("dialog", "FilmRust Studio Pro v6.1");
    w.orientation = "column";
    w.alignChildren = ["fill", "top"];
    w.preferredSize.width = 400;

    // ── 1. 分析结果 ──
    var p1 = w.add("panel", undefined, "智能推荐");
    p1.alignChildren = ["fill", "top"];
    var infoTxt = p1.add("statictext", undefined, infoText);
    infoTxt.graphics.font = ScriptUI.newFont("Arial", "Bold", 11);
    if (analysisTags.length > 0) {
        var tagsTxt = p1.add("statictext", undefined, analysisTags);
        tagsTxt.graphics.font = ScriptUI.newFont("Arial", "Regular", 10);
    }

    // ── 2. 胶片选择 ──
    var p2 = w.add("panel", undefined, "选择胶片 · 57种");
    p2.alignChildren = ["fill", "top"];
    var ddl = p2.add("dropdownlist", undefined);
    for (var i = 0; i < STYLE_LIST.length; i++) {
        ddl.add("item", STYLE_LIST[i].name + "  ← " + STYLE_LIST[i].desc);
    }
    ddl.selection = styleIdx;
    ddl.preferredSize.width = 370;

    var toneTxt = p2.add("statictext", undefined, STYLE_LIST[styleIdx].tone + "  |  " + STYLE_LIST[styleIdx].desc);
    toneTxt.graphics.font = ScriptUI.newFont("Arial", "Bold", 11);

    ddl.onChange = function() {
        var sel = STYLE_LIST[this.selection.index];
        toneTxt.text = sel.tone + "  |  " + sel.desc;
        tipLine2.text = "选中风格: " + sel.tone + " — " + sel.desc;
    };

    // ── 3. 基础调节 ──
    var p3 = w.add("panel", undefined, "基础调节");
    p3.alignChildren = ["fill", "top"];

    var g1 = p3.add("group");
    g1.alignChildren = ["left", "center"];
    g1.add("statictext", undefined, "强度  ");
    var strengthSlider = g1.add("slider", undefined, 100, 0, 150);
    strengthSlider.preferredSize.width = 190;
    var strengthVal = g1.add("statictext", undefined, "100%");
    strengthVal.preferredSize.width = 40;
    strengthSlider.onChanging = function() { strengthVal.text = Math.round(this.value) + "%"; };

    var g2 = p3.add("group");
    g2.alignChildren = ["left", "center"];
    g2.add("statictext", undefined, "颗粒  ");
    var grainSlider = g2.add("slider", undefined, 100, 0, 200);
    grainSlider.preferredSize.width = 190;
    var grainVal = g2.add("statictext", undefined, "100%");
    grainVal.preferredSize.width = 40;
    grainSlider.onChanging = function() { grainVal.text = Math.round(this.value) + "%"; };

    // ── 4. 高级调节（可折叠）──
    var advCheck = w.add("checkbox", undefined, "▸ 高级调节（色温 / 色调 / 饱和度）");
    advCheck.value = false;

    var p4 = w.add("panel", undefined, "高级调节");
    p4.alignChildren = ["fill", "top"];
    p4.visible = false;

    var g3 = p4.add("group");
    g3.alignChildren = ["left", "center"];
    g3.add("statictext", undefined, "色温  ");
    var warmthSlider = g3.add("slider", undefined, 0, -100, 100);
    warmthSlider.preferredSize.width = 190;
    var warmthVal = g3.add("statictext", undefined, " 0.0");
    warmthVal.preferredSize.width = 40;
    warmthSlider.onChanging = function() { warmthVal.text = " " + (this.value / 100).toFixed(1); };

    var g4 = p4.add("group");
    g4.alignChildren = ["left", "center"];
    g4.add("statictext", undefined, "色调  ");
    var tintSlider = g4.add("slider", undefined, 0, -100, 100);
    tintSlider.preferredSize.width = 190;
    var tintVal = g4.add("statictext", undefined, " 0.0");
    tintVal.preferredSize.width = 40;
    tintSlider.onChanging = function() { tintVal.text = " " + (this.value / 100).toFixed(1); };

    var g5 = p4.add("group");
    g5.alignChildren = ["left", "center"];
    g5.add("statictext", undefined, "饱和  ");
    var satSlider = g5.add("slider", undefined, 100, 0, 200);
    satSlider.preferredSize.width = 190;
    var satVal = g5.add("statictext", undefined, "100%");
    satVal.preferredSize.width = 40;
    satSlider.onChanging = function() { satVal.text = Math.round(this.value) + "%"; };

    var advGroups = [g3, g4, g5];
    advCheck.onClick = function() {
        p4.visible = this.value;
        if (this.value) { this.text = "▾ 高级调节（色温 / 色调 / 饱和度）"; }
        else             { this.text = "▸ 高级调节（色温 / 色调 / 饱和度）"; }
        w.layout.layout(true);
    };

    // ── 5. 使用提示 ──
    var p5 = w.add("panel", undefined, "使用提示");
    p5.alignChildren = ["left", "top"];
    p5.add("statictext", undefined, "结果直接贴入当前文档，不建组，用完即走");
    var tipLine2 = p5.add("statictext", undefined, "选中风格: " + STYLE_LIST[styleIdx].tone + " — " + STYLE_LIST[styleIdx].desc);

    // ── 按钮 ──
    var btnGroup = w.add("group");
    btnGroup.alignChildren = ["right", "center"];
    btnGroup.add("button", undefined, "应用", {name:"ok"});
    btnGroup.add("button", undefined, "取消", {name:"cancel"});

    w.add("statictext", undefined, "星TAP 软件 2026").alignment = "right";

    w.center();
    var result = w.show();
    if (result !== 1) return;

    var selectedStyle = STYLE_LIST[ddl.selection.index];
    var strengthValNum = Math.round(strengthSlider.value);
    var grainValNum = Math.round(grainSlider.value);
    var warmthValNum = (warmthSlider.value / 100);
    var tintValNum = (tintSlider.value / 100);
    var satValNum = (satSlider.value / 100);

    // ====== filmr 处理 ======
    var processCmd = '"' + EXE_PATH + '" --input "' + inputPath + '" --output "' + outputPath +
        '" --style ' + selectedStyle.key + ' --strength ' + strengthValNum + ' --grain ' + grainValNum +
        ' --warmth=' + warmthValNum.toFixed(2) + ' --tint=' + tintValNum.toFixed(2) + ' --saturation=' + satValNum.toFixed(2);
    runExe(processCmd);
    waitForFile(outputPath, 30000);

    var outputFile = new File(outputPath);
    if (!outputFile.exists) {
        var logPath = TEMP_DIR + "\\_run_log.txt";
        var logFile = new File(logPath);
        var errMsg = "处理失败：未生成结果文件\n请确认 filmrust.exe 在同目录下";
        if (logFile.exists) {
            logFile.encoding = "UTF-8";
            logFile.open("r");
            var lc = logFile.read();
            logFile.close();
            if (lc.length > 0) errMsg += "\n\n" + lc.substring(0, 400);
        }
        alert(errMsg);
        return;
    }

    try {
        var newDoc = app.open(outputFile);
        newDoc.flatten();
        newDoc.selection.selectAll();
        newDoc.selection.copy();
        newDoc.close(SaveOptions.DONOTSAVECHANGES);

        app.activeDocument = doc;
        doc.paste();
        doc.activeLayer.name = selectedStyle.name;

    } catch (e) {
        alert("导入出错: " + e.message);
    }
}

main();
