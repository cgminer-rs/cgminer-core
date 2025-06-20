//! 错误类型定义

use thiserror::Error;

/// 核心错误类型
#[derive(Error, Debug)]
pub enum CoreError {
    #[error("设备错误: {0}")]
    Device(#[from] DeviceError),

    #[error("配置错误: {message}")]
    Config { message: String },

    #[error("初始化错误: {message}")]
    Initialization { message: String },

    #[error("运行时错误: {message}")]
    Runtime { message: String },

    #[error("网络错误: {0}")]
    Network(#[from] NetworkError),

    #[error("序列化错误: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("IO错误: {0}")]
    Io(#[from] std::io::Error),

    #[error("未知错误: {message}")]
    Unknown { message: String },
}

/// 设备错误类型
#[derive(Error, Debug)]
pub enum DeviceError {
    #[error("设备未找到: {device_id}")]
    NotFound { device_id: u32 },

    #[error("设备初始化失败: {message}")]
    InitializationFailed { message: String },

    #[error("设备通信错误: {message}")]
    CommunicationError { message: String },

    #[error("设备过热: 当前温度 {current}°C, 限制 {limit}°C")]
    Overheating { current: f32, limit: f32 },

    #[error("设备电压异常: 当前 {current}mV, 期望 {expected}mV")]
    VoltageError { current: u32, expected: u32 },

    #[error("设备频率异常: 当前 {current}MHz, 期望 {expected}MHz")]
    FrequencyError { current: u32, expected: u32 },

    #[error("硬件错误: {message}")]
    HardwareError { message: String },

    #[error("设备超时: 操作 '{operation}' 超时")]
    Timeout { operation: String },

    #[error("设备不支持的操作: {operation}")]
    UnsupportedOperation { operation: String },

    #[error("设备配置无效: {message}")]
    InvalidConfiguration { message: String },
}

/// 网络错误类型
#[derive(Error, Debug)]
pub enum NetworkError {
    #[error("连接失败: {address}")]
    ConnectionFailed { address: String },

    #[error("连接超时: {address}")]
    ConnectionTimeout { address: String },

    #[error("认证失败: {message}")]
    AuthenticationFailed { message: String },

    #[error("协议错误: {message}")]
    ProtocolError { message: String },

    #[error("数据解析错误: {message}")]
    ParseError { message: String },
}

impl CoreError {
    /// 创建配置错误
    pub fn config<S: Into<String>>(message: S) -> Self {
        Self::Config {
            message: message.into(),
        }
    }

    /// 创建初始化错误
    pub fn initialization<S: Into<String>>(message: S) -> Self {
        Self::Initialization {
            message: message.into(),
        }
    }

    /// 创建运行时错误
    pub fn runtime<S: Into<String>>(message: S) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }

    /// 创建未知错误
    pub fn unknown<S: Into<String>>(message: S) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }
}

impl DeviceError {
    /// 创建设备未找到错误
    pub fn not_found(device_id: u32) -> Self {
        Self::NotFound { device_id }
    }

    /// 创建初始化失败错误
    pub fn initialization_failed<S: Into<String>>(message: S) -> Self {
        Self::InitializationFailed {
            message: message.into(),
        }
    }

    /// 创建通信错误
    pub fn communication_error<S: Into<String>>(message: S) -> Self {
        Self::CommunicationError {
            message: message.into(),
        }
    }

    /// 创建硬件错误
    pub fn hardware_error<S: Into<String>>(message: S) -> Self {
        Self::HardwareError {
            message: message.into(),
        }
    }

    /// 创建超时错误
    pub fn timeout<S: Into<String>>(operation: S) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// 创建不支持操作错误
    pub fn unsupported_operation<S: Into<String>>(operation: S) -> Self {
        Self::UnsupportedOperation {
            operation: operation.into(),
        }
    }

    /// 创建配置无效错误
    pub fn invalid_configuration<S: Into<String>>(message: S) -> Self {
        Self::InvalidConfiguration {
            message: message.into(),
        }
    }
}
