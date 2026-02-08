//! # 阈值和告警
//!
//! 定义性能指标的阈值和告警级别。

/// 阈值
#[derive(Debug, Clone)]
pub struct Threshold {
    /// 指标名称
    pub metric_name: String,
    /// 指标类型（"histogram", "counter", "gauge"）
    pub metric_type: String,
    /// 阈值
    pub value: f64,
    /// 告警级别
    pub level: ThresholdLevel,
}

impl Threshold {
    /// 创建新的阈值
    pub fn new(
        metric_name: impl Into<String>,
        metric_type: impl Into<String>,
        value: f64,
        level: ThresholdLevel,
    ) -> Self {
        Self {
            metric_name: metric_name.into(),
            metric_type: metric_type.into(),
            value,
            level,
        }
    }

    /// 创建警告级别阈值
    pub fn warning(
        metric_name: impl Into<String>,
        metric_type: impl Into<String>,
        value: f64,
    ) -> Self {
        Self::new(metric_name, metric_type, value, ThresholdLevel::Warning)
    }

    /// 创建严重级别阈值
    pub fn critical(
        metric_name: impl Into<String>,
        metric_type: impl Into<String>,
        value: f64,
    ) -> Self {
        Self::new(metric_name, metric_type, value, ThresholdLevel::Critical)
    }
}

/// 告警级别
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThresholdLevel {
    /// 警告
    Warning,
    /// 严重
    Critical,
    /// 致命
    Fatal,
}

impl ThresholdLevel {
    /// 转换为字符串
    pub fn as_str(&self) -> &str {
        match self {
            Self::Warning => "warning",
            Self::Critical => "critical",
            Self::Fatal => "fatal",
        }
    }

    /// 从字符串解析
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "warning" => Some(Self::Warning),
            "critical" => Some(Self::Critical),
            "fatal" => Some(Self::Fatal),
            _ => None,
        }
    }

    /// 获取优先级（数值越大越严重）
    pub fn priority(&self) -> u8 {
        match self {
            Self::Warning => 1,
            Self::Critical => 2,
            Self::Fatal => 3,
        }
    }
}

/// 预定义的默认阈值
pub fn default_thresholds() -> Vec<Threshold> {
    vec![
        // DOM 捕获延迟
        Threshold::warning("dom_capture_us", "histogram", 3000.0), // 3ms
        Threshold::critical("dom_capture_us", "histogram", 5000.0), // 5ms
        Threshold::new("dom_capture_us", "histogram", 7000.0, ThresholdLevel::Fatal), // 7ms

        // 差分计算延迟
        Threshold::warning("diff_compute_us", "histogram", 8000.0), // 8ms
        Threshold::critical("diff_compute_us", "histogram", 10000.0), // 10ms
        Threshold::new("diff_compute_us", "histogram", 15000.0, ThresholdLevel::Fatal), // 15ms

        // 内存使用
        Threshold::warning("memory_mb", "gauge", 40.0), // 40MB
        Threshold::critical("memory_mb", "gauge", 50.0), // 50MB
        Threshold::new("memory_mb", "gauge", 64.0, ThresholdLevel::Fatal), // 64MB

        // 内存页面数
        Threshold::warning("memory_pages", "gauge", 700.0), // ~45MB
        Threshold::critical("memory_pages", "gauge", 800.0), // ~50MB

        // 对象池命中率
        Threshold::warning("pool_hit_rate", "gauge", 80.0), // 80%
        Threshold::critical("pool_hit_rate", "gauge", 60.0), // 60%

        // Arena 利用率
        Threshold::warning("arena_utilization", "gauge", 80.0), // 80%
        Threshold::critical("arena_utilization", "gauge", 90.0), // 90%
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_threshold_level() {
        assert_eq!(ThresholdLevel::Warning.as_str(), "warning");
        assert_eq!(ThresholdLevel::Critical.as_str(), "critical");
        assert_eq!(ThresholdLevel::Fatal.as_str(), "fatal");

        assert_eq!(ThresholdLevel::from_str("warning"), Some(ThresholdLevel::Warning));
        assert_eq!(ThresholdLevel::from_str("unknown"), None);
    }

    #[test]
    fn test_threshold_priority() {
        assert!(ThresholdLevel::Critical.priority() > ThresholdLevel::Warning.priority());
        assert!(ThresholdLevel::Fatal.priority() > ThresholdLevel::Critical.priority());
    }
}
