// ============================================================
// FilmRust Studio — Photoshop 综合面板
// 版本: 2.0 (综合版)
// ============================================================
// 用法: File > Scripts > Browse... > 选择本文件
//       或直接拖入 Photoshop 窗口
// ============================================================

app.preferences.rulerUnits = Units.PIXELS;

function FilmRust() {
    this.currentDoc = null;
    this.filmStyles = [
        { id: 'portra400', name: 'Kodak Portra 400', desc: '人像友好 / 肤色柔和', tags: ['人像', '婚礼'] },
        { id: 'portra800', name: 'Kodak Portra 800', desc: '低光人像 / 暖色浓郁', tags: ['夜景', '人像'] },
        { id: 'cinestill', name: 'CineStill 800T', desc: '电影感 / 青蓝色调', tags: ['夜景', '街拍', '电影'] },
        { id: 'velvia', name: 'Fuji Velvia 50', desc: '鲜艳风景 / 绿色浓郁', tags: ['风光', '自然'] },
        { id: 'tri_x', name: 'Kodak Tri-X 400', desc: '黑白纪实 / 高对比', tags: ['黑白', '纪实'] },
        { id: 'ektar', name: 'Kodak Ektar 100', desc: '高饱和 / 色彩锐利', tags: ['街头', '商业'] },
        { id: 'gold', name: 'Kodak Gold 200', desc: '日常快照 / 温暖复古', tags: ['生活', '复古'] },
        { id: 'provia', name: 'Fuji Provia 100F', desc: '通用反转片 / 自然真实', tags: ['日常', '通用'] },
    ];
    
    this.styleParams = {
        portra400: { cb_shadow_red: 8, cb_shadow_green: -4, cb_mid_red: 5, cb_hl_red: 12, cb_hl_blue: -8, halation: 0.6, grain: 0.8, sat: 1.05, warmth: 0.1 },
        portra800: { cb_shadow_red: 12, cb_shadow_green: -6, cb_mid_red: 8, cb_hl_red: 15, cb_hl_blue: -10, halation: 0.8, grain: 1.0, sat: 1.1, warmth: 0.2 },
        cinestill: { cb_shadow_red: -5, cb_shadow_green: 5, cb_mid_red: -3, cb_hl_red: 3, cb_hl_blue: -15, halation: 1.2, grain: 1.2, sat: 0.9, warmth: -0.1 },
        velvia: { cb_shadow_red: 5, cb_shadow_green: 8, cb_mid_red: 3, cb_hl_red: 10, cb_hl_blue: -5, halation: 0.5, grain: 0.5, sat: 1.3, warmth: 0.0 },
        tri_x: { cb_shadow_red: 0, cb_shadow_green: 0, cb_mid_red: 0, cb_hl_red: 0, cb_hl_blue: 0, halation: 0.3, grain: 1.5, sat: 0.0, warmth: 0.0 },
        ektar: { cb_shadow_red: 10, cb_shadow_green: -3, cb_mid_red: 12, cb_hl_red: 18, cb_hl_blue: -12, halation: 0.4, grain: 0.6, sat: 1.4, warmth: 0.15 },
        gold: { cb_shadow_red: 15, cb_shadow_green: -5, cb_mid_red: 10, cb_hl_red: 20, cb_hl_blue: -15, halation: 0.7, grain: 1.0, sat: 1.15, warmth: 0.25 },
        provia: { cb_shadow_red: 3, cb_shadow_green: 2, cb_mid_red: 4, cb_hl_red: 8, cb_hl_blue: -5, halation: 0.4, grain: 0.5, sat: 1.1, warmth: 0.0 },
    };
}

FilmRust.prototype.checkDocument = function() {
    if (app.activeDocument == null) {
        alert("请先打开一张图片！\n\nFilmRust Studio 需要一个活跃的文档。");
        return false;
    }
    this.currentDoc = app.activeDocument;
    return true;
};

FilmRust.prototype.analyzeImage = function() {
    var doc = this.currentDoc;
    var hist = doc.histogram;
    
    var total = 0;
    var bright = 0;
    var dark = 0;
    for (var i = 0; i < 256; i++) {
        total += hist[i];
        if (i < 50) dark += hist[i];
        if (i > 200) bright += hist[i];
    }
    
    var darkRatio = dark / total;
    var brightRatio = bright / total;
    
    var suggestions = [];
    if (darkRatio > 0.3) suggestions.push("【暗部较多】推荐: CineStill 800T / Portra 800");
    if (brightRatio > 0.2) suggestions.push("【高光较多】推荐: Velvia 50 / Ektar 100");
    if (darkRatio < 0.1 && brightRatio < 0.1) suggestions.push("【中间调为主】推荐: Portra 400 / Provia");
    
    return suggestions;
};

FilmRust.prototype.showStyleDialog = function(suggestions) {
    var dlg = new Window("dialog", "🎬 FilmRust Studio", [100, 100, 500, 550]);
    dlg.alignChildren = ["fill", "top"];
    
    var titleGroup = dlg.add("group");
    titleGroup.add("statictext", undefined, "🎬 FilmRust Studio", {fontSize: 16, fontWeight: "bold", align: "center"});
    
    var analyzeGroup = dlg.add("group");
    analyzeGroup.alignChildren = ["left", "top"];
    var analyzeLabel = analyzeGroup.add("statictext", undefined, "📊 图像分析:", {fontSize: 11, fontWeight: "bold"});
    if (suggestions.length > 0) {
        for (var i = 0; i < suggestions.length; i++) {
            analyzeGroup.add("statictext", undefined, "  • " + suggestions[i], {fontSize: 10, color: [0, 0.5, 0]});
        }
    } else {
        analyzeGroup.add("statictext", undefined, "  图像均衡，所有风格适用", {fontSize: 10});
    }
    
    dlg.add("statictext", undefined, "📸 选择胶片风格:", {fontSize: 12, fontWeight: "bold"});
    
    var listGroup = dlg.add("panel");
    listGroup.orientation = "column";
    listGroup.preferredSize.width = 380;
    listGroup.preferredSize.height = 280;
    listGroup.alignChildren = ["fill", "top"];
    
    var radioButtons = [];
    for (var i = 0; i < this.filmStyles.length; i++) {
        var style = this.filmStyles[i];
        var row = listGroup.add("group");
        row.alignChildren = ["left", "center"];
        
        var rb = row.add("radiobutton", undefined, style.name, {name: style.id});
        rb.value = (i === 0);
        radioButtons.push(rb);
        
        row.add("statictext", undefined, " — " + style.desc, {fontSize: 10});
        
        var tagsGroup = row.add("group");
        tagsGroup.alignChildren = ["left", "center"];
        for (var j = 0; j < style.tags.length; j++) {
            tagsGroup.add("statictext", undefined, "[" + style.tags[j] + "]", {fontSize: 9, color: [0.5, 0.5, 0.5]});
        }
    }
    
    var controlGroup = dlg.add("panel");
    controlGroup.alignChildren = ["fill", "top"];
    controlGroup.preferredSize.height = 60;
    
    var intensityRow = controlGroup.add("group");
    intensityRow.add("statictext", undefined, "效果强度:", {fontSize: 11});
    var intensitySlider = intensityRow.add("slider", [0, 0, 200, 20], 100, 0, 200, 1);
    intensitySlider.tooltip = "100% = 标准效果";
    
    var btnGroup = dlg.add("group");
    btnGroup.alignChildren = ["center", "center"];
    btnGroup.add("button", undefined, "✅ 应用效果", {name: "ok"});
    btnGroup.add("button", undefined, "❌ 取消", {name: "cancel"});
    
    var result = dlg.show();
    
    if (result === 1) {
        var selectedStyle = null;
        for (var i = 0; i < radioButtons.length; i++) {
            if (radioButtons[i].value) {
                selectedStyle = this.filmStyles[i];
                break;
            }
        }
        
        return {
            style: selectedStyle,
            intensity: intensitySlider.value / 100
        };
    }
    
    return null;
};

FilmRust.prototype.createAdjustmentLayer = function(doc, kind, name) {
    try {
        var layer = doc.artLayers.add();
        layer.name = name;
        layer.kind = kind;
        return layer;
    } catch (e) {
        doc.selection.selectAll();
        var layer = doc.artLayers.add();
        layer.name = name;
        layer.kind = kind;
        doc.selection.deselect();
        return layer;
    }
};

FilmRust.prototype.applyFilmEffect = function(styleId, intensity) {
    var doc = this.currentDoc;
    var params = this.styleParams[styleId];
    if (!params) {
        alert("未知风格: " + styleId);
        return;
    }
    
    var scale = intensity;
    
    doc.suspendHistory("FilmRust " + styleId, function() {
        var groupName = "FilmRust_" + styleId;
        var existingGroup = doc.layerSets.getByName(groupName);
        if (existingGroup) {
            existingGroup.remove();
        }
        var filmGroup = doc.layerSets.add();
        filmGroup.name = groupName;
        
        // 1. Color Balance
        var cbLayer = doc.artLayers.add();
        cbLayer.name = "ColorBalance_Film";
        cbLayer.kind = LayerKind.COLORBALANCE;
        
        cbLayer.adjustment.shadows.red = Math.round(params.cb_shadow_red * scale);
        cbLayer.adjustment.shadows.green = Math.round(params.cb_shadow_green * scale);
        cbLayer.adjustment.shadows.blue = 0;
        cbLayer.adjustment.shadows.preserveLuminosity = true;
        
        cbLayer.adjustment.midtones.red = Math.round(params.cb_mid_red * scale);
        cbLayer.adjustment.midtones.green = 0;
        cbLayer.adjustment.midtones.blue = 0;
        cbLayer.adjustment.midtones.preserveLuminosity = true;
        
        cbLayer.adjustment.highlights.red = Math.round(params.cb_hl_red * scale);
        cbLayer.adjustment.highlights.green = 0;
        cbLayer.adjustment.highlights.blue = Math.round(params.cb_hl_blue * scale);
        cbLayer.adjustment.highlights.preserveLuminosity = true;
        
        cbLayer.moveToEnd(filmGroup);
        
        // 2. Halation
        if (params.halation * scale > 0.3) {
            var haloLayer = doc.activeLayer.duplicate();
            haloLayer.name = "Halation_Glow";
            haloLayer.blendMode = BlendMode.SCREEN;
            haloLayer.opacity = Math.round(params.halation * scale * 25);
            haloLayer.applyGaussianBlur(Math.round(params.halation * scale * 3));
            haloLayer.moveToEnd(filmGroup);
        }
        
        // 3. Grain
        if (params.grain * scale > 0.1) {
            var grainLayer = doc.artLayers.add();
            grainLayer.name = "FilmGrain";
            grainLayer.blendMode = BlendMode.OVERLAY;
            grainLayer.opacity = Math.round(params.grain * scale * 35);
            
            var gray = new SolidColor();
            gray.rgb.red = 128;
            gray.rgb.green = 128;
            gray.rgb.blue = 128;
            
            doc.selection.selectAll();
            doc.selection.fill(gray);
            doc.selection.deselect();
            
            grainLayer.applyAddNoise(Math.round(params.grain * scale * 8), NoiseDistribution.GAUSSIAN, true);
            grainLayer.applyGaussianBlur(0.5);
            grainLayer.moveToEnd(filmGroup);
        }
        
        // 4. Saturation
        if (params.sat !== 1.0) {
            var satLayer = doc.artLayers.add();
            satLayer.name = "Saturation";
            satLayer.kind = LayerKind.HUESATURATION;
            var satLevel = Math.round((params.sat - 1.0) * 50 * scale);
            satLayer.adjustment.adjustSaturation(0, satLevel);
            satLayer.moveToEnd(filmGroup);
        }
        
        // 5. Curves (S-Curve)
        var curveLayer = doc.artLayers.add();
        curveLayer.name = "FilmCurve";
        curveLayer.kind = LayerKind.CURVES;
        
        try {
            var curvePoints = curveLayer.adjustment;
            if (curvePoints) {
                curvePoints.delete();
                curvePoints.addPoint(0, 5);
                curvePoints.addPoint(64, 58);
                curvePoints.addPoint(128, 128);
                curvePoints.addPoint(192, 198);
                curvePoints.addPoint(255, 250);
            }
        } catch(e) {}
        
        curveLayer.moveToEnd(filmGroup);
    });
};

FilmRust.prototype.run = function() {
    if (!this.checkDocument()) return;
    
    var suggestions = this.analyzeImage();
    
    var choice = this.showStyleDialog(suggestions);
    if (!choice) return;
    
    this.applyFilmEffect(choice.style.id, choice.intensity);
    
    alert("✅ 胶片效果应用完成！\n\n预设: " + choice.style.name + "\n强度: " + Math.round(choice.intensity * 100) + "%\n\n提示：在图层面板中调整各图层的不透明度，可以微调效果。");
};

var plugin = new FilmRust();
plugin.run();
