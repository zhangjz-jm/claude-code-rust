//! DCC 工具错误类型

use thiserror::Error;

/// DCC 工具错误类型
#[derive(Error, Debug)]
pub enum DCCError {
    /// 连接错误
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// 工具不可用
    #[error("Tool not available: {tool_type}")]
    ToolNotAvailable { tool_type: String },

    /// 操作超时
    #[error("Operation timeout after {seconds} seconds")]
    Timeout { seconds: u64 },

    /// 操作被取消
    #[error("Operation cancelled")]
    Cancelled,

    /// 执行错误
    #[error("Execution error: {message}")]
    ExecutionError { message: String, details: Option<String> },

    /// 序列化错误
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO 错误
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Python 错误
    #[error("Python error: {0}")]
    PythonError(String),

    /// 无效参数
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    /// 未实现
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// 其他错误
    #[error("DCC error: {0}")]
    Other(String),
}

/// 结果类型别名
pub type DCCResult<T> = Result<T, DCCError>;
