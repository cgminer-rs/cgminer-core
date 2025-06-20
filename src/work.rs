//! 工作管理模块

use crate::error::CoreError;
use crate::types::{Work, MiningResult};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::Notify;
use tracing::{debug, warn};
use uuid::Uuid;

/// 工作队列
pub struct WorkQueue {
    /// 待处理的工作
    pending_work: Arc<RwLock<VecDeque<Work>>>,
    /// 正在处理的工作
    active_work: Arc<RwLock<HashMap<Uuid, Work>>>,
    /// 已完成的工作
    completed_work: Arc<RwLock<HashMap<Uuid, MiningResult>>>,
    /// 最大队列大小
    max_queue_size: usize,
    /// 工作过期时间
    work_expiry: Duration,
    /// 通知器
    notify: Arc<Notify>,
}

impl WorkQueue {
    /// 创建新的工作队列
    pub fn new(max_queue_size: usize, work_expiry: Duration) -> Self {
        Self {
            pending_work: Arc::new(RwLock::new(VecDeque::new())),
            active_work: Arc::new(RwLock::new(HashMap::new())),
            completed_work: Arc::new(RwLock::new(HashMap::new())),
            max_queue_size,
            work_expiry,
            notify: Arc::new(Notify::new()),
        }
    }

    /// 添加工作到队列
    pub fn add_work(&self, work: Work) -> Result<(), CoreError> {
        let mut pending = self.pending_work.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        // 检查队列是否已满
        if pending.len() >= self.max_queue_size {
            // 移除最旧的工作
            if let Some(old_work) = pending.pop_front() {
                warn!("工作队列已满，移除旧工作: {}", old_work.id);
            }
        }

        debug!("添加工作到队列: {}", work.id);
        pending.push_back(work);

        // 通知等待的任务
        self.notify.notify_one();
        Ok(())
    }

    /// 获取下一个工作
    pub fn get_next_work(&self) -> Result<Option<Work>, CoreError> {
        let mut pending = self.pending_work.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        // 清理过期的工作
        self.cleanup_expired_work(&mut pending)?;

        if let Some(work) = pending.pop_front() {
            // 将工作移动到活跃列表
            let mut active = self.active_work.write().map_err(|e| {
                CoreError::runtime(format!("Failed to acquire write lock: {}", e))
            })?;

            active.insert(work.id, work.clone());
            debug!("获取工作: {}", work.id);
            Ok(Some(work))
        } else {
            Ok(None)
        }
    }

    /// 等待新工作
    pub async fn wait_for_work(&self) -> Result<Option<Work>, CoreError> {
        loop {
            // 尝试获取工作
            if let Some(work) = self.get_next_work()? {
                return Ok(Some(work));
            }

            // 等待新工作通知
            self.notify.notified().await;
        }
    }

    /// 标记工作完成
    pub fn complete_work(&self, work_id: Uuid, result: MiningResult) -> Result<(), CoreError> {
        // 从活跃列表中移除
        let mut active = self.active_work.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        if active.remove(&work_id).is_some() {
            // 添加到完成列表
            let mut completed = self.completed_work.write().map_err(|e| {
                CoreError::runtime(format!("Failed to acquire write lock: {}", e))
            })?;

            completed.insert(work_id, result);
            debug!("工作完成: {}", work_id);
            Ok(())
        } else {
            Err(CoreError::runtime(format!("工作 {} 不在活跃列表中", work_id)))
        }
    }

    /// 获取队列统计信息
    pub fn get_stats(&self) -> Result<WorkQueueStats, CoreError> {
        let pending = self.pending_work.read().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire read lock: {}", e))
        })?;

        let active = self.active_work.read().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire read lock: {}", e))
        })?;

        let completed = self.completed_work.read().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire read lock: {}", e))
        })?;

        Ok(WorkQueueStats {
            pending_count: pending.len(),
            active_count: active.len(),
            completed_count: completed.len(),
            max_queue_size: self.max_queue_size,
        })
    }

    /// 清理过期的工作
    fn cleanup_expired_work(&self, pending: &mut VecDeque<Work>) -> Result<(), CoreError> {
        let _now = SystemTime::now();
        let mut expired_count = 0;

        // 从前面开始移除过期的工作
        while let Some(work) = pending.front() {
            if work.is_expired_with_max_age(self.work_expiry) {
                pending.pop_front();
                expired_count += 1;
            } else {
                break;
            }
        }

        if expired_count > 0 {
            debug!("清理了 {} 个过期工作", expired_count);
        }

        Ok(())
    }

    /// 清空队列
    pub fn clear(&self) -> Result<(), CoreError> {
        let mut pending = self.pending_work.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        let mut active = self.active_work.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        let mut completed = self.completed_work.write().map_err(|e| {
            CoreError::runtime(format!("Failed to acquire write lock: {}", e))
        })?;

        pending.clear();
        active.clear();
        completed.clear();

        debug!("工作队列已清空");
        Ok(())
    }
}

/// 工作管理器
pub struct WorkManager {
    /// 工作队列
    work_queue: WorkQueue,
    /// 工作ID计数器
    work_id_counter: Arc<RwLock<u64>>,
}

impl WorkManager {
    /// 创建新的工作管理器
    pub fn new(max_queue_size: usize, work_expiry: Duration) -> Self {
        Self {
            work_queue: WorkQueue::new(max_queue_size, work_expiry),
            work_id_counter: Arc::new(RwLock::new(0)),
        }
    }

    /// 创建新工作
    pub fn create_work(&self, job_id: String, header: [u8; 80], target: [u8; 32], difficulty: f64) -> Result<Work, CoreError> {
        // 更新计数器（用于统计）
        let _work_id = {
            let mut counter = self.work_id_counter.write().map_err(|e| {
                CoreError::runtime(format!("Failed to acquire write lock: {}", e))
            })?;
            *counter += 1;
            *counter
        };

        Ok(Work::new(job_id, target, header, difficulty))
    }

    /// 提交工作
    pub fn submit_work(&self, work: Work) -> Result<(), CoreError> {
        self.work_queue.add_work(work)
    }

    /// 获取下一个工作
    pub fn get_next_work(&self) -> Result<Option<Work>, CoreError> {
        self.work_queue.get_next_work()
    }

    /// 等待新工作
    pub async fn wait_for_work(&self) -> Result<Option<Work>, CoreError> {
        self.work_queue.wait_for_work().await
    }

    /// 完成工作
    pub fn complete_work(&self, work_id: Uuid, result: MiningResult) -> Result<(), CoreError> {
        self.work_queue.complete_work(work_id, result)
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> Result<WorkQueueStats, CoreError> {
        self.work_queue.get_stats()
    }

    /// 清空所有工作
    pub fn clear_all(&self) -> Result<(), CoreError> {
        self.work_queue.clear()
    }
}

/// 工作队列统计信息
#[derive(Debug, Clone)]
pub struct WorkQueueStats {
    /// 待处理工作数量
    pub pending_count: usize,
    /// 活跃工作数量
    pub active_count: usize,
    /// 已完成工作数量
    pub completed_count: usize,
    /// 最大队列大小
    pub max_queue_size: usize,
}
