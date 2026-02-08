//! Arena 基准测试

#[cfg(test)]
mod benches {
    use std::hint::black_box;

    #[test]
    fn test_arena_alloc() {
        // 简单的占位符测试
        let arena = chrome_dom_diff::DomArena::new();
        let s = arena.alloc_str("test");
        assert_eq!(s, "test");
    }
}
