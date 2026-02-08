//! # 性能监控模块
//!
//! 提供实时性能指标收集和告警功能。
//!
//! ## 统计类型
//!
//! - **Histogram**: 分布统计（延迟、大小等）
//! - **Counter**: 单调递增计数（操作次数、错误次数）
//! - **Gauge**: 瞬时值（内存使用、活跃连接）
//!
//! ## 使用示例
//!
//! ```rust
//! use chrome_dom_diff::monitoring::{PerfMonitor, histogram, counter, gauge};
//!
//! // 获取全局监控实例
//! let monitor = PerfMonitor::global();
//!
//! // 记录延迟（微秒）
//! monitor.histogram("dom_capture_us").observe(1234.0);
//!
//! // 增加计数
//! monitor.counter("operations_total").inc();
//!
//! // 设置瞬时值
//! monitor.gauge("memory_mb").set(42.0);
//!
//! // 检查阈值
//! if monitor.check_thresholds() {
//!     // 有指标超过阈值
//! }
//! ```

pub mod histogram;
pub mod counter;
pub mod gauge;
pub mod threshold;

use std::sync::OnceLock;
use std::collections::HashMap;
use std::sync::RwLock;

pub use histogram::Histogram;
pub use counter::Counter;
pub use gauge::Gauge;
pub use threshold::{Threshold, ThresholdLevel};

/// 全局性能监控实例
static GLOBAL_MONITOR: OnceLock<PerfMonitor> = OnceLock::new();

/// 获取全局性能监控实例
#[inline]
pub fn global() -> &'static PerfMonitor {
    GLOBAL_MONITOR.get_or_init(|| PerfMonitor::new())
}

/// 性能监控器
///
/// 收集和管理所有性能指标。
pub struct PerfMonitor {
    histograms: RwLock<HashMap<String, Histogram>>,
    counters: RwLock<HashMap<String, Counter>>,
    gauges: RwLock<HashMap<String, Gauge>>,
    thresholds: RwLock<Vec<Threshold>>,
}

impl PerfMonitor {
    /// 创建新的性能监控器
    pub fn new() -> Self {
        Self {
            histograms: RwLock::new(HashMap::new()),
            counters: RwLock::new(HashMap::new()),
            gauges: RwLock::new(HashMap::new()),
            thresholds: RwLock::new(Vec::new()),
        }
    }

    /// 获取全局实例
    #[inline]
    pub fn global() -> &'static PerfMonitor {
        global()
    }

    // ============================================
    // Histogram 操作
    // ============================================

    /// 获取或创建 Histogram
    #[inline]
    pub fn histogram(&self, name: &str) -> Histogram {
        let mut histograms = self.histograms.write().unwrap();
        histograms
            .entry(name.to_string())
            .or_insert_with(|| Histogram::new(name))
            .clone()
    }

    /// 记录观测值
    #[inline]
    pub fn observe(&self, name: &str, value: f64) {
        if let Ok(mut histograms) = self.histograms.write() {
            if let Some(hist) = histograms.get_mut(name) {
                hist.observe(value);
            } else {
                let mut hist = Histogram::new(name);
                hist.observe(value);
                histograms.insert(name.to_string(), hist);
            }
        }
    }

    /// 记录延迟（微秒）
    #[inline]
    pub fn record_latency_us(&self, name: &str, latency_us: f64) {
        self.observe(name, latency_us);
    }

    /// 记录延迟（毫秒）
    #[inline]
    pub fn record_latency_ms(&self, name: &str, latency_ms: f64) {
        self.observe(name, latency_ms * 1000.0);
    }

    /// 获取 Histogram 统计
    pub fn histogram_stats(&self, name: &str) -> Option<HistogramStats> {
        let histograms = self.histograms.read().ok()?;
        histograms.get(name).map(|h| h.stats())
    }

    // ============================================
    // Counter 操作
    // ============================================

    /// 获取或创建 Counter
    #[inline]
    pub fn counter(&self, name: &str) -> Counter {
        let mut counters = self.counters.write().unwrap();
        counters
            .entry(name.to_string())
            .or_insert_with(|| Counter::new(name))
            .clone()
    }

    /// 增加计数
    #[inline]
    pub fn inc_counter(&self, name: &str) {
        if let Ok(mut counters) = self.counters.write() {
            if let Some(counter) = counters.get_mut(name) {
                counter.inc();
            } else {
                let counter = Counter::new(name);
                counters.insert(name.to_string(), counter);
            }
        }
    }

    /// 增加指定值
    #[inline]
    pub fn inc_counter_by(&self, name: &str, value: u64) {
        if let Ok(mut counters) = self.counters.write() {
            if let Some(counter) = counters.get_mut(name) {
                counter.inc_by(value);
            } else {
                let mut counter = Counter::new(name);
                counter.inc_by(value);
                counters.insert(name.to_string(), counter);
            }
        }
    }

    /// 获取 Counter 值
    pub fn counter_value(&self, name: &str) -> Option<u64> {
        let counters = self.counters.read().ok()?;
        counters.get(name).map(|c| c.value())
    }

    // ============================================
    // Gauge 操作
    // ============================================

    /// 获取或创建 Gauge
    #[inline]
    pub fn gauge(&self, name: &str) -> Gauge {
        let mut gauges = self.gauges.write().unwrap();
        gauges
            .entry(name.to_string())
            .or_insert_with(|| Gauge::new(name))
            .clone()
    }

    /// 设置值
    #[inline]
    pub fn set_gauge(&self, name: &str, value: f64) {
        if let Ok(mut gauges) = self.gauges.write() {
            if let Some(gauge) = gauges.get_mut(name) {
                gauge.set(value);
            } else {
                let gauge = Gauge::new(name);
                gauges.insert(name.to_string(), gauge);
            }
        }
    }

    /// 获取 Gauge 值
    pub fn gauge_value(&self, name: &str) -> Option<f64> {
        let gauges = self.gauges.read().ok()?;
        gauges.get(name).map(|g| g.value())
    }

    // ============================================
    // 阈值管理
    // ============================================

    /// 添加阈值
    pub fn add_threshold(&self, threshold: Threshold) {
        if let Ok(mut thresholds) = self.thresholds.write() {
            thresholds.push(threshold);
        }
    }

    /// 检查所有阈值
    ///
    /// 返回 `true` 如果有指标超过阈值。
    pub fn check_thresholds(&self) -> bool {
        if let Ok(thresholds) = self.thresholds.read() {
            for threshold in thresholds.iter() {
                match threshold.metric_type.as_str() {
                    "histogram" => {
                        if let Some(stats) = self.histogram_stats(&threshold.metric_name) {
                            let value = stats.p95;
                            if value > threshold.value {
                                // 阈值触发
                                return true;
                            }
                        }
                    }
                    "counter" => {
                        if let Some(value) = self.counter_value(&threshold.metric_name) {
                            if value as f64 > threshold.value {
                                return true;
                            }
                        }
                    }
                    "gauge" => {
                        if let Some(value) = self.gauge_value(&threshold.metric_name) {
                            if value > threshold.value {
                                return true;
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// 获取所有触发的阈值
    pub fn triggered_thresholds(&self) -> Vec<ThresholdAlert> {
        let mut alerts = Vec::new();

        if let Ok(thresholds) = self.thresholds.read() {
            for threshold in thresholds.iter() {
                let (triggered, current) = match threshold.metric_type.as_str() {
                    "histogram" => {
                        if let Some(stats) = self.histogram_stats(&threshold.metric_name) {
                            (stats.p95 > threshold.value, stats.p95)
                        } else {
                            continue;
                        }
                    }
                    "counter" => {
                        if let Some(value) = self.counter_value(&threshold.metric_name) {
                            (value as f64 > threshold.value, value as f64)
                        } else {
                            continue;
                        }
                    }
                    "gauge" => {
                        if let Some(value) = self.gauge_value(&threshold.metric_name) {
                            (value > threshold.value, value)
                        } else {
                            continue;
                        }
                    }
                    _ => continue,
                };

                if triggered {
                    alerts.push(ThresholdAlert {
                        metric_name: threshold.metric_name.clone(),
                        metric_type: threshold.metric_type.clone(),
                        threshold: threshold.value,
                        current,
                        level: threshold.level,
                    });
                }
            }
        }

        alerts
    }

    // ============================================
    // 快捷宏
    // ============================================

    /// 重置所有指标
    pub fn reset(&self) {
        if let Ok(mut histograms) = self.histograms.write() {
            histograms.clear();
        }
        if let Ok(mut counters) = self.counters.write() {
            counters.clear();
        }
        if let Ok(mut gauges) = self.gauges.write() {
            gauges.clear();
        }
    }

    /// 导出所有指标为 JSON（需要 serde feature）
    #[cfg(feature = "serde")]
    pub fn export_json(&self) -> String {
        use serde_json::{json, Map, Number, Value};

        let mut result = Map::new();

        // Export histograms
        if let Ok(histograms) = self.histograms.read() {
            for (name, hist) in histograms.iter() {
                let stats = hist.stats();
                result.insert(
                    format!("histogram.{}", name),
                    json!({
                        "count": stats.count,
                        "sum": stats.sum,
                        "min": stats.min,
                        "max": stats.max,
                        "avg": stats.avg,
                        "p50": stats.p50,
                        "p95": stats.p95,
                        "p99": stats.p99,
                        "p999": stats.p999,
                    }),
                );
            }
        }

        // Export counters
        if let Ok(counters) = self.counters.read() {
            for (name, counter) in counters.iter() {
                result.insert(
                    format!("counter.{}", name),
                    Value::Number(counter.value().into()),
                );
            }
        }

        // Export gauges
        if let Ok(gauges) = self.gauges.read() {
            for (name, gauge) in gauges.iter() {
                if let Some(n) = Number::from_f64(gauge.value()) {
                    result.insert(format!("gauge.{}", name), Value::Number(n));
                }
            }
        }

        serde_json::to_string_pretty(&result).unwrap_or_default()
    }
}

impl Default for PerfMonitor {
    fn default() -> Self {
        Self::new()
    }
}

// 重新导出快捷函数

/// 记录观测值到全局监控器
#[inline]
pub fn observe(name: &str, value: f64) {
    global().observe(name, value);
}

/// 记录延迟（微秒）到全局监控器
#[inline]
pub fn record_latency_us(name: &str, latency_us: f64) {
    global().record_latency_us(name, latency_us);
}

/// 记录延迟（毫秒）到全局监控器
#[inline]
pub fn record_latency_ms(name: &str, latency_ms: f64) {
    global().record_latency_ms(name, latency_ms);
}

/// 增加计数到全局监控器
#[inline]
pub fn inc_counter(name: &str) {
    global().inc_counter(name);
}

/// 增加指定值到全局监控器
#[inline]
pub fn inc_counter_by(name: &str, value: u64) {
    global().inc_counter_by(name, value);
}

/// 设置 Gauge 值到全局监控器
#[inline]
pub fn set_gauge(name: &str, value: f64) {
    global().set_gauge(name, value);
}

/// 检查阈值
#[inline]
pub fn check_thresholds() -> bool {
    global().check_thresholds()
}

/// 获取触发的阈值
#[inline]
pub fn triggered_thresholds() -> Vec<ThresholdAlert> {
    global().triggered_thresholds()
}

// ============================================
// 类型定义
// ============================================

/// Histogram 统计结果
#[derive(Debug, Clone)]
pub struct HistogramStats {
    /// 样本数量
    pub count: u64,
    /// 总和
    pub sum: f64,
    /// 最小值
    pub min: f64,
    /// 最大值
    pub max: f64,
    /// 平均值
    pub avg: f64,
    /// P50（中位数）
    pub p50: f64,
    /// P95
    pub p95: f64,
    /// P99
    pub p99: f64,
    /// P99.9
    pub p999: f64,
}

/// 阈值告警
#[derive(Debug, Clone)]
pub struct ThresholdAlert {
    pub metric_name: String,
    pub metric_type: String,
    pub threshold: f64,
    pub current: f64,
    pub level: ThresholdLevel,
}

// ============================================
// 预定义指标名称
// ============================================

/// 预定义的指标名称常量
pub mod metrics {
    // DOM 捕获相关
    pub const DOM_CAPTURE_US: &str = "dom_capture_us";
    pub const DOM_CAPTURE_COUNT: &str = "dom_capture_count";
    pub const DOM_NODES_CAPTURED: &str = "dom_nodes_captured";

    // 差分计算相关
    pub const DIFF_COMPUTE_US: &str = "diff_compute_us";
    pub const DIFF_COUNT: &str = "diff_count";
    pub const DIFF_NODES_PROCESSED: &str = "diff_nodes_processed";
    pub const DIFF_OPS_GENERATED: &str = "diff_ops_generated";

    // 内存相关
    pub const MEMORY_MB: &str = "memory_mb";
    pub const MEMORY_PEAK_MB: &str = "memory_peak_mb";
    pub const MEMORY_PAGES: &str = "memory_pages";

    // 对象池相关
    pub const POOL_ALLOCATION_US: &str = "pool_allocation_us";
    pub const POOL_HIT_RATE: &str = "pool_hit_rate";
    pub const POOL_MISSES: &str = "pool_misses";

    // Arena 相关
    pub const ARENA_ALLOC_US: &str = "arena_alloc_us";
    pub const ARENA_UTILIZATION: &str = "arena_utilization";
    pub const ARENA_CHUNKS: &str = "arena_chunks";

    // WebSocket 相关
    pub const WS_MESSAGES_SENT: &str = "ws_messages_sent";
    pub const WS_MESSAGES_RECEIVED: &str = "ws_messages_received";
    pub const WS_BYTES_SENT: &str = "ws_bytes_sent";
    pub const WS_BYTES_RECEIVED: &str = "ws_bytes_received";
    pub const WS_LATENCY_US: &str = "ws_latency_us";

    // 错误相关
    pub const ERRORS_TOTAL: &str = "errors_total";
    pub const ERRORS_SERIALIZATION: &str = "errors_serialization";
    pub const ERRORS_ALLOCATION: &str = "errors_allocation";
}
