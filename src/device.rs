//! 设备相关类型和特征定义

use crate::error::DeviceError;
use crate::types::{Work, MiningResult, HashRate, Temperature, Voltage, Frequency};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, Instant};
use tracing::{debug, trace};

/// 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// 设备ID
    pub id: u32,
    /// 设备名称
    pub name: String,
    /// 设备类型
    pub device_type: String,
    /// 链ID
    pub chain_id: u8,
    /// 设备路径
    pub device_path: Option<String>,
    /// 序列号
    pub serial_number: Option<String>,
    /// 固件版本
    pub firmware_version: Option<String>,
    /// 硬件版本
    pub hardware_version: Option<String>,
    /// 芯片数量
    pub chip_count: Option<u32>,
    /// 当前温度
    pub temperature: Option<f32>,
    /// 当前电压
    pub voltage: Option<u32>,
    /// 当前频率
    pub frequency: Option<u32>,
    /// 风扇速度
    pub fan_speed: Option<u32>,
    /// 创建时间
    pub created_at: SystemTime,
    /// 更新时间
    pub updated_at: SystemTime,
}

impl DeviceInfo {
    /// 创建新的设备信息
    pub fn new(id: u32, name: String, device_type: String, chain_id: u8) -> Self {
        let now = SystemTime::now();
        Self {
            id,
            name,
            device_type,
            chain_id,
            device_path: None,
            serial_number: None,
            firmware_version: None,
            hardware_version: None,
            chip_count: None,
            temperature: None,
            voltage: None,
            frequency: None,
            fan_speed: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新温度
    pub fn update_temperature(&mut self, temperature: f32) {
        self.temperature = Some(temperature);
        self.updated_at = SystemTime::now();
    }

    /// 更新电压
    pub fn update_voltage(&mut self, voltage: u32) {
        self.voltage = Some(voltage);
        self.updated_at = SystemTime::now();
    }

    /// 更新频率
    pub fn update_frequency(&mut self, frequency: u32) {
        self.frequency = Some(frequency);
        self.updated_at = SystemTime::now();
    }
}

/// 设备状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceStatus {
    /// 未初始化
    Uninitialized,
    /// 初始化中
    Initializing,
    /// 空闲
    Idle,
    /// 运行中
    Running,
    /// 暂停
    Paused,
    /// 错误
    Error(String),
    /// 离线
    Offline,
}

impl std::fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DeviceStatus::Uninitialized => write!(f, "未初始化"),
            DeviceStatus::Initializing => write!(f, "初始化中"),
            DeviceStatus::Idle => write!(f, "空闲"),
            DeviceStatus::Running => write!(f, "运行中"),
            DeviceStatus::Paused => write!(f, "暂停"),
            DeviceStatus::Error(msg) => write!(f, "错误: {}", msg),
            DeviceStatus::Offline => write!(f, "离线"),
        }
    }
}

/// 滑动窗口算力统计（内部使用）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollingHashrateStats {
    /// 1分钟滑动窗口算力
    pub rolling_1m: f64,
    /// 5分钟滑动窗口算力
    pub rolling_5m: f64,
    /// 15分钟滑动窗口算力
    pub rolling_15m: f64,
    /// 最后更新时间（跳过序列化）
    #[serde(skip, default = "Instant::now")]
    pub last_update: Instant,
    /// 日志计数器（跳过序列化）
    #[serde(skip)]
    pub log_counter: u64,
}

impl Default for RollingHashrateStats {
    fn default() -> Self {
        Self {
            rolling_1m: 0.0,
            rolling_5m: 0.0,
            rolling_15m: 0.0,
            last_update: Instant::now(),
            log_counter: 0,
        }
    }
}

impl RollingHashrateStats {
    /// 更新滑动窗口算力统计
    pub fn update(&mut self, hashes_done: u64, time_diff: f64) {
        if time_diff <= 0.0 {
            trace!("滑动窗口更新：时间差无效 ({:.6}s)，跳过更新", time_diff);
            return;
        }

        self.log_counter += 1;
        let hashrate = hashes_done as f64 / time_diff;

        // 每1000次更新才输出一次日志，进一步减少日志输出
        if self.log_counter % 1000 == 0 {
            debug!("滑动窗口更新：{} 哈希, {:.3}s, {:.2} H/s (第{}次更新)",
                   hashes_done, time_diff, hashrate, self.log_counter);
        }

        // 使用改进的指数衰减算法更新滑动窗口算力
        Self::decay_time(&mut self.rolling_1m, hashrate, time_diff, 60.0);
        Self::decay_time(&mut self.rolling_5m, hashrate, time_diff, 300.0);
        Self::decay_time(&mut self.rolling_15m, hashrate, time_diff, 900.0);

        // 完全禁用滑动窗口算力日志，改为使用聚合显示
        // 如果需要调试，可以临时启用以下代码：
        // if self.log_counter % 2000 == 0 {
        //     debug!("滑动窗口算力：1m={:.2} H/s, 5m={:.2} H/s, 15m={:.2} H/s",
        //            self.rolling_1m, self.rolling_5m, self.rolling_15m);
        // }

        self.last_update = Instant::now();
    }

    /// 改进的指数衰减算法（类似传统cgminer的decay_time函数）
    fn decay_time(rolling: &mut f64, hashrate: f64, time_diff: f64, interval: f64) {
        if time_diff <= 0.0 {
            return;
        }

        // 初始情况处理
        if *rolling == 0.0 {
            *rolling = hashrate;
            return;
        }

        // 计算衰减因子，确保数值稳定性
        let decay_factor = (-time_diff / interval).exp();
        let weight = 1.0 - decay_factor;

        // 更新滑动平均值
        *rolling = *rolling * decay_factor + hashrate * weight;

        // 确保结果为正数
        if *rolling < 0.0 {
            *rolling = 0.0;
        }
    }
}

/// 设备统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStats {
    /// 设备ID
    pub device_id: u32,
    /// 当前算力
    pub current_hashrate: HashRate,
    /// 平均算力
    pub average_hashrate: HashRate,
    /// 1分钟平均算力
    pub hashrate_1m: HashRate,
    /// 5分钟平均算力
    pub hashrate_5m: HashRate,
    /// 15分钟平均算力
    pub hashrate_15m: HashRate,
    /// 总计执行的哈希次数
    pub total_hashes: u64,
    /// 接受的工作数
    pub accepted_work: u64,
    /// 拒绝的工作数
    pub rejected_work: u64,
    /// 硬件错误数
    pub hardware_errors: u64,
    /// 运行时间
    pub uptime: std::time::Duration,
    /// 当前温度
    pub temperature: Option<Temperature>,
    /// 当前电压
    pub voltage: Option<Voltage>,
    /// 当前频率
    pub frequency: Option<Frequency>,
    /// 风扇速度
    pub fan_speed: Option<u32>,
    /// 功耗
    pub power_consumption: Option<f64>,
    /// 最后更新时间
    pub last_updated: SystemTime,
    /// 滑动窗口算力统计（内部使用）
    pub(crate) rolling_stats: RollingHashrateStats,
    /// 算力更新计数器（跳过序列化）
    #[serde(skip)]
    pub(crate) hashrate_update_counter: u64,
}

impl DeviceStats {
    /// 创建新的设备统计信息
    pub fn new(device_id: u32) -> Self {
        Self {
            device_id,
            current_hashrate: HashRate::new(0.0),
            average_hashrate: HashRate::new(0.0),
            hashrate_1m: HashRate::new(0.0),
            hashrate_5m: HashRate::new(0.0),
            hashrate_15m: HashRate::new(0.0),
            total_hashes: 0,
            accepted_work: 0,
            rejected_work: 0,
            hardware_errors: 0,
            uptime: std::time::Duration::from_secs(0),
            temperature: None,
            voltage: None,
            frequency: None,
            fan_speed: None,
            power_consumption: None,
            last_updated: SystemTime::now(),
            rolling_stats: RollingHashrateStats::default(),
            hashrate_update_counter: 0,
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

    /// 计算硬件错误率
    pub fn hardware_error_rate(&self) -> f64 {
        let total_work = self.accepted_work + self.rejected_work;
        if total_work == 0 {
            0.0
        } else {
            self.hardware_errors as f64 / total_work as f64
        }
    }

    /// 更新算力统计（基于实际哈希次数）
    pub fn update_hashrate(&mut self, hashes_done: u64, time_diff: f64) {
        // 更新总哈希次数和计数器
        self.total_hashes += hashes_done;
        self.hashrate_update_counter += 1;

        // 确保时间差在合理范围内，避免除零或异常高的算力值
        let min_time_diff = 0.001; // 最小1毫秒
        let effective_time_diff = if time_diff < min_time_diff {
            // 只在第一次或每1000次时输出时间差警告
            if self.hashrate_update_counter == 1 || self.hashrate_update_counter % 1000 == 0 {
                debug!("设备 {} 时间差过小 ({:.6}s)，使用最小值 {:.3}s",
                       self.device_id, time_diff, min_time_diff);
            }
            min_time_diff
        } else {
            time_diff
        };

        // 计算当前算力
        let current_hashrate = hashes_done as f64 / effective_time_diff;
        self.current_hashrate = HashRate::new(current_hashrate);

        // 每500次更新才输出一次算力更新日志（大幅减少输出）
        if self.hashrate_update_counter % 500 == 0 {
            debug!("设备 {} 算力更新: {} 哈希, {:.3}s, {:.2} H/s (第{}次更新)",
                   self.device_id, hashes_done, effective_time_diff, current_hashrate, self.hashrate_update_counter);
        }

        // 更新滑动窗口算力统计
        self.rolling_stats.update(hashes_done, effective_time_diff);

        // 从滑动窗口统计中获取平均算力
        self.hashrate_1m = HashRate::new(self.rolling_stats.rolling_1m);
        self.hashrate_5m = HashRate::new(self.rolling_stats.rolling_5m);
        self.hashrate_15m = HashRate::new(self.rolling_stats.rolling_15m);

        // 计算总体平均算力（指数移动平均）
        let alpha = 0.1; // 平滑因子
        let new_avg = if self.average_hashrate.hashes_per_second == 0.0 {
            // 初始情况，直接使用当前算力
            current_hashrate
        } else {
            self.average_hashrate.hashes_per_second * (1.0 - alpha) + current_hashrate * alpha
        };
        self.average_hashrate = HashRate::new(new_avg);

        // 完全禁用单个设备的详细算力统计日志，改为使用聚合显示
        // 如果需要调试单个设备，可以临时启用以下代码：
        // if self.hashrate_update_counter % 1000 == 0 {
        //     debug!("设备 {} 算力统计: 当前={:.2} H/s, 1m={:.2} H/s, 5m={:.2} H/s, 15m={:.2} H/s, 平均={:.2} H/s",
        //            self.device_id,
        //            current_hashrate,
        //            self.rolling_stats.rolling_1m,
        //            self.rolling_stats.rolling_5m,
        //            self.rolling_stats.rolling_15m,
        //            new_avg);
        // }

        self.last_updated = SystemTime::now();
    }
}

/// 设备配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// 链ID
    pub chain_id: u8,
    /// 是否启用
    pub enabled: bool,
    /// 频率 (MHz)
    pub frequency: u32,
    /// 电压 (mV)
    pub voltage: u32,
    /// 是否自动调优
    pub auto_tune: bool,
    /// 芯片数量
    pub chip_count: u32,
    /// 温度限制
    pub temperature_limit: f32,
    /// 风扇速度
    pub fan_speed: Option<u32>,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            chain_id: 0,
            enabled: true,
            frequency: 600,
            voltage: 900,
            auto_tune: false,
            chip_count: 64,
            temperature_limit: 80.0,
            fan_speed: Some(50),
        }
    }
}

/// 挖矿设备特征
#[async_trait]
pub trait MiningDevice: Send + Sync {
    /// 获取设备ID
    fn device_id(&self) -> u32;

    /// 获取设备信息
    async fn get_info(&self) -> Result<DeviceInfo, DeviceError>;

    /// 初始化设备
    async fn initialize(&mut self, config: DeviceConfig) -> Result<(), DeviceError>;

    /// 启动设备
    async fn start(&mut self) -> Result<(), DeviceError>;

    /// 停止设备
    async fn stop(&mut self) -> Result<(), DeviceError>;

    /// 重启设备
    async fn restart(&mut self) -> Result<(), DeviceError>;

    /// 提交工作
    async fn submit_work(&mut self, work: Work) -> Result<(), DeviceError>;

    /// 获取挖矿结果
    async fn get_result(&mut self) -> Result<Option<MiningResult>, DeviceError>;

    /// 获取设备状态
    async fn get_status(&self) -> Result<DeviceStatus, DeviceError>;

    /// 获取设备统计信息
    async fn get_stats(&self) -> Result<DeviceStats, DeviceError>;

    /// 设置频率
    async fn set_frequency(&mut self, frequency: u32) -> Result<(), DeviceError>;

    /// 设置电压
    async fn set_voltage(&mut self, voltage: u32) -> Result<(), DeviceError>;

    /// 设置风扇速度
    async fn set_fan_speed(&mut self, speed: u32) -> Result<(), DeviceError>;

    /// 重置设备
    async fn reset(&mut self) -> Result<(), DeviceError>;

    /// 获取设备健康状态
    async fn health_check(&self) -> Result<bool, DeviceError>;
}
