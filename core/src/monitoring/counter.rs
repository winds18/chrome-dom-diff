//! # Counter（计数器）统计
//!
//! 单调递增的计数器，用于累计操作次数。

use std::sync::atomic::{AtomicU64, Ordering};

/// 计数器
///
/// ## 线程安全
///
/// 使用原子操作，支持多线程环境。
#[derive(Debug)]
pub struct Counter {
    name: String,
    value: AtomicU64,
}

// 手动实现 Clone，因为 AtomicU64 不支持 Clone
impl Clone for Counter {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            value: AtomicU64::new(self.value.load(Ordering::Relaxed)),
        }
    }
}

impl Counter {
    /// 创建新的计数器
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            value: AtomicU64::new(0),
        }
    }

    /// 增加计数
    pub fn inc(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// 增加指定值
    pub fn inc_by(&self, value: u64) {
        self.value.fetch_add(value, Ordering::Relaxed);
    }

    /// 获取当前值
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }

    /// 重置计数器
    pub fn reset(&self) {
        self.value.store(0, Ordering::Relaxed);
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counter_basic() {
        let counter = Counter::new("test");

        assert_eq!(counter.value(), 0);
        counter.inc();
        assert_eq!(counter.value(), 1);
        counter.inc_by(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_counter_reset() {
        let counter = Counter::new("test");

        counter.inc();
        counter.inc();
        assert_eq!(counter.value(), 2);

        counter.reset();
        assert_eq!(counter.value(), 0);
    }
}
