//! # 内存监控模块
//!
//! 监控内存使用情况，检测内存泄漏。
//!
//! ## 特性
//!
//! - 零 `unsafe` 代码
//! - 周期性采样
//! - 泄漏检测
//! - 告警机制
//!
//! ## 使用示例
//!
//! ```rust
//! use chrome_dom_diff::MemoryMonitor;
//!
//! let monitor = MemoryMonitor::new();
//!
//! // 初始采样
//! monitor.sample();
//!
//! // ... 运行代码 ...
//!
//! // 再次采样
//! monitor.sample();
//!
//! // 检测泄漏
//! if monitor.detect_leak() {
//!     println!("检测到内存泄漏！");
//! }
//! ```

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};

/// 内存监控器
///
/// ## 线程安全
///
/// 使用原子操作，支持多线程环境。
///
/// ## 监控原理
///
/// 1. 定期采样当前内存使用量
/// 2. 计算内存增长率（bytes/hour）
/// 3. 如果增长率超过阈值，触发告警
pub struct MemoryMonitor {
    /// 基线内存（字节）
    baseline: AtomicUsize,

    /// 当前内存（字节）
    current: AtomicUsize,

    /// 峰值内存（字节）
    peak: AtomicUsize,

    /// 增长率（bytes/hour）
    growth_rate: AtomicU64,

    /// 上次采样时间
    last_sample: std::sync::Mutex<Instant>,

    /// 采样次数
    sample_count: AtomicU64,

    /// 累计增长（用于计算平均值）
    cumulative_growth: AtomicU64,
}

impl Default for MemoryMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryMonitor {
    /// 创建新的内存监控器
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        let initial_memory = Self::current_memory_usage();

        Self {
            baseline: AtomicUsize::new(initial_memory),
            current: AtomicUsize::new(initial_memory),
            peak: AtomicUsize::new(initial_memory),
            growth_rate: AtomicU64::new(0),
            last_sample: std::sync::Mutex::new(Instant::now()),
            sample_count: AtomicU64::new(0),
            cumulative_growth: AtomicU64::new(0),
        }
    }

    /// 采样当前内存使用情况
    ///
    /// 应该定期调用此方法（例如每 10 分钟）。
    #[inline]
    pub fn sample(&self) {
        let now = Instant::now();
        let current_memory = Self::current_memory_usage();

        // 更新当前内存
        self.current.store(current_memory, Ordering::Relaxed);

        // 更新峰值
        let mut peak = self.peak.load(Ordering::Relaxed);
        loop {
            if current_memory <= peak {
                break;
            }
            match self.peak.compare_exchange_weak(peak, current_memory, Ordering::Relaxed, Ordering::Relaxed) {
                Ok(_) => break,
                Err(new_peak) => peak = new_peak,
            }
        }

        // 计算时间差
        let elapsed = {
            let mut last_sample = self.last_sample.lock().unwrap();
            let duration = last_sample.elapsed();
            *last_sample = now;
            duration
        };

        // 计算增长率（bytes/hour）
        if elapsed > Duration::ZERO {
            let seconds = elapsed.as_secs_f64();
            let growth = if current_memory > self.baseline.load(Ordering::Relaxed) {
                current_memory - self.baseline.load(Ordering::Relaxed)
            } else {
                0
            };

            let rate_per_second = growth as f64 / seconds;
            let rate_per_hour = rate_per_second * 3600.0;

            self.growth_rate.store(rate_per_hour as u64, Ordering::Relaxed);

            // 更新累计增长
            self.cumulative_growth.fetch_add(growth as u64, Ordering::Relaxed);
        }

        // 更新采样次数
        self.sample_count.fetch_add(1, Ordering::Relaxed);
    }

    /// 检测内存泄漏
    ///
    /// 判断标准：
    /// - 内存持续增长（增长率 > 阈值）
    /// - 采样次数足够（避免误报）
    ///
    /// 返回 `true` 表示可能存在泄漏。
    #[inline]
    #[must_use]
    pub fn detect_leak(&self) -> bool {
        const GROWTH_THRESHOLD: u64 = 1024 * 1024; // 1 MB/hour
        const MIN_SAMPLES: u64 = 2;

        let samples = self.sample_count.load(Ordering::Relaxed);
        let rate = self.growth_rate.load(Ordering::Relaxed);

        samples >= MIN_SAMPLES && rate > GROWTH_THRESHOLD
    }

    /// 如果超过阈值则告警
    ///
    /// 阈值单位：MB
    #[inline]
    pub fn alert_if_exceeded(&self, threshold_mb: usize) {
        let threshold_bytes = threshold_mb * 1024 * 1024;
        let growth = self.current.load(Ordering::Relaxed).saturating_sub(self.baseline.load(Ordering::Relaxed));

        if growth > threshold_bytes {
            eprintln!(
                "⚠️  内存告警：增长了 {} MB (阈值: {} MB)",
                growth / (1024 * 1024),
                threshold_mb
            );
        }
    }

    /// 获取基线内存（KB）
    #[inline]
    #[must_use]
    pub fn baseline_kb(&self) -> usize {
        self.baseline.load(Ordering::Relaxed) / 1024
    }

    /// 获取当前内存（KB）
    #[inline]
    #[must_use]
    pub fn current_kb(&self) -> usize {
        self.current.load(Ordering::Relaxed) / 1024
    }

    /// 获取峰值内存（KB）
    #[inline]
    #[must_use]
    pub fn peak_kb(&self) -> usize {
        self.peak.load(Ordering::Relaxed) / 1024
    }

    /// 获取增长率（bytes/hour）
    #[inline]
    #[must_use]
    pub fn growth_rate(&self) -> u64 {
        self.growth_rate.load(Ordering::Relaxed)
    }

    /// 获取采样次数
    #[inline]
    #[must_use]
    pub fn sample_count(&self) -> u64 {
        self.sample_count.load(Ordering::Relaxed)
    }

    /// 重置监控器
    #[inline]
    pub fn reset(&self) {
        let current_memory = Self::current_memory_usage();
        self.baseline.store(current_memory, Ordering::Relaxed);
        self.current.store(current_memory, Ordering::Relaxed);
        self.peak.store(current_memory, Ordering::Relaxed);
        self.growth_rate.store(0, Ordering::Relaxed);
        self.sample_count.store(0, Ordering::Relaxed);
        self.cumulative_growth.store(0, Ordering::Relaxed);
        *self.last_sample.lock().unwrap() = Instant::now();
    }

    /// 获取内存使用情况摘要
    #[inline]
    #[must_use]
    pub fn summary(&self) -> MemorySummary {
        MemorySummary {
            baseline_kb: self.baseline_kb(),
            current_kb: self.current_kb(),
            peak_kb: self.peak_kb(),
            growth_bytes_per_hour: self.growth_rate(),
            sample_count: self.sample_count(),
            has_leak: self.detect_leak(),
        }
    }

    /// 同步内存指标到性能监控系统
    ///
    /// 此方法会将内存监控数据推送到全局性能监控器，
    /// 用于统一的数据收集和告警。
    ///
    /// ## 集成的指标
    ///
    /// - `memory_mb`: 当前内存使用（MB）
    /// - `memory_peak_mb`: 峰值内存（MB）
    /// - `memory_growth_bytes_per_hour`: 增长率
    /// - `memory_samples`: 采样次数
    ///
    /// ## 使用示例
    ///
    /// ```rust
    /// let monitor = MemoryMonitor::new();
    /// monitor.sample();
    /// monitor.sync_to_perf_monitor();  // 同步到性能监控系统
    /// ```
    #[inline]
    pub fn sync_to_perf_monitor(&self) {
        // 使用性能监控系统的快捷函数
        crate::monitoring::set_gauge(
            crate::monitoring::metrics::MEMORY_MB,
            self.current_kb() as f64 / 1024.0,
        );

        crate::monitoring::set_gauge(
            crate::monitoring::metrics::MEMORY_PEAK_MB,
            self.peak_kb() as f64 / 1024.0,
        );

        crate::monitoring::set_gauge(
            "memory_growth_bytes_per_hour",
            self.growth_rate() as f64,
        );

        crate::monitoring::inc_counter_by("memory_samples", self.sample_count());
    }

    /// 采样并同步到性能监控系统（便捷方法）
    ///
    /// 等价于先调用 `sample()` 再调用 `sync_to_perf_monitor()`。
    #[inline]
    pub fn sample_and_sync(&self) {
        self.sample();
        self.sync_to_perf_monitor();
    }

    /// 获取当前进程内存使用量（字节）
    ///
    /// ## 平台差异
    ///
    /// - Linux: 读取 `/proc/self/status`
    /// - 其他: 返回 0（需要平台特定实现）
    #[must_use]
    fn current_memory_usage() -> usize {
        #[cfg(target_os = "linux")]
        {
            Self::linux_memory_usage()
        }

        #[cfg(not(target_os = "linux"))]
        {
            // 非平台暂不支持
            0
        }
    }

    /// Linux 平台内存读取
    #[cfg(target_os = "linux")]
    #[must_use]
    fn linux_memory_usage() -> usize {
        use std::fs;

        // 读取 /proc/self/status
        if let Ok(status) = fs::read_to_string("/proc/self/status") {
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    // VmRSS: 实际物理内存使用
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<usize>() {
                            return kb * 1024; // 转换为字节
                        }
                    }
                }
            }
        }

        0
    }
}

impl std::fmt::Debug for MemoryMonitor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryMonitor")
            .field("baseline_kb", &self.baseline_kb())
            .field("current_kb", &self.current_kb())
            .field("peak_kb", &self.peak_kb())
            .field("growth_rate", &self.growth_rate())
            .field("sample_count", &self.sample_count())
            .finish()
    }
}

/// 内存使用摘要
#[derive(Debug, Clone, Copy)]
pub struct MemorySummary {
    /// 基线内存（KB）
    pub baseline_kb: usize,

    /// 当前内存（KB）
    pub current_kb: usize,

    /// 峰值内存（KB）
    pub peak_kb: usize,

    /// 增长率（bytes/hour）
    pub growth_bytes_per_hour: u64,

    /// 采样次数
    pub sample_count: u64,

    /// 是否检测到泄漏
    pub has_leak: bool,
}

impl MemorySummary {
    /// 获取内存增长（KB）
    #[must_use]
    pub const fn growth_kb(&self) -> usize {
        self.current_kb.saturating_sub(self.baseline_kb)
    }

    /// 格式化报告
    #[must_use]
    pub fn report(&self) -> String {
        format!(
            "内存监控报告:\n\
             - 基线: {} KB\n\
             - 当前: {} KB\n\
             - 峰值: {} KB\n\
             - 增长: {} KB\n\
             - 增长率: {} bytes/hour\n\
             - 采样次数: {}\n\
             - 泄漏检测: {}",
            self.baseline_kb,
            self.current_kb,
            self.peak_kb,
            self.growth_kb(),
            self.growth_bytes_per_hour,
            self.sample_count,
            if self.has_leak { "⚠️ 检测到泄漏" } else { "✅ 正常" }
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn test_monitor_creation() {
        let monitor = MemoryMonitor::new();

        assert!(monitor.baseline_kb() > 0 || cfg!(not(target_os = "linux")));
        assert_eq!(monitor.current_kb(), monitor.baseline_kb());
        assert_eq!(monitor.peak_kb(), monitor.baseline_kb());
        assert_eq!(monitor.sample_count(), 0);
    }

    #[test]
    fn test_sample() {
        let monitor = MemoryMonitor::new();

        monitor.sample();

        assert_eq!(monitor.sample_count(), 1);

        // 等待一小段时间确保时间差
        thread::sleep(Duration::from_millis(10));
        monitor.sample();

        assert_eq!(monitor.sample_count(), 2);
    }

    #[test]
    fn test_detect_leak() {
        let monitor = MemoryMonitor::new();

        // 初始采样
        monitor.sample();

        // 采样不足，不应检测到泄漏
        assert!(!monitor.detect_leak());

        // 模拟内存增长（通过修改内部状态）
        // 注意：这只是一个测试，实际泄漏需要时间
        monitor.growth_rate.store(2 * 1024 * 1024, Ordering::Relaxed); // 2 MB/hour

        // 再次采样
        monitor.sample();

        // 现在应该检测到泄漏
        assert!(monitor.detect_leak());
    }

    #[test]
    fn test_alert_threshold() {
        let monitor = MemoryMonitor::new();

        // 模拟内存增长
        monitor.current.store(monitor.baseline.load(Ordering::Relaxed) + 2 * 1024 * 1024, Ordering::Relaxed);

        // 阈值 1 MB，应该触发告警
        monitor.alert_if_exceeded(1);

        // 阈值 10 MB，不应触发告警
        monitor.alert_if_exceeded(10);
    }

    #[test]
    fn test_reset() {
        let monitor = MemoryMonitor::new();

        monitor.sample();
        monitor.sample();

        assert_eq!(monitor.sample_count(), 2);

        monitor.reset();

        assert_eq!(monitor.sample_count(), 0);
        assert_eq!(monitor.growth_rate(), 0);
    }

    #[test]
    fn test_summary() {
        let monitor = MemoryMonitor::new();
        monitor.sample();

        let summary = monitor.summary();

        assert_eq!(summary.sample_count, 1);
        assert_eq!(summary.baseline_kb, monitor.baseline_kb());
        assert_eq!(summary.current_kb, monitor.current_kb());
    }

    #[test]
    fn test_summary_report() {
        let monitor = MemoryMonitor::new();
        monitor.sample();

        let summary = monitor.summary();
        let report = summary.report();

        assert!(report.contains("内存监控报告"));
        assert!(report.contains("基线"));
        assert!(report.contains("当前"));
    }

    #[test]
    fn test_peak_tracking() {
        let monitor = MemoryMonitor::new();

        let initial = monitor.current.load(Ordering::Relaxed);

        // 模拟内存增长
        monitor.current.store(initial + 1024 * 1024, Ordering::Relaxed);
        monitor.sample();

        assert!(monitor.peak_kb() >= monitor.baseline_kb());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_linux_memory_reading() {
        let usage = MemoryMonitor::linux_memory_usage();

        // Linux 上应该能读到内存值
        // 但在某些环境（如容器）中可能失败
        if usage > 0 {
            assert!(usage >= 1024); // 至少几 KB
        }
    }

    #[test]
    fn test_concurrent_sampling() {
        let monitor = std::sync::Arc::new(MemoryMonitor::new());
        let mut handles = vec![];

        // 多线程并发采样
        for _ in 0..10 {
            let monitor = monitor.clone();
            handles.push(thread::spawn(move || {
                monitor.sample();
            }));
        }

        for handle in handles {
            handle.join().unwrap();
        }

        assert_eq!(monitor.sample_count(), 10);
    }

    #[test]
    fn test_growth_calculation() {
        let monitor = MemoryMonitor::new();

        // 初始
        monitor.sample();

        // 等待一小段时间
        thread::sleep(Duration::from_millis(50));

        // 模拟内存增长
        let growth = 1024 * 100; // 100 KB
        monitor.current.store(monitor.current.load(Ordering::Relaxed) + growth, Ordering::Relaxed);
        monitor.sample();

        // 增长率应该大于 0
        assert!(monitor.growth_rate() > 0);
    }
}
