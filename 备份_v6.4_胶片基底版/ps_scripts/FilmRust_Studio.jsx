// ============================================================
// FilmRust Studio — Photoshop 联动脚本 v6.1
// 架构: filmrust.exe 分析 + 处理 + JSX 回显（静默运行）
// 要求: 把 filmrust.exe 放在与本脚本同一目录
// 特点: 静默后台运行，无黑终端弹窗，无完成弹窗
// 版权: 星TAP 软件 2026  csb603@qq.com
// ============================================================

var SCRIPT_FILE = new File($.fileName);
var SCRIPT_DIR = SCRIPT_FILE.parent.fsName;
var EXE_PATH = SCRIPT_DIR + "\\filmrust.exe";
var TEMP_DIR = "C:\\Temp\\filmrust_temp";

// 57 种胶片预设，按分组排序（常用→人像→风光→黑白→宝丽来→特殊效果）
var STYLE_LIST = [
    // ⭐ 常用
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

    // 人像
    {key:"kodak_portra_160", name:"Kodak Portra 160", desc:"低感人像，更细腻的肤色"},
    {key:"kodak_portra_800", name:"Kodak Portra 800", desc:"弱光人像，温暖颗粒感"},
    {key:"kodak_portra_400_artistic", name:"Kodak Portra 400 Artistic", desc:"艺术版，增强色彩分离"},
    {key:"fujifilm_superia_200", name:"Fujifilm Superia 200", desc:"暖调人像，日系清新"},
    {key:"fujifilm_superia_100", name:"Fujifilm Superia 100", desc:"低感人像，细腻柔和"},
    {key:"agfa_vista_400", name:"Agfa Vista 400", desc:"德系暖调，浓郁色彩人像"},
    {key:"agfa_vista_200", name:"Agfa Vista 200", desc:"德系暖调，日常人像"},
    {key:"agfa_vista_100", name:"Agfa Vista 100", desc:"德系低感人像"},
    {key:"lucky_color_200", name:"Lucky Color 200", desc:"国产乐凯，暖调怀旧"},

    // 风光
    {key:"kodak_ektachrome_100", name:"Kodak Ektachrome 100", desc:"经典反转片，暖调风光"},
    {key:"kodak_ektachrome_100vs", name:"Kodak Ektachrome 100 VS", desc:"超鲜艳反转片，极致色彩"},
    {key:"kodak_kodachrome_64", name:"Kodak Kodachrome 64", desc:"经典柯达克罗姆，暖调浓郁"},
    {key:"kodak_kodachrome_25", name:"Kodak Kodachrome 25", desc:"极致细腻柯达克罗姆"},
    {key:"fujifilm_velvia_50_artistic", name:"Fujifilm Velvia 50 Artistic", desc:"增强版Velvia，极致鲜艳"},
    {key:"fujifilm_astia_100f", name:"Fujifilm Astia 100F", desc:"柔和反转片，淡彩风光"},
    {key:"agfa_optima_200", name:"Agfa Optima 200", desc:"暖调风光，德系反转片"},
    {key:"agfa_precisa_100", name:"Agfa Precisa 100", desc:"暖调反转片，风光人像通用"},

    // 黑白
    {key:"kodak_tri_x_400_artistic", name:"Kodak Tri-X 400 Artistic", desc:"增强版，更强颗粒对比"},
    {key:"kodak_plus_x_125", name:"Kodak Plus-X 125", desc:"细腻黑白，中调丰富"},
    {key:"ilford_hp5_plus_400_artistic", name:"Ilford HP5 Plus 400 Artistic", desc:"增强版HP5，更强颗粒"},
    {key:"ilford_fp4_plus_125", name:"Ilford FP4 Plus 125", desc:"中速黑白，细腻过渡"},
    {key:"ilford_delta_400", name:"Ilford Delta 400 Professional", desc:"现代黑白，颗粒锐利"},
    {key:"ilford_delta_100", name:"Ilford Delta 100 Professional", desc:"超细腻现代黑白"},
    {key:"ilford_pan_f_plus_50", name:"Ilford Pan F Plus 50", desc:"极细腻低感黑白，风光专用"},
    {key:"ilford_xp2_super_400", name:"Ilford XP2 Super 400", desc:"C41工艺黑白，冲印方便"},
    {key:"ilford_sfx_200", name:"Ilford SFX 200", desc:"红外效果黑白，独特质感"},
    {key:"ilford_ortho_plus_80", name:"Ilford Ortho Plus 80", desc:"正色片，高对比反差"},
    {key:"fujifilm_neopan_400", name:"Fujifilm Neopan 400", desc:"日系黑白，细腻灰阶"},
    {key:"fujifilm_neopan_100", name:"Fujifilm Neopan 100", desc:"日系低感黑白"},
    {key:"agfa_apx_400", name:"Agfa APX 400", desc:"经典德系黑白"},
    {key:"agfa_apx_100", name:"Agfa APX 100", desc:"经典德系细腻黑白"},
    {key:"polaroid_bw_667", name:"Polaroid B&W 667", desc:"宝丽来黑白，即时显影质感"},
    {key:"polaroid_55_bw", name:"Polaroid 55 B&W", desc:"宝丽来正负片，极致黑白"},
    {key:"orwo_un54", name:"Orwo UN54", desc:"东德经典黑白，高对比"},
    {key:"orwo_un64", name:"Orwo UN64", desc:"东德低感黑白，细腻"},
    {key:"ricoh_gr_street", name:"Ricoh GR Street Night", desc:"街拍高感黑白，粗颗粒"},
    {key:"agfa_scala_200", name:"Agfa Scala 200", desc:"黑白反转片，高反差"},

    // 宝丽来
    {key:"polaroid_sx70_color", name:"Polaroid SX-70 Color", desc:"经典SX-70，暖调柔和"},
    {key:"polaroid_i_type_color", name:"Polaroid i-Type Color", desc:"现代宝丽来，鲜艳色彩"},
    {key:"polaroid_spectra_color", name:"Polaroid Spectra Color", desc:"宽幅宝丽来，偏冷调"},
    {key:"polaroid_100_color", name:"Polaroid 100 Color", desc:"老式宝丽来100，褪色怀旧"},

    // 特殊效果
    {key:"lomography_lomochrome_purple", name:"Lomography Lomochrome Purple", desc:"紫色幻彩，独特艺术效果"},
    {key:"ferrania_solaris_400", name:"Ferrania Solaris 400", desc:"意式暖调，复古褪色感"},
    {key:"ferrania_solaris_100", name:"Ferrania Solaris 100", desc:"意式低感，暖调柔和"},
];

function ensureTempDir() {
    var dir = new Folder(TEMP_DIR);
    if (!dir.exists) dir.create();
}

function deleteFileSafe(path) {
    var f = new File(path);
    if (f.exists) f.remove();
}

// ========== 静默运行 exe（最小化终端窗口） ==========
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
    var content = f.read();
    f.close();
    return content;
}

function getStyleIndexByKey(key) {
    for (var i = 0; i < STYLE_LIST.length; i++) {
        if (STYLE_LIST[i].key === key) return i;
    }
    return 0;
}

function getStyleDisplayName(i) {
    return STYLE_LIST[i].name + " — " + STYLE_LIST[i].desc;
}

function extractJsonField(text, field) {
    var pattern = '"' + field + '"\\s*:\\s*"([^"]*)"';
    var re = new RegExp(pattern);
    var m = text.match(re);
    if (m) return m[1];
    return "";
}

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
    var infoText = "（分析不可用，使用默认推荐）";

    try {
        var dupDoc = doc.duplicate("filmrust_analyze", true);
        dupDoc.flatten();
        var jpgOpts = new JPEGSaveOptions();
        jpgOpts.quality = 10;
        dupDoc.saveAs(new File(inputPath), jpgOpts, true, Extension.LOWERCASE);
        dupDoc.close(SaveOptions.DONOTSAVECHANGES);

        var cmd = '"' + EXE_PATH + '" --analyze "' + inputPath + '" --json-output "' + analyzePath + '"';
        runExe(cmd);

        $.sleep(500);

        var jsonText = readTextFile(analyzePath);
        if (jsonText.length > 0) {
            var rec = extractJsonField(jsonText, "recommended");
            var recName = extractJsonField(jsonText, "recommended_name");
            var reasonText = extractJsonField(jsonText, "reason");
            if (rec.length > 0) recommendedKey = rec;
            if (reasonText.length > 0) infoText = reasonText;
            else if (recName.length > 0) infoText = "推荐: " + recName;
        }
    } catch (e) {
        infoText = "（分析跳过）";
    }

    var styleIdx = getStyleIndexByKey(recommendedKey);

    var w = new Window("dialog", "FilmRust Studio -- 胶片模拟 v4.2");
    w.orientation = "column";
    w.alignChildren = ["fill", "top"];

    var titlePanel = w.add("panel", undefined, "分析结果与风格选择");
    titlePanel.alignChildren = ["fill", "top"];
    var infoTxt = titlePanel.add("statictext", undefined, infoText);
    infoTxt.graphics.font = ScriptUI.newFont("Arial", "Regular", 11);

    var stylePanel = w.add("panel", undefined, "选择胶片风格（57种）");
    stylePanel.alignChildren = ["fill", "top"];
    var ddl = stylePanel.add("dropdownlist", undefined);
    for (var i = 0; i < STYLE_LIST.length; i++) {
        ddl.add("item", getStyleDisplayName(i));
    }
    ddl.selection = styleIdx;

    var sliderPanel = w.add("panel", undefined, "效果强度");
    sliderPanel.orientation = "column";
    sliderPanel.alignChildren = ["fill", "top"];

    var strengthGroup = sliderPanel.add("group");
    strengthGroup.alignChildren = ["left", "center"];
    strengthGroup.add("statictext", undefined, "强度:");
    var strengthSlider = strengthGroup.add("slider", undefined, 100, 0, 150);
    strengthSlider.preferredSize.width = 200;
    var strengthVal = strengthGroup.add("statictext", undefined, "100%");
    strengthSlider.onChanging = function() {
        strengthVal.text = Math.round(this.value) + "%";
    };

    var grainGroup = sliderPanel.add("group");
    grainGroup.alignChildren = ["left", "center"];
    grainGroup.add("statictext", undefined, "颗粒:");
    var grainSlider = grainGroup.add("slider", undefined, 100, 0, 200);
    grainSlider.preferredSize.width = 200;
    var grainVal = grainGroup.add("statictext", undefined, "100%");
    grainSlider.onChanging = function() {
        grainVal.text = Math.round(this.value) + "%";
    };

    var btnGroup = w.add("group");
    btnGroup.alignChildren = ["right", "center"];
    btnGroup.add("button", undefined, "应用效果", {name:"ok"});
    btnGroup.add("button", undefined, "取消", {name:"cancel"});

    w.add("statictext", undefined, "星TAP 软件 2026  csb603@qq.com").alignment = "right";

    var result = w.show();
    if (result !== 1) return;

    var selectedStyle = STYLE_LIST[ddl.selection.index];
    var strengthValNum = Math.round(strengthSlider.value);
    var grainValNum = Math.round(grainSlider.value);

    // 处理过程无黑终端、无完成弹窗，直接应用
    var processCmd = '"' + EXE_PATH + '" --input "' + inputPath + '" --output "' + outputPath + '" --style ' + selectedStyle.key + ' --strength ' + strengthValNum + ' --grain ' + grainValNum;
    runExe(processCmd);

    var waitCount = 0;
    var outputFile = new File(outputPath);
    while (!outputFile.exists && waitCount < 60) {
        $.sleep(500);
        waitCount++;
    }

    if (!outputFile.exists) {
        var errMsg = "处理失败：未找到输出文件";
        var logPath = TEMP_DIR + "\\_run_log.txt";
        var logFile = new File(logPath);
        if (logFile.exists) {
            logFile.encoding = "UTF-8";
            logFile.open("r");
            var logContent = logFile.read();
            logFile.close();
            if (logContent.length > 0) {
                errMsg += "\n\n错误日志:\n" + logContent.substring(0, 500);
            }
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

        var newLayer = doc.activeLayer;
        newLayer.name = "FilmRust_" + selectedStyle.key;

        // 不弹完成对话框 — 效果已静默应用

    } catch (e) {
        alert("处理出错: " + e.message);
    }
}

main();
