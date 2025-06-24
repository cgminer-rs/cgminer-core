//! 挖矿核心特征定义

use crate::device::{DeviceInfo, DeviceConfig, MiningDevice};
use crate::error::CoreError;
use crate::types::{Work, MiningResult};
use crate::CoreType;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// 核心信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    /// 核心名称
    pub name: String,
    /// 核心类型
    pub core_type: CoreType,
    /// 版本
    pub version: String,
    /// 描述
    pub description: String,
    /// 作者
    pub author: String,
    /// 支持的设备类型
    pub supported_devices: Vec<String>,
    /// 创建时间
    pub created_at: SystemTime,
}

impl CoreInfo {
    /// 创建新的核心信息
    pub fn new(
        name: String,
        core_type: CoreType,
        version: String,
        description: String,
        author: String,
        supported_devices: Vec<String>,
    ) -> Self {
        Self {
            name,
            core_type,
            version,
            description,
            author,
            supported_devices,
            created_at: SystemTime::now(),
        }
    }
}

/// 核心能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreCapabilities {
    /// 是否支持自动调优
    pub supports_auto_tuning: bool,
    /// 温度相关能力
    pub temperature_capabilities: TemperatureCapabilities,
    /// 电压相关能力
    pub voltage_capabilities: VoltageCapabilities,
    /// 频率相关能力
    pub frequency_capabilities: FrequencyCapabilities,
    /// 风扇相关能力
    pub fan_capabilities: FanCapabilities,
    /// 是否支持多链
    pub supports_multiple_chains: bool,
    /// 最大设备数量
    pub max_devices: Option<u32>,
    /// 支持的算法
    pub supported_algorithms: Vec<String>,
    /// CPU特有能力（仅CPU核心使用）
    pub cpu_capabilities: Option<CpuSpecificCapabilities>,
    /// 核心类型
    pub core_type: CoreType,
}

/// 温度相关能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemperatureCapabilities {
    /// 是否支持温度监控
    pub supports_monitoring: bool,
    /// 是否支持温度控制（如风扇调节）
    pub supports_control: bool,
    /// 是否支持温度阈值告警
    pub supports_threshold_alerts: bool,
    /// 温度监控精度（摄氏度）
    pub monitoring_precision: Option<f32>,
}

/// 电压相关能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VoltageCapabilities {
    /// 是否支持电压监控
    pub supports_monitoring: bool,
    /// 是否支持电压控制
    pub supports_control: bool,
    /// 电压控制范围（毫伏）
    pub control_range: Option<(u32, u32)>,
}

/// 频率相关能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyCapabilities {
    /// 是否支持频率监控
    pub supports_monitoring: bool,
    /// 是否支持频率控制
    pub supports_control: bool,
    /// 频率控制范围（MHz）
    pub control_range: Option<(u32, u32)>,
}

/// 风扇相关能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FanCapabilities {
    /// 是否支持风扇监控
    pub supports_monitoring: bool,
    /// 是否支持风扇控制
    pub supports_control: bool,
    /// 风扇数量
    pub fan_count: Option<u32>,
}

/// CPU特有能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSpecificCapabilities {
    /// 支持的SIMD指令集
    pub simd_support: Vec<String>,
    /// 是否支持CPU绑定
    pub supports_cpu_affinity: bool,
    /// 是否支持NUMA感知
    pub supports_numa_awareness: bool,
    /// 物理核心数
    pub physical_cores: u32,
    /// 逻辑核心数
    pub logical_cores: u32,
    /// 缓存信息
    pub cache_info: Option<CpuCacheInfo>,
}

/// CPU缓存信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuCacheInfo {
    /// L1数据缓存大小（KB）
    pub l1_data_kb: u32,
    /// L1指令缓存大小（KB）
    pub l1_instruction_kb: u32,
    /// L2缓存大小（KB）
    pub l2_kb: u32,
    /// L3缓存大小（KB）
    pub l3_kb: u32,
}

impl Default for CoreCapabilities {
    fn default() -> Self {
        Self {
            supports_auto_tuning: false,
            temperature_capabilities: TemperatureCapabilities::default(),
            voltage_capabilities: VoltageCapabilities::default(),
            frequency_capabilities: FrequencyCapabilities::default(),
            fan_capabilities: FanCapabilities::default(),
            supports_multiple_chains: false,
            max_devices: None,
            supported_algorithms: vec!["SHA256".to_string()],
            cpu_capabilities: None,
            core_type: CoreType::Custom("unknown".to_string()),
        }
    }
}

impl Default for TemperatureCapabilities {
    fn default() -> Self {
        Self {
            supports_monitoring: false,
            supports_control: false,
            supports_threshold_alerts: false,
            monitoring_precision: None,
        }
    }
}

impl Default for VoltageCapabilities {
    fn default() -> Self {
        Self {
            supports_monitoring: false,
            supports_control: false,
            control_range: None,
        }
    }
}

impl Default for FrequencyCapabilities {
    fn default() -> Self {
        Self {
            supports_monitoring: false,
            supports_control: false,
            control_range: None,
        }
    }
}

impl Default for FanCapabilities {
    fn default() -> Self {
        Self {
            supports_monitoring: false,
            supports_control: false,
            fan_count: None,
        }
    }
}

impl Default for CpuSpecificCapabilities {
    fn default() -> Self {
        Self {
            simd_support: vec![],
            supports_cpu_affinity: false,
            supports_numa_awareness: false,
            physical_cores: 1,
            logical_cores: 1,
            cache_info: None,
        }
    }
}

impl Default for CpuCacheInfo {
    fn default() -> Self {
        Self {
            l1_data_kb: 32,
            l1_instruction_kb: 32,
            l2_kb: 256,
            l3_kb: 8192,
        }
    }
}

/// 核心配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreConfig {
    /// 核心名称
    pub name: String,
    /// 是否启用
    pub enabled: bool,
    /// 设备配置
    pub devices: Vec<DeviceConfig>,
    /// 自定义参数
    pub custom_params: HashMap<String, serde_json::Value>,
}

impl Default for CoreConfig {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            enabled: true,
            devices: Vec::new(),
            custom_params: HashMap::new(),
        }
    }
}

/// 核心统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreStats {
    /// 核心名称
    pub core_name: String,
    /// 设备数量
    pub device_count: u32,
    /// 活跃设备数量
    pub active_devices: u32,
    /// 总算力
    pub total_hashrate: f64,
    /// 平均算力
    pub average_hashrate: f64,
    /// 接受的工作数
    pub accepted_work: u64,
    /// 拒绝的工作数
    pub rejected_work: u64,
    /// 硬件错误数
    pub hardware_errors: u64,
    /// 运行时间
    pub uptime: std::time::Duration,
    /// 最后更新时间
    pub last_updated: SystemTime,
}

impl Default for CoreStats {
    fn default() -> Self {
        Self::new("unknown".to_string())
    }
}

impl CoreStats {
    /// 创建新的核心统计信息
    pub fn new(core_name: String) -> Self {
        Self {
            core_name,
            device_count: 0,
            active_devices: 0,
            total_hashrate: 0.0,
            average_hashrate: 0.0,
            accepted_work: 0,
            rejected_work: 0,
            hardware_errors: 0,
            uptime: std::time::Duration::from_secs(0),
            last_updated: SystemTime::now(),
        }
    }

    /// 计算错误率
    pub fn error_rate(&self) -> f64 {
        let total_work = self.accepted_work + self.rejected_work;
        if total_work == 0 {
            0.0
        } else {
            self.rejected_work as f64 / total_work as f64
        }
    }
}

/// 挖矿核心特征
#[async_trait]
pub trait MiningCore: Send + Sync {
    /// 获取核心信息
    fn get_info(&self) -> &CoreInfo;

    /// 获取核心能力
    fn get_capabilities(&self) -> &CoreCapabilities;

    /// 初始化核心
    async fn initialize(&mut self, config: CoreConfig) -> Result<(), CoreError>;

    /// 启动核心
    async fn start(&mut self) -> Result<(), CoreError>;

    /// 停止核心
    async fn stop(&mut self) -> Result<(), CoreError>;

    /// 重启核心
    async fn restart(&mut self) -> Result<(), CoreError>;

    /// 扫描设备
    async fn scan_devices(&self) -> Result<Vec<DeviceInfo>, CoreError>;

    /// 创建设备
    async fn create_device(&self, device_info: DeviceInfo) -> Result<Box<dyn MiningDevice>, CoreError>;

    /// 获取所有设备
    async fn get_devices(&self) -> Result<Vec<Box<dyn MiningDevice>>, CoreError>;

    /// 获取设备数量
    async fn device_count(&self) -> Result<u32, CoreError>;

    /// 提交工作到所有设备 - 使用Arc<Work>实现零拷贝
    async fn submit_work(&mut self, work: std::sync::Arc<Work>) -> Result<(), CoreError>;

    /// 收集所有设备的挖矿结果
    async fn collect_results(&mut self) -> Result<Vec<MiningResult>, CoreError>;

    /// 获取核心统计信息
    async fn get_stats(&self) -> Result<CoreStats, CoreError>;

    /// 健康检查
    async fn health_check(&self) -> Result<bool, CoreError>;

    /// 验证配置
    fn validate_config(&self, config: &CoreConfig) -> Result<(), CoreError>;

    /// 获取默认配置
    fn default_config(&self) -> CoreConfig;

    /// 关闭核心
    async fn shutdown(&mut self) -> Result<(), CoreError>;
}
