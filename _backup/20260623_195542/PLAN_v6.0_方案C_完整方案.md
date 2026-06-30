# FilmRust Studio v6.0 — 方案C 完整实施计划

## 调研结论总览

### 一、预设审计结论：60 → 约48（砍12个冗余 + 新增6个热门）

#### 砍掉的（12个，保留无意义）

| 预设 | 理由 |
|------|------|
| Kodak Portra 400 Artistic | 不存在真实对应胶卷，"艺术版"无参考样片 |
| Kodak Tri-X 400 Artistic | 同上 |
| Ilford HP5 Plus 400 Artistic | 同上 |
| Fujifilm Velvia 50 Artistic | 同上 |
| Kodak Plus-X 125 | 已停产数十年，无现役用户讨论 |
| Ilford FP4 Plus 125 | 使用率极低，网络讨论几乎为零 |
| Ilford SFX 200 | 红外胶片，极小众，真实用户极少 |
| Ilford Ortho Plus 80 | 正色片，极小众 |
| Polaroid B&W 667 | 无真实用户讨论 |
| Polaroid 55 B&W | 已停产数十年 |
| Polaroid Spectra Color | 宽幅拍立得，无讨论 |
| Polaroid 100 Color | 无真实用户讨论 |

**以上12个保留代码但移出主列表，折叠到"小众/特殊"分组**，避免主选择列表过长。

#### 新增的（6个，真实胶片圈热门但缺失）

| 预设 | 理由 |
|------|------|
| Kodak ColorPlus 200 | 柯达最便宜入门卷，大量讨论，"新手首选" |
| Fujicolor C200 | 富士最便宜入门卷，与ColorPlus对标 |
| Kodak Pro Image 100 | 高性价比日光负片，国内流行 |
| Fujifilm Acros 100 II | 富士经典黑白极细颗粒，有独特高反差灰调 |
| Ilford Delta 3200 | 超高感黑白，暗光/音乐会场景经典 |
| Lomography Color Negative 400 | Lomo圈最主流彩色负片 |

#### 黑白合并（17→10）

保留最具代表性的：
1. Kodak Tri-X 400 — 高对比粗颗粒必选
2. Ilford HP5 Plus 400 — 经典通用黑白
3. Ilford Delta 100 — 超细腻现代黑白
4. Ilford Delta 400 — 现代细腻黑白
5. Fujifilm Neopan 400 — 日系黑白
6. Agfa APX 400 — 德系黑白
7. Kodak T-Max 400 — 现代锐利黑白（已有，保有）
8. Ilford XP2 Super 400 — C41工艺便利
9. Fujifilm Acros 100 II — 新增，高端黑白
10. Ilford Delta 3200 — 新增，暗光专用

其余（Pan F+, Orwo UN54/UN64, Agfa APX 100, Neopan 100, Scala, Ricoh GR）→ 折叠到"黑白特殊"

---

### 二、方案C 新管线架构

```
现有管线（完全保留，不动）:
  filmr 物理模拟 → Color(色温/色调/饱和度) → Curves(对比度/高光/阴影)

新增后处理（可单独开关，全可调）:
  → SkinHSL(肤色优化) → SplitTone(色调分离) → Sharp(输出锐化)
```

每个新层在 GUI 中都是一个独立 Layer，以 checkbox 控制启用/禁用，以 slider 控制强度。

#### 新增 LayerType 定义

```rust
enum LayerType {
    // 现有：
    FilmBase { stock_id, strength, grain, auto_levels },
    Color { warmth, tint, saturation },
    Curves { contrast, highlights, shadows },
    
    // === 新增 ===
    
    /// 肤色优化（HSL 通道级）
    SkinHsl {
        enabled: bool,          // 全局开关
        orange_hue: f32,        // -30~+30, 默认-5（橙偏红，去黄气）
        orange_saturation: f32, // -100~+100%, 默认-25%（降低橙色饱和，显白）
        orange_luminance: f32,  // -100~+100%, 默认+15%（提高橙色明度，通透）
        yellow_saturation: f32, // -100~+100%, 默认-15%（降低黄色饱和，去土黄）
        red_luminance: f32,     // -100~+100%, 默认-20%（压暗红色，嘴唇不过曝）
        strength: f32,          // 0~100%, 效果叠加量
    },
    
    /// 色调分离（Split Toning）
    SplitTone {
        enabled: bool,
        highlight_hue: f32,     // 0~360
        highlight_saturation: f32, // 0~100%
        shadow_hue: f32,        // 0~360
        shadow_saturation: f32, // 0~100%
        balance: f32,           // -100~+100, 偏向高光/阴影
        strength: f32,          // 0~100%, 整体强度
    },
    
    /// 输出锐化
    Sharp {
        enabled: bool,
        amount: f32,            // 0~100, 默认根据分辨率自适应
        radius: f32,            // 0.5~3.0 px, 默认1.0
    },
}
```

#### 各胶卷默认配方

##### SkinHSL 默认值

| 胶卷 | orange_hue | orange_sat | orange_lum | yellow_sat | red_lum | 理由 |
|------|-----------|-----------|-----------|-----------|--------|------|
| Portra 400 | -5° | -25% | +18% | -20% | -15% | 低反差人像卷,肤色需要通透 |
| Portra 160 | -5° | -20% | +15% | -15% | -10% | 更细腻，微调即可 |
| Portra 800 | -5° | -20% | +15% | -15% | -15% | 同上 |
| Ultramax 400 | -5° | -15% | +12% | -10% | -15% | 高饱和消费卷,适量去黄 |
| Gold 200 | -5° | -15% | +10% | -10% | -10% | 保留暖调 |
| Ektar 100 | -3° | -10% | +8% | -8% | -20% | 风光高饱和,保留色彩浓度 |
| Pro 400H | -3° | -30% | +22% | -25% | -10% | 日系冷白皮,奶白透亮 |
| Natura 1600 | -5° | -15% | +10% | -10% | -15% | 高感暖调 |
| Superia 400 | -5° | -20% | +15% | -15% | -10% | 日系清新 |
| CineStill 800T | -3° | -10% | +8% | -5% | -15% | 电影卷,保留偏色特色 |
| 通用彩色负片 | -5° | -20% | +12% | -15% | -15% | 安全通用 |

##### SplitTone 默认值

| 胶卷 | 高光色相 | 高光饱和 | 阴影色相 | 阴影饱和 | balance | 色调特征 |
|------|---------|---------|---------|---------|---------|---------|
| Portra 400 | 41°(橙) | 13% | 190°(青绿) | 22% | 0 | 经典暖橙高光+青绿阴影 |
| Pro 400H | 52°(黄) | 19% | 195°(青) | 15% | -10 | 高光暖粉+阴影冷蓝（日系） |
| Ultramax 400 | 35°(暖橙) | 18% | 180°(青) | 25% | +5 | 消费级浓郁对比 |
| Gold 200 | 45°(黄橙) | 15% | 190°(青绿) | 20% | +5 | 90年代温暖 |
| Ektar 100 | 30°(橙红) | 8% | 210°(蓝青) | 15% | +10 | 风光冷调高饱和 |
| Natura 1600 | 35°(暖橙) | 20% | 170°(青绿) | 25% | 0 | 月光暖调+青绿阴影 |
| CineStill 800T | 25°(暖橙) | 10% | 220°(蓝) | 30% | -15 | 钨丝灯冷蓝阴影 |
| 黑白全部 | 0 | 0% | 0 | 0% | 0 | 不改色调 |
| 通用彩色负片 | 41° | 12% | 190° | 20% | 0 | 通用胶片氛围 |

##### Sharp 默认值

自动模式：
```
如果 图片长边 < 1200px: amount=15, radius=0.8 （低分辨率，弱锐化）
如果 1200 <= 长边 < 2400px: amount=25, radius=1.0 （中等分辨率）
如果 长边 >= 2400px: amount=35, radius=1.2 （高分辨率，可较强锐化）
```

全部可手动覆盖。

---

### 三、代码变更清单

#### 阶段一：基础设施

| 文件 | 变更 |
|------|------|
| `src/layers.rs` | LayerType 新增 `SkinHsl`, `SplitTone`, `Sharp` 3个变体+各自字段 |
| `src/layers.rs` | `LayerStack::composite()` 新增在 Color→Curves 之后的 3 步处理 |
| `src/presets.rs` | `FilmPreset` 新增 `default_skin_hsl/ split_tone / sharp` 默认值字段 |
| `src/presets.rs` | 所有预设配方数据写入 |
| `src/lib.rs` | 导出新模块函数 |

#### 阶段二：GUI Pro 界面

| 文件 | 变更 |
|------|------|
| `src/bin/filmrust_gui_pro.rs` | 新增 "肤色优化" 层 UI（orange_hue/sat/lum + yellow_sat + red_lum + strength + enable checkbox） |
| `src/bin/filmrust_gui_pro.rs` | 新增 "色调分离" 层 UI（高光色相/饱和 + 阴影色相/饱和 + balance + strength + enable checkbox） |
| `src/bin/filmrust_gui_pro.rs` | 新增 "输出锐化" UI（amount/radius slider + enable checkbox + "自动"按钮） |
| `src/bin/filmrust_gui_pro.rs` | 初始化时默认启用 SkinHSL + SplitTone，Sharp 默认关闭 |
| `src/bin/filmrust_gui_pro.rs` | `do_process` 中串联新 3 层处理 |
| `src/bin/filmrust_gui_pro.rs` | 切换预设时更新 SkinHSL + SplitTone 默认值 |

#### 阶段三：CLI + JSX

| 文件 | 变更 |
|------|------|
| `src/main.rs` | CLI 新增 `--skin-hsl / --split-tone / --sharp` 参数组 |
| `src/main.rs` | CLI process_image 中应用默认配方 + 用户传入微调 |
| JSX | 新增"高级调节"下的 3 个折叠面板 |
| JSX | 构建命令时附加新参数 |

#### 阶段四：预设列表重构

| 文件 | 变更 |
|------|------|
| `src/main.rs` | `FILM_STYLES` 数组：删12个冗余，增6个热门，黑白从17缩到10 |
| `src/main.rs` | 删除的12个移入 `FILM_STYLES_SPECIAL` 折叠分区 |
| `胶片调色.jsx` | `STYLE_LIST` 同步更新 |
| `胶片调色.jsx` | GUI 增加分组折叠性 |

---

### 四、预设列表重构结果（预估）

**常用（16个）**：Ultramax 400, Pro 400H, Natura 1600, Portra 400, Gold 200, Ektar 100, Tri-X 400, Superia 400, Provia 100F, Velvia 50, CineStill 800T, CineStill 50D, HP5 Plus 400, Standard Daylight, ColorPlus 200(新), Fuji C200(新)

**人像（6个）**：Portra 160, Portra 800, Superia 200, Superia 100, Agfa Vista 200, Lucky Color 200

**风光（6个）**：Ektachrome 100, Ektachrome 100VS, Kodachrome 64, Kodachrome 25, Astia 100F, Pro Image 100(新)

**黑白（10个）**：Tri-X, HP5, Delta 100, Delta 400, Neopan 400, APX 400, T-Max 400, XP2 Super, Acros 100 II(新), Delta 3200(新)

**拍立得（3个）**：Polaroid 600, SX-70, i-Type

**特殊/小众（折叠~12个）**：Agfa Vista 400/100, Agfa Optima/Precisa, Ferrania Solaris 400/100, Lomo Purple, Velvia 50 Artistic 等 + 被折叠的8个小众黑白

总计：**常用16 + 人像6 + 风光6 + 黑白10 + 拍立得3 + 特殊12(折叠) = 53种**

---

### 五、实施注意事项（风险管控）

1. **先备份后改**（已完成）
2. **预设列表重构只改2个文件**：`src/main.rs` + `胶片调色.jsx`
3. **删除的预设保留代码注释**：不删 FilmStock 数据，只从 `FILM_STYLES`/`STYLE_LIST` 移除
4. **JSX 兼容性**：旧版本 JSX 调用新版 CLI 没问题（缺省参数已有默认值）；新版 JSX 调用旧版 CLI 可能报参数未知 → 需要同步发布
5. **每个新 Layer 默认关闭** → 用户升级后视觉不变，需要手动打开才看到效果
6. **性能**：Sharp 只在输出最终图时执行一次（不要实时预览），SkinHSL 和 SplitTone 只是像素级矩阵运算，比 filmr 物理模拟轻量得多

---

### 六、验收标准

1. ✅ `cargo clippy --all-targets -- -D warnings` 零警告
2. ✅ `cargo build --release` 通过
3. ✅ GUI Pro 启动后看到 3 个新层（默认已启用 SkinHSL + SplitTone）
4. ✅ 选 Portra 400，皮肤明显比之前通透（橙色饱和度降低、明度提高生效）
5. ✅ SplitTone 开关切换能看到阴影色调变化（青绿 ↔ 中性）
6. ✅ CLI 传 `--warmth=0.1 --skin-hsl` 兼容旧命令
7. ✅ JSX 新版本选 Ultramax 400 不用任何手动调整，结果与 GUI Pro 一致
8. ✅ 切回老预设，之前的效果不变（向后兼容）
