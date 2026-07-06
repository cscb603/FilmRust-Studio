# 星TAP 胶片调色 | FilmRust Studio

![FilmRust Studio 界面](dist/win-x64/7.3.4界面图.png)

**物理级胶片模拟工具 — 不是调色，是"重新冲洗"**

把你的数码照片，通过物理引擎重新"冲洗"成胶片的样子。不是套滤镜，不是调颜色曲线，是模拟胶片三层乳剂真实的化学反应。

---

## ✨ 小白看这里

**你是不是也遇到过这些烦恼？**

- 花大价钱买的"胶片滤镜"——套上去特别假，一股模板味
- LR/PS 里拉了半天的曲线——还是不像真胶片
- 网上教程一堆，调出来却根本不是那个味

**FilmRust Studio 不一样。** 它不"调色"，而是模拟了胶片乳剂的物理化学反应。57 种真实胶卷的光谱数据，层叠调整，输出你想要的质感。

**使用步骤：**
1. 打开软件 → 拖入照片
2. 选一种胶片风格（57 种慢慢试）
3. 10 种调节层微调色彩、曲线、肤色、色调分离
4. 导出 → 搞定

不需要学任何东西。拖动鼠标，就能出片。

---

## 🛠️ 极客看这里

### 技术栈

| 模块 | 技术 |
|------|------|
| 前端 GUI | egui 0.34 (Rust 原生 UI 框架) |
| 物理引擎 | filmr 0.13 核心库（Rust） |
| 图层管线 | 10 层 F32 精度合成管线 |
| 图像处理 | image-rs 0.25 + 自研算法 |
| 导出 | JPEG/PNG（保留 EXIF） |
| 二进制大小 | GUI 版 ~17MB，CLI 版 ~2MB |
| 编译 | Rust 2021 edition, LTO 全量优化 |

### v7.3.4 更新内容

| 改进 | 详情 |
|------|------|
| **色彩校准重写** | 色温暖调用 R↑(主)+G↑(辅)+B↓(辅) 三通道权控，告别传统 R↑B↓ 的"屎黄/紫蓝"；亮度权重保护暗部高光不偏色 |
| **quad_boost 渐变加浓** | 滑杆中间温和精准、两端加速加浓，适应严重偏色胶片基座 |
| **肤色优化增强** | 减黄/减绿/加粉/加红强度范围提升至 18-20%，同样 quad_boost 曲线控制过渡自然 |
| **soft_clamp 自然滚降** | 边界用 smoothstep 三次缓动代替 hard clamp，消除生硬截止 |
| **动画管线修复** | 显影动画不再"闪回"胶片基座，全程平滑过渡到最终合成效果 |
| **性能大幅提升** | composite() 跳过参数全默认的 idle 层级，避免无谓的全像素遍历 |
| **预设保存/加载** | 保存全部图层参数为预设，一键加载复用 |
| **全局强度滑杆** | 整体效果透明度控制，原图到效果图无缝混合 |

### 性能指标

- 1920x1280 预览：<30ms 更新（quad_boost 优化后）
- 全分辨率导出：<2s（视尺寸）
- 内存：~80MB 峰值
- GPU：支持 OpenGL/Vulkan/Metal（egui 自动选择）

---

## 📦 下载

| 平台 | 版本 | 文件 | 大小 |
|------|------|------|------|
| Windows | v7.3.4 | [FilmRust-Studio-Win-v7.3.4.exe](dist/win-x64/FilmRust-Studio-v7.3.4.exe) | ~17MB |
| macOS | v7.3 | [FilmRust_Studio_Pro_macos.dmg](https://github.com/cscb603/FilmRust-Studio/releases) | — |

> 💡 Windows 版绿色免安装，直接双击运行。macOS 版需 macOS 12+。

---

## 🚀 快速开始

### Windows
1. 下载 `FilmRust-Studio-Win-v7.3.4.exe`
2. 双击运行 → 拖入照片
3. 左侧选胶片风格 → 右侧调色彩/曲线/肤色
4. 点「开始显影」预览 → 导出

### macOS
1. 下载 `.dmg` → 拖入 Applications
2. 打开 → 拖入照片 → 选风格 → 导出

### Photoshop 集成
```bash
# 一键安装脚本（Windows）
右键 install_to_ps.bat → 以管理员身份运行
# PS 菜单 → 文件 → 脚本 → FilmRust_Studio
```

---

## 🧬 渲染管线

```
原图 → filmr 物理引擎 → 胶片基座
                          ↓
              ┌── Color（色温/色调/饱和度）
              ├── Curves（对比度/高光/阴影）
              ├── Grain（颗粒感）
              ├── Vignette（暗角/光晕）
              ├── LightLeak（漏光）
              ├── Blur（模糊特效）
              ├── SkinHSL（肤色优化）
              ├── ModernTone（现代色调）
              ├── SplitTone（色调分离）
              └── Sharp（输出锐化）
                          ↓
              global_strength 混合 → 导出
```

全部调节层作用在胶片基座之后，F32 精度累加，soft_clamp 边界滚降，每层支持独立 blend_mode + opacity。

---

## ⚖️ 开源

本项目基于 Rust 开发，核心算法开源于 GitHub。
欢迎 Issue 和 PR，也欢迎 Star 支持。

星TAP 实验室 © 2026 | [cscb603@qq.com](mailto:cscb603@qq.com) | 极致速度，极简生活。
