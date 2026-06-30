//! 统一错误处理 - 遵循 rust-core-lib 规范
//!
//! - 库层：`FilmRustError` + `thiserror` (具体、可匹配)
//! - 应用层：`anyhow::Result` (灵活、上下文链)

use thiserror::Error;

/// 具体错误类型（库层专用）
#[derive(Error, Debug)]
pub enum FilmRustError {
    #[error("输入文件不存在: {0}")]
    InputNotFound(String),

    #[error("不支持的图像格式: {0}")]
    UnsupportedFormat(String),

    #[error("图像处理失败: {0}")]
    ImageProcessingFailed(String),

    #[error("找不到胶片预设: {0}")]
    PresetNotFound(String),

    #[error("PS .jsx 脚本生成失败: {0}")]
    JsxGenerationFailed(String),

    #[error("IO 失败: {0}")]
    Io(#[from] std::io::Error),

    #[error("参数验证失败: {0}")]
    InvalidParameter(String),

    #[error("{0}")]
    Other(String),
}

/// 应用层统一 Result 类型
pub type FilmRustResult<T> = anyhow::Result<T>;

// 便捷宏 re-export (与 rust-core-lib 保持一致)
pub use anyhow::{anyhow as anyhow_err, bail as bail_err};

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn test_error_variants() {
        let err = FilmRustError::InputNotFound("test.jpg".into());
        assert!(err.to_string().contains("输入文件不存在"));

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let wrapped: FilmRustError = io_err.into();
        assert!(wrapped.to_string().contains("IO"));
    }

    #[test]
    fn test_anyhow_context() -> FilmRustResult<()> {
        let result: FilmRustResult<()> = Err(anyhow_err!("test error"));
        let with_ctx = result.with_context(|| "additional context");
        assert!(with_ctx.is_err());
        Ok(())
    }
}
