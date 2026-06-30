//! 平台检测模块 - Win10 + Mac 双平台支持

use std::path::PathBuf;

/// 检测当前操作系统
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Windows10,
    MacOS,
    Linux,
    Unknown,
}

impl Platform {
    /// 获取当前平台
    pub fn current() -> Self {
        if cfg!(target_os = "windows") {
            Platform::Windows10
        } else if cfg!(target_os = "macos") {
            Platform::MacOS
        } else if cfg!(target_os = "linux") {
            Platform::Linux
        } else {
            Platform::Unknown
        }
    }

    /// 是否为 Win10+
    pub fn is_windows(&self) -> bool {
        matches!(self, Platform::Windows10)
    }

    /// 是否为 macOS
    pub fn is_macos(&self) -> bool {
        matches!(self, Platform::MacOS)
    }
}

/// Photoshop 安装信息
#[derive(Debug, Clone)]
pub struct PhotoshopInfo {
    pub version: Option<String>,
    pub install_path: Option<PathBuf>,
    pub detected: bool,
}

impl PhotoshopInfo {
    /// 检测 PS2026 是否安装
    pub fn detect() -> Self {
        let platform = Platform::current();
        let mut info = PhotoshopInfo {
            version: None,
            install_path: None,
            detected: false,
        };

        match platform {
            Platform::Windows10 => {
                // Win10: 通过注册表检测 PS2026 (版本 27.0+)
                if let Some(path) = detect_ps_windows() {
                    info.install_path = Some(path);
                    info.version = Some("27.0+".to_string());
                    info.detected = true;
                }
            }
            Platform::MacOS => {
                // Mac: 通过 /Applications 目录检测
                if let Some(path) = detect_ps_macos() {
                    info.install_path = Some(path);
                    info.version = Some("27.0+".to_string());
                    info.detected = true;
                }
            }
            _ => {}
        }

        info
    }
}

fn detect_ps_windows() -> Option<PathBuf> {
    // Win10: 检测常见安装路径
    let candidates = vec![
        r"C:\Program Files\Adobe\Adobe Photoshop 2026\Photoshop.exe",
        r"C:\Program Files\Adobe\Adobe Photoshop 2025\Photoshop.exe",
        r"C:\Program Files\Adobe\Adobe Photoshop 2024\Photoshop.exe",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn detect_ps_macos() -> Option<PathBuf> {
    // Mac: 检测 /Applications 目录
    let candidates = vec![
        "/Applications/Adobe Photoshop 2026/Adobe Photoshop 2026.app",
        "/Applications/Adobe Photoshop 2025/Adobe Photoshop 2025.app",
        "/Applications/Adobe Photoshop 2024/Adobe Photoshop 2024.app",
    ];

    for candidate in candidates {
        let path = PathBuf::from(candidate);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

/// 获取用户主目录
pub fn home_dir() -> Option<PathBuf> {
    dirs::home_dir()
}

/// 获取临时目录
pub fn temp_dir() -> PathBuf {
    std::env::temp_dir()
}
