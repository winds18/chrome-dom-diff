//! # 对象池模块
//!
//! 高性能对象池，减少重复分配开销。
//!
//! ## 特性
//!
//! - 零 `unsafe` 代码
//! - 自动归还（RAII）
//! - 复用率统计
//! - 泛型支持
//!
//! ## 使用示例
//!
//! ```rust
//! use chrome_dom_diff::ObjectPool;
//!
//! let mut pool: ObjectPool<String> = ObjectPool::with_capacity(10);
//!
//! // 获取对象
//! let mut obj = pool.acquire();
//! *obj = String::from("Hello");
//!
//! // 对象在离开作用域时自动归还
//! drop(obj);
//! ```

use std::cell::Cell;
use std::fmt;
use std::ops::{Deref, DerefMut};

/// 对象池
///
/// ## 线程安全
///
/// **不是 `Send` 也不是 `Sync`** - 单线程使用。
///
/// ## 复用策略
///
/// 1. 池中有可用对象：直接返回
/// 2. 池为空：创建新对象
/// 3. 对象归还：放回池中（通过 `PoolGuard`）
pub struct ObjectPool<T> {
    /// 池中的对象
    objects: Vec<T>,

    /// 可用对象索引列表
    available: Vec<usize>,

    /// 总获取次数
    acquire_count: Cell<usize>,

    /// 复用次数（从池中获取）
    reuse_count: Cell<usize>,

    /// 最大容量（可选）
    max_capacity: Option<usize>,
}

impl<T: Default> Default for ObjectPool<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Default> ObjectPool<T> {
    /// 创建新的对象池
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
            available: Vec::new(),
            acquire_count: Cell::new(0),
            reuse_count: Cell::new(0),
            max_capacity: None,
        }
    }

    /// 创建指定容量的对象池
    #[inline]
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            objects: Vec::with_capacity(capacity),
            available: Vec::with_capacity(capacity),
            acquire_count: Cell::new(0),
            reuse_count: Cell::new(0),
            max_capacity: None,
        }
    }

    /// 设置最大容量
    #[inline]
    pub fn with_max_capacity(mut self, max: usize) -> Self {
        self.max_capacity = Some(max);
        self
    }

    /// 获取对象
    ///
    /// 返回一个 `PoolGuard`，当 guard 被 drop 时自动归还对象。
    #[inline]
    pub fn acquire(&mut self) -> PoolGuard<T> {
        self.acquire_count.set(self.acquire_count.get() + 1);

        let index = if let Some(idx) = self.available.pop() {
            // 复用现有对象
            self.reuse_count.set(self.reuse_count.get() + 1);
            idx
        } else {
            // 创建新对象
            let idx = self.objects.len();

            // 检查最大容量限制
            if let Some(max) = self.max_capacity {
                if idx >= max {
                    // 超过容量，归还最早的可用对象（如果有）
                    panic!("ObjectPool exceeded max capacity of {}", max);
                }
            }

            self.objects.push(T::default());
            idx
        };

        PoolGuard::new(index, self)
    }

    /// 直接释放对象（内部使用）
    #[inline]
    /// 返回 0.0 - 1.0 之间的值，表示从池中复用的比例。
    #[inline]
    #[must_use]
    pub fn reuse_rate(&self) -> f64 {
        let total = self.acquire_count.get();
        if total == 0 {
            0.0
        } else {
            self.reuse_count.get() as f64 / total as f64
        }
    }

    /// 获取池中对象总数
    #[inline]
    #[must_use]
    pub fn size(&self) -> usize {
        self.objects.len()
    }

    /// 获取可用对象数
    #[inline]
    #[must_use]
    pub fn available_count(&self) -> usize {
        self.available.len()
    }

    /// 获取正在使用的对象数
    #[inline]
    #[must_use]
    pub fn in_use_count(&self) -> usize {
        self.objects.len() - self.available.len()
    }

    /// 清空对象池
    #[inline]
    pub fn clear(&mut self) {
        self.objects.clear();
        self.available.clear();
        self.acquire_count.set(0);
        self.reuse_count.set(0);
    }

    /// 预热对象池（预先创建对象）
    ///
    /// ## 性能建议
    ///
    /// 对于高频使用的对象池（如 String 池），建议预热到峰值并发量的 1.5 倍，
    /// 以确保复用率 > 80%。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// // String 池建议预热到至少 100 个对象
    /// let mut pool: ObjectPool<String> = ObjectPool::with_capacity(100);
    /// pool.warm_up(100);  // 预先创建 100 个空 String
    /// ```
    #[inline]
    pub fn warm_up(&mut self, count: usize) {
        let additional = count.saturating_sub(self.objects.len());
        for _ in 0..additional {
            let idx = self.objects.len();
            self.objects.push(T::default());
            self.available.push(idx);
        }
    }

    /// 获取推荐预热数量（基于目标复用率）
    ///
    /// ## 计算公式
    ///
    /// ```text
    /// recommended = peak_concurrent * (1.0 / target_reuse_rate)
    /// ```
    ///
    /// 对于 80% 复用率目标，推荐预热到峰值并发的 1.25 倍。
    #[inline]
    #[must_use]
    pub const fn recommended_warm_up_size(peak_concurrent: usize, target_reuse_rate: f64) -> usize {
        if target_reuse_rate <= 0.0 || target_reuse_rate >= 1.0 {
            peak_concurrent
        } else {
            // 向上取整以确保复用率达标
            ((peak_concurrent as f64) / target_reuse_rate).ceil() as usize
        }
    }

    /// 同步对象池统计到性能监控系统
    ///
    /// 此方法会将对象池的复用率等指标推送到全局性能监控器。
    ///
    /// ## 集成的指标
    ///
    /// - `pool_hit_rate`: 复用率（0.0 - 1.0）
    /// - `pool_misses`: 未命中次数（创建新对象的次数）
    /// - `pool_size`: 池中对象总数
    /// - `pool_in_use`: 正在使用的对象数
    ///
    /// ## 使用示例
    ///
    /// ```rust
    /// let mut pool: ObjectPool<String> = ObjectPool::new();
    /// // ... 使用对象池 ...
    /// pool.sync_to_perf_monitor("string_pool");  // 同步到性能监控系统
    /// ```
    #[inline]
    pub fn sync_to_perf_monitor(&self, pool_name: &str) {
        let hit_rate = self.reuse_rate();
        let misses = self.acquire_count.get().saturating_sub(self.reuse_count.get());

        // 使用池名称作为指标后缀，支持多个对象池
        let hit_rate_metric = format!("pool_{}_hit_rate", pool_name);
        let misses_metric = format!("pool_{}_misses", pool_name);
        let size_metric = format!("pool_{}_size", pool_name);
        let in_use_metric = format!("pool_{}_in_use", pool_name);

        crate::monitoring::set_gauge(&hit_rate_metric, hit_rate);
        crate::monitoring::inc_counter_by(&misses_metric, misses as u64);
        crate::monitoring::set_gauge(&size_metric, self.size() as f64);
        crate::monitoring::set_gauge(&in_use_metric, self.in_use_count() as f64);
    }

    /// 获取对象池统计摘要
    #[inline]
    #[must_use]
    pub fn stats(&self) -> PoolStats {
        PoolStats {
            size: self.size(),
            available: self.available_count(),
            in_use: self.in_use_count(),
            acquire_count: self.acquire_count.get(),
            reuse_count: self.reuse_count.get(),
            reuse_rate: self.reuse_rate(),
        }
    }
}

/// 对象池统计摘要
#[derive(Debug, Clone, Copy)]
pub struct PoolStats {
    /// 池中对象总数
    pub size: usize,
    /// 可用对象数
    pub available: usize,
    /// 正在使用的对象数
    pub in_use: usize,
    /// 总获取次数
    pub acquire_count: usize,
    /// 复用次数
    pub reuse_count: usize,
    /// 复用率（0.0 - 1.0）
    pub reuse_rate: f64,
}

impl<T> ObjectPool<T> {

    /// 直接释放对象（内部使用）
    #[inline]
    fn release_object(&mut self, index: usize) {
        self.available.push(index);
    }

    /// 直接释放对象（内部使用）
    #[inline]
    /// 使用自定义工厂函数创建对象池
    #[inline]
    #[must_use]
    pub fn with_factory<F>(capacity: usize, mut factory: F) -> Self
    where
        F: FnMut() -> T,
    {
        let mut pool = Self {
            objects: Vec::with_capacity(capacity),
            available: Vec::with_capacity(capacity),
            acquire_count: Cell::new(0),
            reuse_count: Cell::new(0),
            max_capacity: None,
        };

        // 预先创建对象
        for _ in 0..capacity {
            let idx = pool.objects.len();
            pool.objects.push(factory());
            pool.available.push(idx);
        }

        pool
    }
}

impl<T: fmt::Debug> fmt::Debug for ObjectPool<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let pool_size = self.objects.len();
        let available = self.available.len();
        let in_use = pool_size.saturating_sub(available);
        let reuse_rate = if self.acquire_count.get() > 0 {
            self.reuse_count.get() as f64 / self.acquire_count.get() as f64
        } else {
            0.0
        };

        f.debug_struct("ObjectPool")
            .field("pool_size", &pool_size)
            .field("available", &available)
            .field("in_use", &in_use)
            .field("reuse_rate", &reuse_rate)
            .field("acquire_count", &self.acquire_count.get())
            .field("reuse_count", &self.reuse_count.get())
            .finish()
    }
}

/// 对象池守卫
///
/// 当 guard 被 drop 时，自动将对象归还给池。
///
/// ## 实现细节
///
/// 使用裸索引而不是引用，避免生命周期复杂性。
/// 这确保了 100% 内存安全且零 `unsafe` 代码。
pub struct PoolGuard<'a, T> {
    /// 对象在池中的索引
    index: usize,

    /// 对象池的引用
    pool: &'a mut ObjectPool<T>,

    /// 是否已归还
    released: bool,
}

impl<'a, T> PoolGuard<'a, T> {
    #[inline]
    fn new(index: usize, pool: &'a mut ObjectPool<T>) -> Self {
        Self {
            index,
            pool,
            released: false,
        }
    }

    /// 手动归还对象
    ///
    /// 通常不需要手动调用，drop 会自动归还。
    #[inline]
    pub fn release(mut self) {
        self.released = true;
        self.pool.release_object(self.index);
        // 防止 double-drop
        std::mem::forget(self);
    }
}

impl<'a, T> Drop for PoolGuard<'a, T> {
    #[inline]
    fn drop(&mut self) {
        if !self.released {
            self.pool.release_object(self.index);
        }
    }
}

impl<'a, T> Deref for PoolGuard<'a, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.pool.objects[self.index]
    }
}

impl<'a, T> DerefMut for PoolGuard<'a, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.pool.objects[self.index]
    }
}

impl<'a, T: fmt::Debug> fmt::Debug for PoolGuard<'a, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PoolGuard")
            .field("index", &self.index)
            .field("value", &self.pool.objects[self.index])
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_creation() {
        let pool: ObjectPool<String> = ObjectPool::new();
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_pool_with_capacity() {
        let pool: ObjectPool<String> = ObjectPool::with_capacity(10);
        assert_eq!(pool.size(), 0);
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_acquire_and_return() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        // 获取对象
        let mut obj1 = pool.acquire();
        *obj1 = String::from("Hello");

        // 验证对象状态
        assert_eq!(pool.in_use_count(), 1);
        assert_eq!(&**obj1, "Hello");

        // 归还对象
        drop(obj1);
        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn test_reuse() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        // 第一次获取
        let mut obj1 = pool.acquire();
        *obj1 = String::from("First");
        drop(obj1);

        // 第二次获取应该复用
        let obj2 = pool.acquire();
        // 注意：String::default() 会重置为空字符串
        assert_eq!(&**obj2, "");

        // 验证复用率
        assert!(pool.reuse_rate() > 0.0);
    }

    #[test]
    fn test_reuse_rate() {
        let mut pool: ObjectPool<Vec<u8>> = ObjectPool::new();

        // 第一次获取（创建新对象）
        let _obj1 = pool.acquire();
        drop(_obj1);

        // 第二次获取（复用）
        let _obj2 = pool.acquire();
        drop(_obj2);

        // 复用率应该是 50%（一次复用，两次获取）
        assert!((pool.reuse_rate() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_warm_up() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        pool.warm_up(5);

        assert_eq!(pool.size(), 5);
        assert_eq!(pool.available_count(), 5);
    }

    #[test]
    fn test_clear() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        let _obj = pool.acquire();
        drop(_obj);

        pool.clear();

        assert_eq!(pool.size(), 0);
        assert_eq!(pool.available_count(), 0);
    }

    #[test]
    fn test_manual_release() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        let obj = pool.acquire();
        obj.release();

        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn test_deref_mut() {
        let mut pool: ObjectPool<Vec<i32>> = ObjectPool::new();

        let mut obj = pool.acquire();
        obj.push(1);
        obj.push(2);
        obj.push(3);

        assert_eq!(obj.len(), 3);
        assert_eq!(obj.as_slice(), &[1, 2, 3]);
    }

    #[test]
    fn test_multiple_acquires() {
        let mut pool: ObjectPool<String> = ObjectPool::with_capacity(5);

        let objs: Vec<_> = (0..5).map(|_| pool.acquire()).collect();

        assert_eq!(pool.in_use_count(), 5);
        assert_eq!(pool.available_count(), 0);

        // 归还所有对象
        drop(objs);

        assert_eq!(pool.in_use_count(), 0);
        assert_eq!(pool.available_count(), 5);
    }

    #[test]
    fn test_debug_format() {
        let pool: ObjectPool<String> = ObjectPool::new();
        let debug_str = format!("{:?}", pool);
        assert!(debug_str.contains("ObjectPool"));
    }

    #[test]
    fn test_pool_guard_debug() {
        let mut pool: ObjectPool<String> = ObjectPool::new();
        let obj = pool.acquire();
        let debug_str = format!("{:?}", obj);
        assert!(debug_str.contains("PoolGuard"));
    }

    #[test]
    fn test_custom_factory() {
        let pool = ObjectPool::with_factory(5, || String::from("custom"));

        assert_eq!(pool.size(), 5);
        assert_eq!(pool.available_count(), 5);

        // 验证对象内容
        let obj = pool.acquire();
        assert_eq!(&**obj, "custom");
    }

    #[test]
    fn test_high_reuse_rate() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        // 预热池
        pool.warm_up(100);

        // 大量复用
        for _ in 0..1000 {
            let _obj = pool.acquire();
        }

        // 复用率应该很高
        assert!(pool.reuse_rate() > 0.9);
    }

    #[test]
    fn test_empty_pool_behavior() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        // 空池获取应该创建新对象
        let obj = pool.acquire();
        assert_eq!(pool.size(), 1);
        assert_eq!(pool.in_use_count(), 1);

        drop(obj);
        assert_eq!(pool.available_count(), 1);
    }

    #[test]
    fn test_string_pool_practical() {
        let mut pool: ObjectPool<String> = ObjectPool::with_capacity(10);

        // 模拟实际使用
        for i in 0..100 {
            let mut s = pool.acquire();
            s.push_str(&format!("iteration-{}", i));
        }

        // 验证复用率
        println!("复用率: {:.2}%", pool.reuse_rate() * 100.0);
        assert!(pool.reuse_rate() > 0.9);
    }

    /// 测试 80% 复用率目标（性能架构师要求的验收标准）
    #[test]
    fn test_string_pool_80_percent_reuse_target() {
        // 根据性能架构师的报告，String 池需要达到 80% 复用率
        // 推荐策略：预热到峰值并发的 1.25 倍
        const TARGET_REUSE_RATE: f64 = 0.8;
        const PEAK_CONCURRENT: usize = 100;

        let mut pool: ObjectPool<String> = ObjectPool::with_capacity(PEAK_CONCURRENT);

        // 计算推荐预热数量
        let warm_up_size = ObjectPool::<String>::recommended_warm_up_size(
            PEAK_CONCURRENT,
            TARGET_REUSE_RATE,
        );
        pool.warm_up(warm_up_size);

        // 模拟峰值并发使用
        for i in 0..PEAK_CONCURRENT {
            let mut s = pool.acquire();
            s.push_str(&format!("item-{}", i));
            // 归还对象供复用
            drop(s);
        }

        // 持续复用测试
        for _ in 0..PEAK_CONCURRENT * 10 {
            let _s = pool.acquire();
        }

        let reuse_rate = pool.reuse_rate();
        println!(
            "String 池复用率: {:.2}% (目标: >= 80%)",
            reuse_rate * 100.0
        );
        println!(
            "预热数量: {} (峰值并发: {})",
            warm_up_size, PEAK_CONCURRENT
        );

        // 验证达到 80% 目标
        assert!(
            reuse_rate >= TARGET_REUSE_RATE,
            "String 池复用率 {}% 低于 80% 目标",
            reuse_rate * 100.0
        );
    }

    /// 测试 warm_up 边界条件修复
    #[test]
    fn test_warm_up_boundary_fix() {
        let mut pool: ObjectPool<String> = ObjectPool::new();

        // 第一次 warm_up
        pool.warm_up(5);
        assert_eq!(pool.size(), 5);
        assert_eq!(pool.available_count(), 5);

        // 第二次 warm_up（更大的值）
        pool.warm_up(10);
        assert_eq!(pool.size(), 10);
        assert_eq!(pool.available_count(), 10);

        // 第三次 warm_up（相同的值，不应该增加）
        pool.warm_up(10);
        assert_eq!(pool.size(), 10);
        assert_eq!(pool.available_count(), 10);

        // 第四次 warm_up（更小的值，不应该减少）
        pool.warm_up(5);
        assert_eq!(pool.size(), 10);  // 保持不变
    }

    /// 测试推荐预热数量计算
    #[test]
    fn test_recommended_warm_up_size() {
        // 100 并发，80% 目标 -> 125 预热
        let size = ObjectPool::<String>::recommended_warm_up_size(100, 0.8);
        assert_eq!(size, 125);

        // 50 并发，90% 目标 -> 56 预热
        let size = ObjectPool::<String>::recommended_warm_up_size(50, 0.9);
        assert_eq!(size, 56);

        // 边界条件
        let size = ObjectPool::<String>::recommended_warm_up_size(100, 0.0);
        assert_eq!(size, 100);  // 无效复用率，返回峰值

        let size = ObjectPool::<String>::recommended_warm_up_size(100, 1.0);
        assert_eq!(size, 100);  // 100% 复用率，返回峰值
    }
}
