# 星TAP 胶片调色 | FilmRust Studio

![FilmRust Studio](assets/icon.png)

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
3. 7 个核心调节层微调曝光、色调、肤色
4. 导出 → 搞定

不需要学任何东西。拖动鼠标，就能出片。

---

## 🛠️ 极客看这里

### 技术栈

| 模块 | 技术 |
|------|------|
| 前端 GUI | egui (Rust 原生 UI 框架) |
| 物理引擎 | 自研 filmr 核心库（Rust） |
| 图层管线 | 7 层 F32 精度合成管线 |
| 图像处理 | image-rs + 自研算法 |
| 导出 | JPEG/PNG（保留 EXIF） |
| 二进制大小 | GUI 版 ~17MB，CLI 版 ~2MB |

### 核心特性

- **物理级胶片模拟**：57 种真实胶卷光谱数据（Kodak、Fuji、Ilford 等），三层乳剂反应级模拟
- **7 层 F32 精度管线**：胶片基底 → 色彩 → 曲线 → 肤色优化 → 现代色调 → 色调分离 → 输出锐化
- **16-bit TIFF 支持**：高位深输入/输出，保留全动态范围
- **EXIF 保留**：导出时原样复制拍摄信息
- **无外部依赖**：单 exe 绿色运行，无需 Python / Photoshop / 任何运行时

### 性能指标

- 1920x1280 预览：<50ms 更新
- 全分辨率导出：<2s（视尺寸）
- 内存：~80MB 峰值
- GPU：支持 OpenGL/Vulkan/Metal（egui 自动选择）

---

## 📦 下载

| 平台 | 文件 | 大小 |
|------|------|------|
| Windows | [FilmRust-Studio-Win-v7.3.0.zip](https://github.com/cscb603/FilmRust-Studio/releases) | ~17MB |
| macOS (Intel/Apple Silicon) | [FilmRust_Studio_Pro_macos_v7.3.dmg](https://github.com/cscb603/FilmRust-Studio/releases) | ~XXMB |

> 💡 Windows 版绿色免安装，解压即可运行。macOS 版需 macOS 12+。

---

## 🚀 快速开始

### Windows
1. 下载 `FilmRust-Studio-Win-v7.3.0.zip`
2. 解压 → 双击 `filmrust-gui-pro.exe`
3. 拖入照片 → 选风格 → 调参数 → 导出

### macOS
1. 下载 `FilmRust_Studio_Pro_macos_v7.3.dmg`
2. 双击挂载 → 拖入 Applications
3. 打开 → 拖入照片 → 选风格 → 导出

### Photoshop 集成
```bash
# 一键安装脚本（Windows）
右键 install_to_ps.bat → 以管理员身份运行
# PS 菜单 → 文件 → 脚本 → FilmRust_Studio
```

---

## ⚖️ 开源

本项目基于 Rust 开发，核心算法开源于 GitHub。
欢迎 Issue 和 PR，也欢迎 Star 支持。

---

星TAP 实验室 © 2026 | [cscb603@qq.com](mailto:cscb603@qq.com) | 极致速度，极简生活。
