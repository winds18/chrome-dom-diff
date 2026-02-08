//! 内存增长基准测试

#[cfg(test)]
mod benches {
    #[test]
    fn test_memory_growth() {
        let monitor = chrome_dom_diff::MemoryMonitor::new();
        monitor.sample();
        assert!(monitor.sample_count() >= 1);
    }
}
