//! 基础类型定义

use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// 统一的工作单元结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Work {
    /// 工作UUID - 唯一标识符
    pub id: Uuid,
    /// 工作ID (数字形式，用于兼容)
    pub work_id: u64,
    /// Stratum作业ID
    pub job_id: String,
    /// 区块头数据 (固定80字节)
    #[serde(with = "serde_bytes")]
    pub header: [u8; 80],
    /// 目标难度 (固定32字节)
    #[serde(with = "serde_bytes")]
    pub target: [u8; 32],
    /// Merkle根 (固定32字节)
    #[serde(with = "serde_bytes")]
    pub merkle_root: [u8; 32],
    /// 中间状态 (用于优化哈希计算)
    #[serde(skip)]
    pub midstate: [[u8; 32]; 8],
    /// Extra nonce 1 (来自矿池)
    pub extranonce1: Vec<u8>,
    /// Extra nonce 2 (本地生成)
    pub extranonce2: Vec<u8>,
    /// Extra nonce 2 大小
    pub extranonce2_size: usize,
    /// Coinbase 1
    pub coinbase1: Vec<u8>,
    /// Coinbase 2
    pub coinbase2: Vec<u8>,
    /// Merkle分支
    pub merkle_branches: Vec<Vec<u8>>,
    /// 版本
    pub version: u32,
    /// nBits (难度目标)
    pub nbits: u32,
    /// nTime (时间戳)
    pub ntime: u32,
    /// 工作难度
    pub difficulty: f64,
    /// 创建时间
    pub created_at: SystemTime,
    /// 过期时间
    pub expires_at: SystemTime,
    /// 是否需要清理旧作业
    pub clean_jobs: bool,
}

impl Work {
    /// 创建新的工作单元
    pub fn new(job_id: String, target: [u8; 32], header: [u8; 80], difficulty: f64) -> Self {
        let now = SystemTime::now();
        let id = Uuid::new_v4();

        Self {
            id,
            work_id: id.as_u128() as u64, // 从UUID生成数字ID
            job_id,
            header,
            target,
            merkle_root: [0u8; 32],
            midstate: [[0u8; 32]; 8],
            extranonce1: Vec::new(),
            extranonce2: Vec::new(),
            extranonce2_size: 4, // 默认4字节
            coinbase1: Vec::new(),
            coinbase2: Vec::new(),
            merkle_branches: Vec::new(),
            version: 0,
            nbits: 0,
            ntime: 0,
            difficulty,
            created_at: now,
            expires_at: now + Duration::from_secs(120), // 2分钟过期
            clean_jobs: false,
        }
    }

    /// 从Stratum作业创建工作
    #[allow(clippy::too_many_arguments)]
    pub fn from_stratum_job(
        job_id: String,
        previous_hash: &str,
        coinbase1: Vec<u8>,
        coinbase2: Vec<u8>,
        merkle_branches: Vec<Vec<u8>>,
        version: u32,
        nbits: u32,
        ntime: u32,
        extranonce1: Vec<u8>,
        extranonce2_size: usize,
        difficulty: f64,
        clean_jobs: bool,
    ) -> Result<Self, String> {
        let now = SystemTime::now();
        let id = Uuid::new_v4();

        // 解析previous_hash
        let prev_hash_bytes = hex::decode(previous_hash)
            .map_err(|_| "Invalid previous hash format")?;
        if prev_hash_bytes.len() != 32 {
            return Err("Previous hash must be 32 bytes".to_string());
        }

        let mut header = [0u8; 80];
        // 设置版本 (前4字节)
        header[0..4].copy_from_slice(&version.to_le_bytes());
        // 设置前一个区块哈希 (4-36字节)
        header[4..36].copy_from_slice(&prev_hash_bytes);
        // merkle root将在后续计算中填充 (36-68字节)
        // 设置时间 (68-72字节)
        header[68..72].copy_from_slice(&ntime.to_le_bytes());
        // 设置难度目标 (72-76字节)
        header[72..76].copy_from_slice(&nbits.to_le_bytes());
        // nonce将在挖矿时填充 (76-80字节)

        // 计算目标值
        let target = Self::nbits_to_target(nbits);

        Ok(Self {
            id,
            work_id: id.as_u128() as u64,
            job_id,
            header,
            target,
            merkle_root: [0u8; 32], // 将在计算merkle root时填充
            midstate: [[0u8; 32]; 8],
            extranonce1,
            extranonce2: Vec::new(),
            extranonce2_size,
            coinbase1,
            coinbase2,
            merkle_branches,
            version,
            nbits,
            ntime,
            difficulty,
            created_at: now,
            expires_at: now + Duration::from_secs(120),
            clean_jobs,
        })
    }

    /// 将nBits转换为目标值
    fn nbits_to_target(nbits: u32) -> [u8; 32] {
        let mut target = [0u8; 32];

        let exponent = (nbits >> 24) as usize;
        let mantissa = nbits & 0x00ffffff;

        if exponent <= 3 {
            let mantissa_bytes = mantissa.to_be_bytes();
            let start_idx = 32 - exponent;
            if start_idx < 32 {
                target[start_idx..32].copy_from_slice(&mantissa_bytes[4-exponent..4]);
            }
        } else if exponent < 32 {
            let mantissa_bytes = mantissa.to_be_bytes();
            let start_idx = 32 - exponent;
            target[start_idx..start_idx+3].copy_from_slice(&mantissa_bytes[1..4]);
        }

        target
    }

    /// 检查工作是否过期
    pub fn is_expired(&self) -> bool {
        SystemTime::now() > self.expires_at
    }

    /// 检查工作是否过期（带自定义最大年龄）
    pub fn is_expired_with_max_age(&self, max_age: Duration) -> bool {
        SystemTime::now()
            .duration_since(self.created_at)
            .map(|age| age > max_age)
            .unwrap_or(true)
    }

    /// 获取到期剩余时间
    pub fn time_to_expire(&self) -> Duration {
        self.expires_at.duration_since(SystemTime::now())
            .unwrap_or(Duration::from_secs(0))
    }

    /// 设置extranonce2
    pub fn set_extranonce2(&mut self, extranonce2: Vec<u8>) {
        self.extranonce2 = extranonce2;
    }

    /// 构建完整的coinbase交易
    pub fn build_coinbase(&self) -> Vec<u8> {
        let mut coinbase = Vec::new();
        coinbase.extend_from_slice(&self.coinbase1);
        coinbase.extend_from_slice(&self.extranonce1);
        coinbase.extend_from_slice(&self.extranonce2);
        coinbase.extend_from_slice(&self.coinbase2);
        coinbase
    }

    /// 验证coinbase交易格式
    pub fn validate_coinbase(&self) -> Result<(), String> {
        // 检查coinbase1和coinbase2是否存在
        if self.coinbase1.is_empty() {
            return Err("Coinbase1 is empty".to_string());
        }

        if self.coinbase2.is_empty() {
            return Err("Coinbase2 is empty".to_string());
        }

        // 检查extranonce1是否存在
        if self.extranonce1.is_empty() {
            return Err("Extranonce1 is empty".to_string());
        }

        // 检查extranonce2长度是否正确
        if self.extranonce2.len() != self.extranonce2_size {
            return Err(format!(
                "Extranonce2 length mismatch: expected {}, got {}",
                self.extranonce2_size,
                self.extranonce2.len()
            ));
        }

        // 构建完整的coinbase并检查最小长度
        let coinbase = self.build_coinbase();
        if coinbase.len() < 42 { // 最小的coinbase交易长度
            return Err(format!(
                "Coinbase transaction too short: {} bytes (minimum 42)",
                coinbase.len()
            ));
        }

        // 验证coinbase交易的基本结构
        if let Err(e) = self.parse_coinbase_transaction(&coinbase) {
            return Err(format!("Invalid coinbase transaction structure: {}", e));
        }

        Ok(())
    }

    /// 解析coinbase交易结构（基本验证）
    fn parse_coinbase_transaction(&self, coinbase: &[u8]) -> Result<(), String> {
        if coinbase.len() < 42 {
            return Err("Transaction too short".to_string());
        }

        let mut offset = 0;

        // 检查版本字段 (4 bytes)
        if offset + 4 > coinbase.len() {
            return Err("Missing version field".to_string());
        }
        offset += 4;

        // 检查输入数量 (通常是1字节的0x01)
        if offset >= coinbase.len() {
            return Err("Missing input count".to_string());
        }
        let input_count = coinbase[offset];
        if input_count != 1 {
            return Err(format!("Invalid input count: expected 1, got {}", input_count));
        }
        offset += 1;

        // 检查前一个输出哈希 (32 bytes, 应该全为0)
        if offset + 32 > coinbase.len() {
            return Err("Missing previous output hash".to_string());
        }
        let prev_hash = &coinbase[offset..offset + 32];
        if !prev_hash.iter().all(|&b| b == 0) {
            return Err("Previous output hash should be all zeros for coinbase".to_string());
        }
        offset += 32;

        // 检查前一个输出索引 (4 bytes, 应该是0xffffffff)
        if offset + 4 > coinbase.len() {
            return Err("Missing previous output index".to_string());
        }
        let prev_index = &coinbase[offset..offset + 4];
        if prev_index != [0xff, 0xff, 0xff, 0xff] {
            return Err("Previous output index should be 0xffffffff for coinbase".to_string());
        }
        offset += 4;

        // 检查脚本长度字段
        if offset >= coinbase.len() {
            return Err("Missing script length".to_string());
        }

        // 这里可以添加更多的验证逻辑，但基本结构检查已经足够

        Ok(())
    }

    /// 获取coinbase交易的哈希
    pub fn get_coinbase_hash(&self) -> [u8; 32] {
        let coinbase = self.build_coinbase();
        Self::double_sha256(&coinbase)
    }

    /// 计算并设置merkle root
    pub fn calculate_merkle_root(&mut self) -> Result<(), String> {
        let coinbase = self.build_coinbase();

        // 计算coinbase的双重SHA256
        let mut current_hash = Self::double_sha256(&coinbase);

        // 通过merkle分支计算merkle root
        for branch in &self.merkle_branches {
            if branch.len() != 32 {
                return Err("Invalid merkle branch length".to_string());
            }

            let mut combined = Vec::with_capacity(64);
            combined.extend_from_slice(&current_hash);
            combined.extend_from_slice(branch);

            current_hash = Self::double_sha256(&combined);
        }

        self.merkle_root = current_hash;
        // 更新区块头中的merkle root
        self.header[36..68].copy_from_slice(&current_hash);

        Ok(())
    }

    /// 双重SHA256哈希
    fn double_sha256(data: &[u8]) -> [u8; 32] {
        use sha2::{Sha256, Digest};

        let first_hash = Sha256::digest(data);
        let second_hash = Sha256::digest(first_hash);

        let mut result = [0u8; 32];
        result.copy_from_slice(&second_hash);
        result
    }
}

/// 挖矿结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningResult {
    /// 工作UUID
    pub work_id: Uuid,
    /// 工作ID (数字形式，用于兼容)
    pub work_id_numeric: u64,
    /// 设备ID
    pub device_id: u32,
    /// 随机数
    pub nonce: u32,
    /// Extra nonce 2
    pub extranonce2: Vec<u8>,
    /// 时间戳
    pub timestamp: SystemTime,
    /// 哈希值
    pub hash: Vec<u8>,
    /// 是否满足目标难度
    pub meets_target: bool,
    /// 计算的份额难度
    pub share_difficulty: f64,
}

impl MiningResult {
    /// 创建新的挖矿结果
    pub fn new(
        work_id: Uuid,
        device_id: u32,
        nonce: u32,
        hash: Vec<u8>,
        meets_target: bool,
    ) -> Self {
        Self {
            work_id,
            work_id_numeric: work_id.as_u128() as u64,
            device_id,
            nonce,
            extranonce2: Vec::new(),
            timestamp: SystemTime::now(),
            hash,
            meets_target,
            share_difficulty: 1.0, // 默认值，需要后续计算
        }
    }

    /// 设置extranonce2
    pub fn with_extranonce2(mut self, extranonce2: Vec<u8>) -> Self {
        self.extranonce2 = extranonce2;
        self
    }

    /// 设置份额难度
    pub fn with_share_difficulty(mut self, difficulty: f64) -> Self {
        self.share_difficulty = difficulty;
        self
    }

    /// 计算份额难度
    pub fn calculate_share_difficulty(&mut self) -> Result<(), String> {
        if self.hash.len() != 32 {
            return Err("Invalid hash length".to_string());
        }

        // 将哈希转换为大端序的256位整数
        let mut hash_bytes = [0u8; 32];
        hash_bytes.copy_from_slice(&self.hash);
        hash_bytes.reverse(); // 转换为大端序

        // 计算难度 = 0x00000000FFFF0000000000000000000000000000000000000000000000000000 / hash
        // 这里使用简化的计算方法
        let hash_value = u128::from_be_bytes([
            hash_bytes[0], hash_bytes[1], hash_bytes[2], hash_bytes[3],
            hash_bytes[4], hash_bytes[5], hash_bytes[6], hash_bytes[7],
            hash_bytes[8], hash_bytes[9], hash_bytes[10], hash_bytes[11],
            hash_bytes[12], hash_bytes[13], hash_bytes[14], hash_bytes[15],
        ]);

        if hash_value == 0 {
            self.share_difficulty = f64::INFINITY;
        } else {
            // 使用简化的难度计算
            // 难度1的目标值（前32位）
            const DIFFICULTY_1_TARGET: u128 = 0x00000000FFFF0000000000000000000u128;
            self.share_difficulty = DIFFICULTY_1_TARGET as f64 / hash_value as f64;
        }

        Ok(())
    }
}

/// Nonce验证器
#[derive(Debug, Clone)]
pub struct NonceValidator {
    /// 最近提交的nonce历史（用于重复检测）
    recent_nonces: std::collections::VecDeque<(Uuid, u32, SystemTime)>,
    /// 最大历史记录数量
    max_history: usize,
    /// nonce有效期（用于清理过期记录）
    nonce_lifetime: Duration,
}

impl NonceValidator {
    /// 创建新的nonce验证器
    pub fn new(max_history: usize, nonce_lifetime: Duration) -> Self {
        Self {
            recent_nonces: std::collections::VecDeque::new(),
            max_history,
            nonce_lifetime,
        }
    }

    /// 验证nonce是否有效
    pub fn validate_nonce(&mut self, work: &Work, nonce: u32) -> Result<NonceValidationResult, String> {
        // 1. 基本范围检查
        if nonce == 0 {
            return Ok(NonceValidationResult::Invalid("Nonce cannot be zero".to_string()));
        }

        // 2. 检查重复nonce
        if self.is_duplicate_nonce(&work.id, nonce) {
            return Ok(NonceValidationResult::Duplicate);
        }

        // 3. 验证nonce是否满足工作目标
        let validation_result = self.verify_nonce_target(work, nonce)?;

        // 4. 如果验证通过，记录nonce
        if matches!(validation_result, NonceValidationResult::Valid) {
            self.record_nonce(work.id, nonce);
        }

        Ok(validation_result)
    }

    /// 检查是否为重复nonce
    fn is_duplicate_nonce(&self, work_id: &Uuid, nonce: u32) -> bool {
        self.recent_nonces.iter().any(|(id, n, _)| id == work_id && *n == nonce)
    }

    /// 验证nonce是否满足目标难度
    fn verify_nonce_target(&self, work: &Work, nonce: u32) -> Result<NonceValidationResult, String> {
        // 构建完整的区块头
        let mut header = work.header;

        // 将nonce写入区块头的最后4个字节
        header[76..80].copy_from_slice(&nonce.to_le_bytes());

        // 计算SHA256双重哈希
        let hash = self.double_sha256(&header);

        // 检查哈希是否满足目标
        if self.hash_meets_target(&hash, &work.target) {
            // 计算实际难度
            let _actual_difficulty = self.calculate_difficulty_from_hash(&hash);
            Ok(NonceValidationResult::Valid)
        } else {
            Ok(NonceValidationResult::Invalid("Hash does not meet target".to_string()))
        }
    }

    /// 记录有效的nonce
    fn record_nonce(&mut self, work_id: Uuid, nonce: u32) {
        let now = SystemTime::now();

        // 清理过期记录
        self.cleanup_expired_nonces(now);

        // 添加新记录
        self.recent_nonces.push_back((work_id, nonce, now));

        // 限制历史记录数量
        while self.recent_nonces.len() > self.max_history {
            self.recent_nonces.pop_front();
        }
    }

    /// 清理过期的nonce记录
    fn cleanup_expired_nonces(&mut self, now: SystemTime) {
        while let Some((_, _, timestamp)) = self.recent_nonces.front() {
            if now.duration_since(*timestamp).unwrap_or(Duration::from_secs(0)) > self.nonce_lifetime {
                self.recent_nonces.pop_front();
            } else {
                break;
            }
        }
    }

    /// 执行SHA256双重哈希
    fn double_sha256(&self, data: &[u8]) -> Vec<u8> {
        use sha2::{Sha256, Digest};
        let first_hash = Sha256::digest(data);
        let second_hash = Sha256::digest(first_hash);
        second_hash.to_vec()
    }

    /// 检查哈希是否满足目标
    fn hash_meets_target(&self, hash: &[u8], target: &[u8; 32]) -> bool {
        // Bitcoin使用小端序比较，从最高位开始比较
        for i in (0..32).rev() {
            match hash[i].cmp(&target[i]) {
                std::cmp::Ordering::Less => return true,
                std::cmp::Ordering::Greater => return false,
                std::cmp::Ordering::Equal => continue,
            }
        }
        true // 完全相等也算满足
    }

    /// 从哈希计算实际难度
    fn calculate_difficulty_from_hash(&self, hash: &[u8]) -> f64 {
        // 将哈希转换为大整数并计算难度
        // 这是一个简化的实现
        let mut hash_value = 0u64;
        for (i, &byte) in hash[24..32].iter().enumerate() {
            hash_value |= (byte as u64) << (i * 8);
        }

        if hash_value == 0 {
            return f64::MAX;
        }

        // 难度1的目标值
        let diff1_target = 0x00000000FFFF0000u64;
        diff1_target as f64 / hash_value as f64
    }

    /// 获取验证统计信息
    pub fn get_stats(&self) -> NonceValidatorStats {
        NonceValidatorStats {
            total_nonces_checked: self.recent_nonces.len(),
            history_size: self.recent_nonces.len(),
            max_history: self.max_history,
        }
    }
}

/// Nonce验证结果
#[derive(Debug, Clone, PartialEq)]
pub enum NonceValidationResult {
    /// 有效的nonce
    Valid,
    /// 无效的nonce
    Invalid(String),
    /// 重复的nonce
    Duplicate,
    /// 过期的工作
    Stale,
}

/// Nonce验证器统计信息
#[derive(Debug, Clone)]
pub struct NonceValidatorStats {
    pub total_nonces_checked: usize,
    pub history_size: usize,
    pub max_history: usize,
}

/// 算力类型
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct HashRate {
    /// 每秒哈希数
    pub hashes_per_second: f64,
}

impl HashRate {
    /// 创建新的算力
    pub fn new(hashes_per_second: f64) -> Self {
        Self { hashes_per_second }
    }

    /// 转换为 TH/s
    pub fn as_th_per_second(&self) -> f64 {
        self.hashes_per_second / 1_000_000_000_000.0
    }

    /// 转换为 GH/s
    pub fn as_gh_per_second(&self) -> f64 {
        self.hashes_per_second / 1_000_000_000.0
    }

    /// 转换为 MH/s
    pub fn as_mh_per_second(&self) -> f64 {
        self.hashes_per_second / 1_000_000.0
    }

    /// 从 TH/s 创建
    pub fn from_th_per_second(th_per_second: f64) -> Self {
        Self::new(th_per_second * 1_000_000_000_000.0)
    }

    /// 从 GH/s 创建
    pub fn from_gh_per_second(gh_per_second: f64) -> Self {
        Self::new(gh_per_second * 1_000_000_000.0)
    }

    /// 从 MH/s 创建
    pub fn from_mh_per_second(mh_per_second: f64) -> Self {
        Self::new(mh_per_second * 1_000_000.0)
    }
}

impl std::fmt::Display for HashRate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hashrate = self.hashes_per_second;

        // 智能选择最合适的单位，确保显示值在合理范围内（1-999）
        if hashrate >= 1_000_000_000_000.0 {
            let th_value = self.as_th_per_second();
            if th_value >= 100.0 {
                write!(f, "{:.1} TH/s", th_value)
            } else if th_value >= 10.0 {
                write!(f, "{:.2} TH/s", th_value)
            } else {
                write!(f, "{:.3} TH/s", th_value)
            }
        } else if hashrate >= 1_000_000_000.0 {
            let gh_value = self.as_gh_per_second();
            if gh_value >= 100.0 {
                write!(f, "{:.1} GH/s", gh_value)
            } else if gh_value >= 10.0 {
                write!(f, "{:.2} GH/s", gh_value)
            } else if gh_value >= 1.0 {
                write!(f, "{:.3} GH/s", gh_value)
            } else {
                // 如果GH值小于1，降级到MH
                let mh_value = self.as_mh_per_second();
                if mh_value >= 100.0 {
                    write!(f, "{:.1} MH/s", mh_value)
                } else if mh_value >= 10.0 {
                    write!(f, "{:.2} MH/s", mh_value)
                } else {
                    write!(f, "{:.3} MH/s", mh_value)
                }
            }
        } else if hashrate >= 1_000_000.0 {
            let mh_value = self.as_mh_per_second();
            if mh_value >= 100.0 {
                write!(f, "{:.1} MH/s", mh_value)
            } else if mh_value >= 10.0 {
                write!(f, "{:.2} MH/s", mh_value)
            } else if mh_value >= 1.0 {
                write!(f, "{:.3} MH/s", mh_value)
            } else {
                // 如果MH值小于1，降级到KH
                let kh_value = hashrate / 1_000.0;
                if kh_value >= 100.0 {
                    write!(f, "{:.1} KH/s", kh_value)
                } else if kh_value >= 10.0 {
                    write!(f, "{:.2} KH/s", kh_value)
                } else {
                    write!(f, "{:.3} KH/s", kh_value)
                }
            }
        } else if hashrate >= 1_000.0 {
            let kh_value = hashrate / 1_000.0;
            if kh_value >= 100.0 {
                write!(f, "{:.1} KH/s", kh_value)
            } else if kh_value >= 10.0 {
                write!(f, "{:.2} KH/s", kh_value)
            } else if kh_value >= 1.0 {
                write!(f, "{:.3} KH/s", kh_value)
            } else {
                // 如果KH值小于1，降级到H
                if hashrate >= 100.0 {
                    write!(f, "{:.1} H/s", hashrate)
                } else if hashrate >= 10.0 {
                    write!(f, "{:.2} H/s", hashrate)
                } else {
                    write!(f, "{:.3} H/s", hashrate)
                }
            }
        } else if hashrate >= 1.0 {
            if hashrate >= 100.0 {
                write!(f, "{:.1} H/s", hashrate)
            } else if hashrate >= 10.0 {
                write!(f, "{:.2} H/s", hashrate)
            } else {
                write!(f, "{:.3} H/s", hashrate)
            }
        } else {
            // 对于非常小的算力值，显示更高精度
            write!(f, "{:.6} H/s", hashrate)
        }
    }
}

/// 温度类型
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Temperature {
    /// 摄氏度
    pub celsius: f32,
}

impl Temperature {
    /// 创建新的温度
    pub fn new(celsius: f32) -> Self {
        Self { celsius }
    }

    /// 转换为华氏度
    pub fn as_fahrenheit(&self) -> f32 {
        self.celsius * 9.0 / 5.0 + 32.0
    }

    /// 从华氏度创建
    pub fn from_fahrenheit(fahrenheit: f32) -> Self {
        Self::new((fahrenheit - 32.0) * 5.0 / 9.0)
    }
}

impl std::fmt::Display for Temperature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.1}°C", self.celsius)
    }
}

/// 电压类型
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Voltage {
    /// 毫伏
    pub millivolts: u32,
}

impl Voltage {
    /// 创建新的电压
    pub fn new(millivolts: u32) -> Self {
        Self { millivolts }
    }

    /// 转换为伏特
    pub fn as_volts(&self) -> f32 {
        self.millivolts as f32 / 1000.0
    }

    /// 从伏特创建
    pub fn from_volts(volts: f32) -> Self {
        Self::new((volts * 1000.0) as u32)
    }
}

impl std::fmt::Display for Voltage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.3}V", self.as_volts())
    }
}

/// 频率类型
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Frequency {
    /// 兆赫兹
    pub megahertz: u32,
}

impl Frequency {
    /// 创建新的频率
    pub fn new(megahertz: u32) -> Self {
        Self { megahertz }
    }

    /// 转换为赫兹
    pub fn as_hertz(&self) -> u64 {
        self.megahertz as u64 * 1_000_000
    }

    /// 从赫兹创建
    pub fn from_hertz(hertz: u64) -> Self {
        Self::new((hertz / 1_000_000) as u32)
    }
}

impl std::fmt::Display for Frequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}MHz", self.megahertz)
    }
}

/// UUID转换工具
pub struct UuidConverter {
    /// UUID到数字ID的映射
    uuid_to_numeric: std::collections::HashMap<Uuid, u64>,
    /// 数字ID到UUID的映射
    numeric_to_uuid: std::collections::HashMap<u64, Uuid>,
    /// 下一个可用的数字ID
    next_numeric_id: u64,
}

impl UuidConverter {
    /// 创建新的UUID转换器
    pub fn new() -> Self {
        Self {
            uuid_to_numeric: std::collections::HashMap::new(),
            numeric_to_uuid: std::collections::HashMap::new(),
            next_numeric_id: 1,
        }
    }

    /// 将UUID转换为数字ID，如果不存在则创建新的映射
    pub fn uuid_to_numeric(&mut self, uuid: Uuid) -> u64 {
        if let Some(&numeric_id) = self.uuid_to_numeric.get(&uuid) {
            numeric_id
        } else {
            let numeric_id = self.next_numeric_id;
            self.next_numeric_id += 1;
            self.uuid_to_numeric.insert(uuid, numeric_id);
            self.numeric_to_uuid.insert(numeric_id, uuid);
            numeric_id
        }
    }

    /// 将数字ID转换为UUID，如果不存在则返回None
    pub fn numeric_to_uuid(&self, numeric_id: u64) -> Option<Uuid> {
        self.numeric_to_uuid.get(&numeric_id).copied()
    }

    /// 安全地从u64创建UUID（如果存在映射）或生成新的UUID
    pub fn safe_uuid_from_u64(&mut self, numeric_id: u64) -> Uuid {
        if let Some(uuid) = self.numeric_to_uuid(numeric_id) {
            uuid
        } else {
            // 如果没有映射，生成新的UUID并创建映射
            let uuid = Uuid::new_v4();
            self.uuid_to_numeric.insert(uuid, numeric_id);
            self.numeric_to_uuid.insert(numeric_id, uuid);
            uuid
        }
    }

    /// 清理旧的映射（可选的内存管理）
    pub fn cleanup_old_mappings(&mut self, max_entries: usize) {
        if self.uuid_to_numeric.len() > max_entries {
            // 简单的清理策略：保留最近的一半映射
            let mut entries: Vec<_> = self.uuid_to_numeric.iter().map(|(k, v)| (*k, *v)).collect();
            entries.sort_by_key(|(_, numeric_id)| *numeric_id);

            let keep_count = max_entries / 2;
            let to_remove: Vec<_> = entries.iter().take(entries.len() - keep_count).cloned().collect();

            for (uuid, numeric_id) in to_remove {
                self.uuid_to_numeric.remove(&uuid);
                self.numeric_to_uuid.remove(&numeric_id);
            }
        }
    }
}

impl Default for UuidConverter {
    fn default() -> Self {
        Self::new()
    }
}

/// 全局UUID转换器实例（线程安全）
static UUID_CONVERTER: std::sync::LazyLock<std::sync::Mutex<UuidConverter>> =
    std::sync::LazyLock::new(|| std::sync::Mutex::new(UuidConverter::new()));

/// 安全地将u64转换为UUID
pub fn safe_uuid_from_u64(numeric_id: u64) -> Uuid {
    UUID_CONVERTER.lock().unwrap().safe_uuid_from_u64(numeric_id)
}

/// 将UUID转换为u64
pub fn uuid_to_u64(uuid: Uuid) -> u64 {
    UUID_CONVERTER.lock().unwrap().uuid_to_numeric(uuid)
}

/// 尝试将u64转换为已知的UUID
pub fn try_uuid_from_u64(numeric_id: u64) -> Option<Uuid> {
    UUID_CONVERTER.lock().unwrap().numeric_to_uuid(numeric_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_merkle_root_calculation() {
        // 创建一个测试Work
        let mut work = Work::new(
            "test_job".to_string(),
            [0u8; 32], // target
            [0u8; 80], // header
            1.0,       // difficulty
        );

        // 设置测试数据
        work.coinbase1 = hex::decode("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff").unwrap();
        work.coinbase2 = hex::decode("ffffffff01").unwrap();
        work.extranonce1 = hex::decode("f2b944e0").unwrap();
        work.extranonce2 = hex::decode("00000000").unwrap();

        // 添加一个测试merkle分支
        work.merkle_branches = vec![
            hex::decode("3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a").unwrap()
        ];

        // 计算merkle root
        let result = work.calculate_merkle_root();
        assert!(result.is_ok(), "Merkle root calculation should succeed");

        // 验证merkle root不为空
        assert_ne!(work.merkle_root, [0u8; 32], "Merkle root should not be zero");

        // 验证区块头中的merkle root已更新
        assert_eq!(work.header[36..68], work.merkle_root, "Header merkle root should match calculated value");
    }

    #[test]
    fn test_double_sha256() {
        let test_data = b"hello world";
        let result = Work::double_sha256(test_data);

        // 验证结果长度
        assert_eq!(result.len(), 32, "Double SHA256 should produce 32 bytes");

        // 验证结果不为空
        assert_ne!(result, [0u8; 32], "Double SHA256 should not produce zero hash");
    }

    #[test]
    fn test_coinbase_building() {
        let mut work = Work::new(
            "test_job".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0,
        );

        work.coinbase1 = vec![1, 2, 3];
        work.extranonce1 = vec![4, 5];
        work.extranonce2 = vec![6, 7];
        work.coinbase2 = vec![8, 9, 10];

        let coinbase = work.build_coinbase();
        let expected = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        assert_eq!(coinbase, expected, "Coinbase should be correctly concatenated");
    }

    #[test]
    fn test_nbits_to_target() {
        // 测试难度1的nBits (0x1d00ffff)
        let target = Work::nbits_to_target(0x1d00ffff);

        // 验证目标值不为空
        assert_ne!(target, [0u8; 32], "Target should not be zero");

        // 验证目标值的前几个字节
        assert_eq!(target[0], 0x00, "Target should start with zeros for high difficulty");
    }

    #[test]
    fn test_work_expiration() {
        let work = Work::new(
            "test_job".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0,
        );

        // 新创建的工作不应该过期
        assert!(!work.is_expired(), "Newly created work should not be expired");

        // 测试自定义过期时间
        assert!(!work.is_expired_with_max_age(Duration::from_secs(300)), "Work should not be expired with 5 minute max age");
        assert!(work.is_expired_with_max_age(Duration::from_millis(1)), "Work should be expired with 1ms max age");
    }

    #[test]
    fn test_coinbase_validation() {
        let mut work = Work::new(
            "test_job".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0,
        );

        // 测试空coinbase1的情况
        work.coinbase1 = vec![];
        assert!(work.validate_coinbase().is_err(), "Empty coinbase1 should fail validation");

        // 设置有效的coinbase数据
        work.coinbase1 = hex::decode("01000000010000000000000000000000000000000000000000000000000000000000000000ffffffff4d04").unwrap();
        work.coinbase2 = hex::decode("0a636b706f6f6c122f4249542e434f4d2f4e656f5363727970742fffffffff01").unwrap();
        work.extranonce1 = hex::decode("f2b944e0").unwrap();
        work.extranonce2 = vec![0, 0, 0, 0];
        work.extranonce2_size = 4;

        // 现在验证应该通过
        assert!(work.validate_coinbase().is_ok(), "Valid coinbase should pass validation");

        // 测试extranonce2长度不匹配
        work.extranonce2 = vec![0, 0]; // 长度为2，但期望4
        assert!(work.validate_coinbase().is_err(), "Mismatched extranonce2 length should fail validation");
    }

    #[test]
    fn test_coinbase_hash() {
        let mut work = Work::new(
            "test_job".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0,
        );

        work.coinbase1 = vec![1, 2, 3];
        work.extranonce1 = vec![4, 5];
        work.extranonce2 = vec![6, 7];
        work.coinbase2 = vec![8, 9, 10];

        let hash = work.get_coinbase_hash();

        // 验证哈希长度
        assert_eq!(hash.len(), 32, "Coinbase hash should be 32 bytes");

        // 验证哈希不为空
        assert_ne!(hash, [0u8; 32], "Coinbase hash should not be zero");
    }

    #[test]
    fn test_nonce_validator() {
        let mut validator = NonceValidator::new(100, Duration::from_secs(300));

        let work = Work::new(
            "test_job".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0,
        );

        // 测试基本nonce验证
        let result = validator.validate_nonce(&work, 0);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), NonceValidationResult::Invalid(_)));

        // 测试有效nonce
        let result = validator.validate_nonce(&work, 12345);
        assert!(result.is_ok());

        // 测试重复nonce检测
        let result1 = validator.validate_nonce(&work, 54321);
        assert!(result1.is_ok());

        let result2 = validator.validate_nonce(&work, 54321);
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), NonceValidationResult::Duplicate);

        // 测试统计信息
        let stats = validator.get_stats();
        assert!(stats.history_size > 0);
        assert_eq!(stats.max_history, 100);
    }

    #[test]
    fn test_nonce_validation_result() {
        // 测试验证结果的比较
        assert_eq!(NonceValidationResult::Valid, NonceValidationResult::Valid);
        assert_ne!(NonceValidationResult::Valid, NonceValidationResult::Duplicate);

        let invalid1 = NonceValidationResult::Invalid("reason1".to_string());
        let invalid2 = NonceValidationResult::Invalid("reason2".to_string());
        assert_ne!(invalid1, invalid2);
    }

    #[test]
    fn test_nonce_validator_cleanup() {
        let mut validator = NonceValidator::new(10, Duration::from_millis(100));

        let work = Work::new(
            "test_job".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0,
        );

        // 添加一些nonce
        for i in 1..=5 {
            let _ = validator.validate_nonce(&work, i);
        }

        assert_eq!(validator.get_stats().history_size, 5);

        // 等待过期
        std::thread::sleep(Duration::from_millis(150));

        // 添加新nonce应该触发清理
        let _ = validator.validate_nonce(&work, 100);

        // 过期的记录应该被清理
        let stats = validator.get_stats();
        assert!(stats.history_size <= 1); // 只有最新的nonce
    }
}
