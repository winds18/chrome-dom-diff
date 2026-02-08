//! # Gauge（仪表）统计
//!
//! 可增减的瞬时值指标，用于记录当前状态。

use std::sync::atomic::{AtomicU64, Ordering};

/// 仪表
///
/// ## 线程安全
///
/// 使用原子操作，支持多线程环境。
#[derive(Debug)]
pub struct Gauge {
    name: String,
    /// 使用 u64 存储 f64 的比特位
    bits: AtomicU64,
}

// 手动实现 Clone，因为 AtomicU64 不支持 Clone
impl Clone for Gauge {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            bits: AtomicU64::new(self.bits.load(Ordering::Relaxed)),
        }
    }
}

impl Gauge {
    /// 创建新的仪表
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            bits: AtomicU64::new(0),
        }
    }

    /// 设置值
    pub fn set(&self, value: f64) {
        self.bits.store(value.to_bits(), Ordering::Relaxed);
    }

    /// 增加值
    pub fn add(&self, delta: f64) {
        let current = self.bits.load(Ordering::Relaxed);
        let current_f = f64::from_bits(current);
        self.bits.store((current_f + delta).to_bits(), Ordering::Relaxed);
    }

    /// 减少值
    pub fn sub(&self, delta: f64) {
        let current = self.bits.load(Ordering::Relaxed);
        let current_f = f64::from_bits(current);
        self.bits.store((current_f - delta).to_bits(), Ordering::Relaxed);
    }

    /// 获取当前值
    pub fn value(&self) -> f64 {
        f64::from_bits(self.bits.load(Ordering::Relaxed))
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
    fn test_gauge_basic() {
        let gauge = Gauge::new("test");

        assert_eq!(gauge.value(), 0.0);
        gauge.set(42.0);
        assert_eq!(gauge.value(), 42.0);
        gauge.add(8.0);
        assert_eq!(gauge.value(), 50.0);
        gauge.sub(10.0);
        assert_eq!(gauge.value(), 40.0);
    }

    #[test]
    fn test_gauge_negative() {
        let gauge = Gauge::new("test");

        gauge.set(-10.0);
        assert_eq!(gauge.value(), -10.0);
        gauge.add(5.0);
        assert_eq!(gauge.value(), -5.0);
    }
}
