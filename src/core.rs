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
    /// 是否支持温度监控
    pub supports_temperature_monitoring: bool,
    /// 是否支持电压控制
    pub supports_voltage_control: bool,
    /// 是否支持频率控制
    pub supports_frequency_control: bool,
    /// 是否支持风扇控制
    pub supports_fan_control: bool,
    /// 是否支持多链
    pub supports_multiple_chains: bool,
    /// 最大设备数量
    pub max_devices: Option<u32>,
    /// 支持的算法
    pub supported_algorithms: Vec<String>,
}

impl Default for CoreCapabilities {
    fn default() -> Self {
        Self {
            supports_auto_tuning: false,
            supports_temperature_monitoring: false,
            supports_voltage_control: false,
            supports_frequency_control: false,
            supports_fan_control: false,
            supports_multiple_chains: false,
            max_devices: None,
            supported_algorithms: vec!["SHA256".to_string()],
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

    /// 提交工作到所有设备
    async fn submit_work(&mut self, work: Work) -> Result<(), CoreError>;

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
