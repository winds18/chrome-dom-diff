//! # Histogram（直方图）统计
//!
//! 用于收集和分析数值分布数据，如延迟、大小等。

use super::HistogramStats;
use std::collections::VecDeque;

/// 直方图统计器
///
/// 使用固定数量的桶来近似分布，避免存储所有样本。
#[derive(Debug, Clone)]
pub struct Histogram {
    name: String,
    count: u64,
    sum: f64,
    min: f64,
    max: f64,
    // 使用指数分级桶来近似分布
    buckets: Vec<Bucket>,
}

#[derive(Debug, Clone)]
struct Bucket {
    upper_bound: f64,
    count: u64,
}

impl Histogram {
    /// 创建新的 Histogram
    ///
    /// 使用默认的桶配置（适用于微秒级延迟）。
    pub fn new(name: &str) -> Self {
        Self::with_buckets(name, &default_buckets())
    }

    /// 使用自定义桶创建
    pub fn with_buckets(name: &str, buckets: &[f64]) -> Self {
        Self {
            name: name.to_string(),
            count: 0,
            sum: 0.0,
            min: f64::MAX,
            max: f64::MIN,
            buckets: buckets
                .iter()
                .map(|&upper| Bucket {
                    upper_bound: upper,
                    count: 0,
                })
                .collect(),
        }
    }

    /// 记录观测值
    #[inline]
    pub fn observe(&mut self, value: f64) {
        self.count += 1;
        self.sum += value;
        self.min = self.min.min(value);
        self.max = self.max.max(value);

        // 找到合适的桶
        for bucket in &mut self.buckets {
            if value <= bucket.upper_bound {
                bucket.count += 1;
                return;
            }
        }
    }

    /// 获取样本数量
    #[inline]
    pub fn count(&self) -> u64 {
        self.count
    }

    /// 获取总和
    #[inline]
    pub fn sum(&self) -> f64 {
        self.sum
    }

    /// 获取平均值
    #[inline]
    pub fn avg(&self) -> f64 {
        if self.count > 0 {
            self.sum / self.count as f64
        } else {
            0.0
        }
    }

    /// 获取最小值
    #[inline]
    pub fn min(&self) -> f64 {
        if self.count > 0 {
            self.min
        } else {
            0.0
        }
    }

    /// 获取最大值
    #[inline]
    pub fn max(&self) -> f64 {
        self.max
    }

    /// 计算百分位数
    ///
    /// 使用线性插值从桶中估算。
    pub fn percentile(&self, p: f64) -> f64 {
        if self.count == 0 {
            return 0.0;
        }

        let target = (p / 100.0 * self.count as f64).ceil() as u64;
        let mut cumulative = 0;

        for (i, bucket) in self.buckets.iter().enumerate() {
            cumulative += bucket.count;
            if cumulative >= target {
                // 线性插值
                let lower_bound = if i > 0 {
                    self.buckets[i - 1].upper_bound
                } else {
                    0.0
                };
                return (lower_bound + bucket.upper_bound) / 2.0;
            }
        }

        self.buckets.last().map(|b| b.upper_bound).unwrap_or(0.0)
    }

    /// 获取 P50（中位数）
    #[inline]
    pub fn p50(&self) -> f64 {
        self.percentile(50.0)
    }

    /// 获取 P95
    #[inline]
    pub fn p95(&self) -> f64 {
        self.percentile(95.0)
    }

    /// 获取 P99
    #[inline]
    pub fn p99(&self) -> f64 {
        self.percentile(99.0)
    }

    /// 获取 P99.9
    #[inline]
    pub fn p999(&self) -> f64 {
        self.percentile(99.9)
    }

    /// 获取完整统计
    pub fn stats(&self) -> HistogramStats {
        HistogramStats {
            count: self.count,
            sum: self.sum,
            min: self.min(),
            max: self.max,
            avg: self.avg(),
            p50: self.p50(),
            p95: self.p95(),
            p99: self.p99(),
            p999: self.p999(),
        }
    }

    /// 重置统计
    pub fn reset(&mut self) {
        self.count = 0;
        self.sum = 0.0;
        self.min = f64::MAX;
        self.max = f64::MIN;
        for bucket in &mut self.buckets {
            bucket.count = 0;
        }
    }

    /// 获取名称
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl Default for Histogram {
    fn default() -> Self {
        Self::new("")
    }
}

/// 默认桶配置（微秒级）
///
/// 桶边界：1, 5, 10, 25, 50, 100, 250, 500, 1000, 2500, 5000, 10000, 30000, 60000, ...
fn default_buckets() -> Vec<f64> {
    vec![
        1.0,      // 1 微秒
        5.0,      // 5 微秒
        10.0,     // 10 微秒
        25.0,     // 25 微秒
        50.0,     // 50 微秒
        100.0,    // 100 微秒
        250.0,    // 250 微秒
        500.0,    // 500 微秒
        1000.0,   // 1 毫秒
        2500.0,   // 2.5 毫秒
        5000.0,   // 5 毫秒
        10000.0,  // 10 毫秒
        30000.0,  // 30 毫秒
        60000.0,  // 60 毫秒
        120000.0, // 120 毫秒
        300000.0, // 300 毫秒
        600000.0, // 600 毫秒
        f64::MAX, // > 600 毫秒
    ]
}

/// 延迟专用桶（微秒）
pub fn latency_buckets_us() -> Vec<f64> {
    default_buckets()
}

/// 字节数专用桶
pub fn size_buckets() -> Vec<f64> {
    vec![
        1.0,      // 1 字节
        10.0,     // 10 字节
        100.0,    // 100 字节
        1024.0,   // 1 KB
        10240.0,  // 10 KB
        102400.0, // 100 KB
        f64::MAX,
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_histogram_basic() {
        let mut hist = Histogram::new("test");

        hist.observe(1.0);
        hist.observe(2.0);
        hist.observe(3.0);

        assert_eq!(hist.count(), 3);
        assert_eq!(hist.sum(), 6.0);
        assert_eq!(hist.avg(), 2.0);
        assert_eq!(hist.min(), 1.0);
        assert_eq!(hist.max(), 3.0);
    }

    #[test]
    fn test_histogram_percentiles() {
        let mut hist = Histogram::new("test");

        // 100 个样本，均匀分布 1-100
        for i in 1..=100 {
            hist.observe(i as f64);
        }

        assert_eq!(hist.p50(), 50.0);
        assert_eq!(hist.p95(), 95.0);
        assert_eq!(hist.p99(), 99.0);
    }
}
