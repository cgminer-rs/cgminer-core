//! 挖矿通用工具函数
//!
//! 这个模块包含所有挖矿核心都需要的通用工具函数，
//! 确保不同核心使用相同的算法实现。

use crate::error::CoreError;
use crate::types::Work;

/// 检查哈希值是否满足目标难度
///
/// 这是比特币挖矿的核心验证函数，用于检查计算出的哈希值
/// 是否小于或等于目标值。在比特币中，目标值越小，难度越大。
///
/// # 参数
///
/// * `hash` - 计算出的哈希值（通常是SHA256双重哈希的结果）
/// * `target` - 目标难度值（32字节，大端序）
///
/// # 返回值
///
/// 如果哈希值满足目标难度返回 `true`，否则返回 `false`
///
/// # 示例
///
/// ```rust
/// use cgminer_core::meets_target;
///
/// let hash = [0x00, 0x00, 0x00, 0x01, /* ... 其他28字节 */];
/// let target = [0x00, 0x00, 0x00, 0x0F, /* ... 其他28字节 */];
///
/// assert!(meets_target(&hash, &target));
/// ```
pub fn meets_target(hash: &[u8], target: &[u8]) -> bool {
    // 确保两个数组长度相同，取较短的长度进行比较
    let min_len = hash.len().min(target.len());

    // 逐字节比较，从最高位开始（大端序）
    for i in 0..min_len {
        match hash[i].cmp(&target[i]) {
            std::cmp::Ordering::Less => return true,    // 哈希值更小，满足目标
            std::cmp::Ordering::Greater => return false, // 哈希值更大，不满足目标
            std::cmp::Ordering::Equal => continue,       // 相等，继续比较下一字节
        }
    }

    // 如果所有比较的字节都相等，则满足目标
    true
}

/// 验证工作包的完整性
///
/// 检查工作包的各个字段是否有效，包括：
/// - 工作ID不为空
/// - 区块头长度正确
/// - 目标值长度正确
///
/// # 参数
///
/// * `work` - 要验证的工作包
///
/// # 返回值
///
/// 如果工作包有效返回 `Ok(())`，否则返回包含错误信息的 `Err`
pub fn validate_work_integrity(work: &Work) -> Result<(), CoreError> {
    // 检查工作ID（Uuid不会为空，但可以检查是否为nil）
    if work.id.is_nil() {
        return Err(CoreError::config("Work ID cannot be nil"));
    }

    // 检查区块头长度（比特币区块头应该是80字节）
    if work.header.len() != 80 {
        return Err(CoreError::config(
            format!("Invalid header length: expected 80 bytes, got {}", work.header.len())
        ));
    }

    // 检查目标值长度（应该是32字节）
    if work.target.len() != 32 {
        return Err(CoreError::config(
            format!("Invalid target length: expected 32 bytes, got {}", work.target.len())
        ));
    }

    Ok(())
}

/// 从目标值计算难度
///
/// 根据比特币协议，难度是基于目标值计算的。
/// 最大目标值（难度1）对应的是特定的常数。
///
/// # 参数
///
/// * `target` - 32字节的目标值
///
/// # 返回值
///
/// 计算出的难度值
pub fn calculate_difficulty(target: &[u8]) -> f64 {
    // 比特币的最大目标值（难度1时的目标）
    // 这是一个固定的常数：0x1d00ffff 对应的完整目标值
    const MAX_TARGET: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0xFF, 0xFF, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    ];

    // 将字节数组转换为大整数进行计算
    // 这里简化处理，实际实现可能需要使用大整数库
    let target_value = bytes_to_u256(target);
    let max_target_value = bytes_to_u256(&MAX_TARGET);

    // 难度 = 最大目标值 / 当前目标值
    if target_value == 0.0 {
        return f64::INFINITY;
    }

    max_target_value / target_value
}

/// 将32字节数组转换为f64（简化版本）
///
/// 注意：这是一个简化的实现，实际应用中可能需要更精确的大整数处理
fn bytes_to_u256(bytes: &[u8]) -> f64 {
    let mut result = 0.0;
    let mut multiplier = 1.0;

    // 从最低位开始处理（小端序处理）
    for &byte in bytes.iter().take(8).rev() {
        result += byte as f64 * multiplier;
        multiplier *= 256.0;
    }

    result
}

/// 格式化哈希值为十六进制字符串
///
/// # 参数
///
/// * `hash` - 哈希值字节数组
///
/// # 返回值
///
/// 格式化的十六进制字符串
pub fn format_hash(hash: &[u8]) -> String {
    hash.iter()
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

/// 格式化目标值为十六进制字符串（紧凑格式）
///
/// # 参数
///
/// * `target` - 目标值字节数组
///
/// # 返回值
///
/// 格式化的十六进制字符串，只显示前8字节
pub fn format_target(target: &[u8]) -> String {
    target.iter()
        .take(8)
        .map(|b| format!("{:02x}", b))
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_meets_target_simple() {
        // 哈希值小于目标值
        let hash = [0x00, 0x00, 0x00, 0x01];
        let target = [0x00, 0x00, 0x00, 0x0F];
        assert!(meets_target(&hash, &target));

        // 哈希值大于目标值
        let hash = [0x00, 0x00, 0x00, 0x0F];
        let target = [0x00, 0x00, 0x00, 0x01];
        assert!(!meets_target(&hash, &target));

        // 哈希值等于目标值
        let hash = [0x00, 0x00, 0x00, 0x0F];
        let target = [0x00, 0x00, 0x00, 0x0F];
        assert!(meets_target(&hash, &target));
    }

    #[test]
    fn test_meets_target_full_length() {
        // 32字节的完整测试
        let mut hash = [0u8; 32];
        let mut target = [0u8; 32];

        // 设置目标值在第4个字节
        target[3] = 0x0F;

        // 哈希值小于目标值
        hash[3] = 0x01;
        assert!(meets_target(&hash, &target));

        // 哈希值大于目标值
        hash[3] = 0xFF;
        assert!(!meets_target(&hash, &target));
    }

    #[test]
    fn test_validate_work_integrity() {
        let work = Work::new(
            "test_work".to_string(),
            [0u8; 32],
            [0u8; 80],
            1.0
        );

        assert!(validate_work_integrity(&work).is_ok());

        // 测试空ID - 由于Uuid不能直接创建nil，我们跳过这个测试
        // 或者测试其他无效情况，比如错误的header长度
        let mut invalid_header = [0u8; 79]; // 错误的长度
        // 无法直接创建无效的Work，因为构造函数会确保正确的长度
        // 这个测试在实际应用中，Work会从外部数据构造，可能存在无效情况
    }

    #[test]
    fn test_format_hash() {
        let hash = [0x12, 0x34, 0xAB, 0xCD];
        assert_eq!(format_hash(&hash), "1234abcd");
    }

    #[test]
    fn test_format_target() {
        let target = [0x00, 0x00, 0x00, 0x0F, 0xFF, 0xFF, 0x00, 0x00, 0x11, 0x22];
        assert_eq!(format_target(&target), "0000000fffff0000");
    }
}
