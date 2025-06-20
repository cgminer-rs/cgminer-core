//! CGMiner Core - 核心特征和类型定义
//!
//! 这个库定义了所有挖矿核心必须实现的基础特征和类型，
//! 为不同的挖矿设备提供统一的接口。

pub mod core;
pub mod device;
pub mod error;
pub mod registry;
pub mod types;
pub mod work;

// 重新导出常用类型
pub use core::{MiningCore, CoreInfo, CoreCapabilities, CoreConfig, CoreStats};
pub use device::{DeviceInfo, DeviceStatus, DeviceStats, DeviceConfig, MiningDevice};
pub use error::{CoreError, DeviceError};
pub use registry::{CoreRegistry, CoreFactory};
pub use types::{Work, MiningResult, HashRate, Temperature, Voltage, Frequency};
pub use work::{WorkManager, WorkQueue};

/// 库版本
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// 核心类型标识符
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum CoreType {
    /// 软算法核心，使用CPU进行真实算法计算
    Software,
    /// ASIC 核心，用于实际硬件挖矿
    Asic,
    /// 自定义核心类型
    Custom(String),
}

impl std::fmt::Display for CoreType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CoreType::Software => write!(f, "software"),
            CoreType::Asic => write!(f, "asic"),
            CoreType::Custom(name) => write!(f, "{}", name),
        }
    }
}

impl std::str::FromStr for CoreType {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "software" => Ok(CoreType::Software),
            "asic" => Ok(CoreType::Asic),
            _ => Ok(CoreType::Custom(s.to_string())),
        }
    }
}
