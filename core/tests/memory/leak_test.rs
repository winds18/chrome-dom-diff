//! 内存泄漏测试

#[cfg(test)]
mod tests {
    #[test]
    fn test_leak() {
        let monitor = chrome_dom_diff::MemoryMonitor::new();
        monitor.sample();
        assert!(!monitor.detect_leak());
    }
}
