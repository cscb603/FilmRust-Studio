//! CLI 参数解析 - 兼容 Python 版 film_style_cli 格式

use clap::{Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};

/// 智能胶片调色系统 - 兼容 film_style_cli Python 版格式
#[derive(Parser, Debug)]
#[command(name = "filmrust", version, about = "智能胶片调色系统", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// 处理图片（兼容 Python 版 --input --output --style 格式）
    Process {
        #[arg(long = "input")]
        input: std::path::PathBuf,

        #[arg(long = "output")]
        output: Option<std::path::PathBuf>,

        #[arg(long = "style")]
        style: Option<String>,

        #[arg(long = "strength", default_value_t = 100)]
        strength: i32,

        #[arg(long = "grain", default_value_t = 100)]
        grain: i32,

        #[arg(long = "auto")]
        auto: bool,
    },

    /// 分析图片（输出 JSON）
    Analyze {
        #[arg(long = "analyze")]
        analyze: std::path::PathBuf,

        #[arg(long = "json-output")]
        json_output: Option<std::path::PathBuf>,
    },

    /// 列出所有风格（兼容 Python 版格式）
    ListStyles {
        #[arg(long = "list-styles")]
        list_styles: bool,
    },

    /// 旧版兼容入口
    Legacy {
        #[arg(long = "input")]
        input: Option<std::path::PathBuf>,

        #[arg(long = "output")]
        output: Option<std::path::PathBuf>,

        #[arg(long = "style")]
        style: Option<String>,

        #[arg(long = "strength", default_value_t = 100)]
        strength: i32,

        #[arg(long = "grain", default_value_t = 100)]
        grain: i32,

        #[arg(long = "analyze")]
        analyze: Option<std::path::PathBuf>,

        #[arg(long = "json-output")]
        json_output: Option<std::path::PathBuf>,

        #[arg(long = "list-styles")]
        list_styles: bool,

        #[arg(long = "auto")]
        auto: bool,
    },
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Serialize, Deserialize)]
pub enum OutputFormat {
    Png,
    Jpeg,
    Tiff,
}
