//! Pool 基准测试

#[cfg(test)]
mod benches {
    #[test]
    fn test_pool_acquire() {
        let mut pool = chrome_dom_diff::ObjectPool::<String>::new();
        let mut obj = pool.acquire();
        *obj = String::from("test");
        assert_eq!(&**obj, "test");
    }
}
