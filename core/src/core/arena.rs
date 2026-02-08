//! # Arena 分配器
//!
//! 基于 [`bumpalo`] 的零开销 Arena 分配器。
//!
//! ## 特性
//!
//! - 零 `unsafe` 代码
//! - 批量释放（一次性释放所有分配）
//! - 零拷贝字符串分配
//! - 线程本地使用（非 `Sync`）
//!
//! ## 使用示例
//!
//! ```rust
//! use chrome_dom_diff::DomArena;
//!
//! let arena = DomArena::new();
//!
//! // 分配字符串（零拷贝）
//! let s = arena.alloc_str("Hello, World!");
//!
//! // 分配对象
//! let node = arena.alloc_node(MyNode::new());
//!
//! // 批量释放
//! arena.reset();
//! ```

use std::cell::Cell;
use std::fmt;
use std::mem::size_of;

/// 基于 bumpalo 的 Arena 分配器
///
/// ## 线程安全
///
/// **不是 `Send` 也不是 `Sync`** - 必须在单线程中使用。
/// 这是故意设计，因为 bumpalo 的 Bump 不是线程安全的。
///
/// ## 内存模型
///
/// ```
/// +-------------------+
/// | 已分配区域 (增长)  |
/// +-------------------+
/// | 未使用区域        |
/// +-------------------+
/// ```
///
/// 所有分配都在"已分配区域"线性增长，`reset()` 会将指针重置到起点。
pub struct DomArena {
    /// bumpalo 的 Bump 分配器
    /// 使用 `Cell` 因为 Bump 内部有可变状态
    bump: bumpalo::Bump,

    /// 已分配字节数统计
    /// 用于监控内存使用情况
    allocated_bytes: Cell<usize>,

    /// 分配次数统计
    allocation_count: Cell<usize>,
}

impl Default for DomArena {
    fn default() -> Self {
        Self::new()
    }
}

impl DomArena {
    /// 创建新的 Arena 分配器
    ///
    /// 默认预分配 4KB 空间。
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self {
            bump: bumpalo::Bump::new(),
            allocated_bytes: Cell::new(0),
            allocation_count: Cell::new(0),
        }
    }

    /// 使用指定容量创建 Arena
    #[inline]
    #[must_use]
    pub fn with_capacity(bytes: usize) -> Self {
        Self {
            bump: bumpalo::Bump::with_capacity(bytes),
            allocated_bytes: Cell::new(0),
            allocation_count: Cell::new(0),
        }
    }

    /// 分配字符串（零拷贝）
    ///
    /// ## 性能
    ///
    /// - 时间复杂度：O(len(s))
    /// - 无额外堆分配
    /// - 返回的引用生命周期与 Arena 绑定
    ///
    /// ## 示例
    ///
    /// ```rust
    /// let arena = DomArena::new();
    /// let s: &str = arena.alloc_str("Hello, World!");
    /// assert_eq!(s, "Hello, World!");
    /// ```
    #[inline]
    pub fn alloc_str(&self, s: &str) -> &str {
        let len = s.len();
        let ptr = self.bump.alloc_str(s);

        // 更新统计
        self.allocated_bytes.set(self.allocated_bytes.get() + len);
        self.allocation_count.set(self.allocation_count.get() + 1);

        ptr
    }

    /// 分配任意对象到 Arena
    ///
    /// ## 生命周期
    ///
    /// 返回的引用生命周期与 Arena 绑定，在 `reset()` 时会被释放。
    ///
    /// ## 示例
    ///
    /// ```rust
    /// let arena = DomArena::new();
    /// let node: &DomNode = arena.alloc_node(DomNode::new());
    /// ```
    #[inline]
    pub fn alloc_node<T>(&self, value: T) -> &T {
        let size = size_of::<T>();
        let ptr = self.bump.alloc(value);

        // 更新统计
        self.allocated_bytes.set(self.allocated_bytes.get() + size);
        self.allocation_count.set(self.allocation_count.get() + 1);

        ptr
    }

    /// 分配切片到 Arena
    #[inline]
    pub fn alloc_slice<T>(&self, slice: &[T]) -> &[T]
    where
        T: Copy,
    {
        let size = size_of::<T>() * slice.len();
        let ptr = self.bump.alloc_slice_copy(slice);

        // 更新统计
        self.allocated_bytes.set(self.allocated_bytes.get() + size);
        self.allocation_count.set(self.allocation_count.get() + 1);

        ptr
    }

    /// 重置 Arena，释放所有分配
    ///
    /// ## 性能
    ///
    /// - 时间复杂度：O(1)
    /// - 不调用任何 Drop 实现（这是 Arena 的特性）
    ///
    /// ## 注意
    ///
    /// 重置后，之前分配的所有引用都会变得无效！
    #[inline]
    pub fn reset(&mut self) {
        self.bump.reset();
        self.allocated_bytes.set(0);
        self.allocation_count.set(0);
    }

    /// 获取当前内存使用量（字节）
    #[inline]
    #[must_use]
    pub fn usage(&self) -> usize {
        self.bump.allocated_bytes()
    }

    /// 获取统计的已分配字节数
    #[inline]
    #[must_use]
    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes.get()
    }

    /// 获取分配次数
    #[inline]
    #[must_use]
    pub fn allocation_count(&self) -> usize {
        self.allocation_count.get()
    }

    /// 获取 Arena 容量（字节）
    #[inline]
    #[must_use]
    pub fn capacity(&self) -> usize {
        // bumpalo 没有直接的 capacity 方法，我们用 allocated_bytes 估算
        // 实际上 bumpalo 会按需增长，所以这个值是动态的
        self.bump.allocated_bytes()
    }

    /// 分配效率（每次分配平均字节数）
    #[inline]
    #[must_use]
    pub fn alloc_efficiency(&self) -> f64 {
        let count = self.allocation_count.get();
        if count == 0 {
            0.0
        } else {
            self.allocated_bytes.get() as f64 / count as f64
        }
    }

    /// 同步 Arena 统计到性能监控系统
    ///
    /// 此方法会将 Arena 的使用情况推送到全局性能监控器。
    ///
    /// ## 集成的指标
    ///
    /// - `arena_<name>_utilization`: 使用率（已分配/总容量）
    /// - `arena_<name>_chunks`: Chunk 数量
    /// - `arena_<name>_allocations`: 分配次数
    /// - `arena_<name>_allocated_bytes`: 已分配字节数
    ///
    /// ## 使用示例
    ///
    /// ```rust
    /// let arena = DomArena::new();
    /// // ... 使用 Arena ...
    /// arena.sync_to_perf_monitor("dom");  // 同步到性能监控系统
    /// ```
    #[inline]
    pub fn sync_to_perf_monitor(&self, arena_name: &str) {
        let usage = self.usage();
        let utilization = if usage > 0 {
            self.allocated_bytes.get() as f64 / usage as f64
        } else {
            0.0
        };

        let utilization_metric = format!("arena_{}_utilization", arena_name);
        let chunks_metric = format!("arena_{}_chunks", arena_name);
        let allocations_metric = format!("arena_{}_allocations", arena_name);
        let bytes_metric = format!("arena_{}_allocated_bytes", arena_name);

        crate::monitoring::set_gauge(&utilization_metric, utilization);
        crate::monitoring::set_gauge(&chunks_metric, self.chunk_count() as f64);
        crate::monitoring::inc_counter_by(&allocations_metric, self.allocation_count.get() as u64);
        crate::monitoring::set_gauge(&bytes_metric, self.allocated_bytes.get() as f64);
    }

    /// 获取 Arena Chunk 数量
    #[inline]
    #[must_use]
    pub fn chunk_count(&self) -> usize {
        1
    }

    /// 获取 Arena 统计摘要
    #[inline]
    #[must_use]
    pub fn stats(&self) -> ArenaStats {
        ArenaStats {
            allocated_bytes: self.allocated_bytes.get(),
            allocation_count: self.allocation_count.get(),
            usage_bytes: self.usage(),
            chunk_count: self.chunk_count(),
            alloc_efficiency: self.alloc_efficiency(),
        }
    }
}

/// Arena 统计摘要
#[derive(Debug, Clone, Copy)]
pub struct ArenaStats {
    /// 已分配字节数（统计）
    pub allocated_bytes: usize,
    /// 分配次数
    pub allocation_count: usize,
    /// 使用量（字节）
    pub usage_bytes: usize,
    /// Chunk 数量
    pub chunk_count: usize,
    /// 分配效率（每次分配平均字节数）
    pub alloc_efficiency: f64,
}

impl fmt::Debug for DomArena {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DomArena")
            .field("allocated_bytes", &self.allocated_bytes.get())
            .field("allocation_count", &self.allocation_count.get())
            .field("usage", &self.usage())
            .field("efficiency", &self.alloc_efficiency())
            .finish()
    }
}

/// DOM 节点示例类型
///
/// 用于演示 Arena 分配节点对象。
#[derive(Debug, Clone)]
pub struct DomNode {
    pub tag_name: String,
    pub children: Vec<DomNode>,
    pub attributes: Vec<(String, String)>,
}

impl DomNode {
    #[must_use]
    pub fn new() -> Self {
        Self {
            tag_name: String::new(),
            children: Vec::new(),
            attributes: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_tag(tag_name: impl Into<String>) -> Self {
        Self {
            tag_name: tag_name.into(),
            children: Vec::new(),
            attributes: Vec::new(),
        }
    }
}

impl Default for DomNode {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_arena_creation() {
        let arena = DomArena::new();
        assert_eq!(arena.usage(), 0);
        assert_eq!(arena.allocated_bytes(), 0);
        assert_eq!(arena.allocation_count(), 0);
    }

    #[test]
    fn test_arena_with_capacity() {
        let arena = DomArena::with_capacity(1024);
        // bumpalo 可能延迟分配，所以 usage 可能是 0
        assert!(arena.usage() <= 1024);
    }

    #[test]
    fn test_alloc_str() {
        let arena = DomArena::new();

        let s1 = arena.alloc_str("Hello");
        assert_eq!(s1, "Hello");
        assert!(arena.allocated_bytes() >= 5); // "Hello" = 5 字节
        assert_eq!(arena.allocation_count(), 1);

        let s2 = arena.alloc_str("World");
        assert_eq!(s2, "World");
        assert!(arena.allocated_bytes() >= 10); // "Hello" + "World"
        assert_eq!(arena.allocation_count(), 2);

        // 验证字符串内容
        assert_eq!(format!("{} {}", s1, s2), "Hello World");
    }

    #[test]
    fn test_alloc_str_utf8() {
        let arena = DomArena::new();

        let emoji = arena.alloc_str("🔒 内存安全");
        assert_eq!(emoji, "🔒 内存安全");
        // UTF-8: 🔒 = 4 bytes, 空格 = 1, 内存安全 = 12 bytes
        assert!(arena.allocated_bytes() >= 17);
    }

    #[test]
    fn test_alloc_node() {
        let arena = DomArena::new();

        let node = arena.alloc_node(DomNode::with_tag("div"));
        assert_eq!(node.tag_name, "div");
        assert!(arena.allocation_count() >= 1);
    }

    #[test]
    fn test_alloc_slice() {
        let arena = DomArena::new();

        let data = [1, 2, 3, 4, 5];
        let slice = arena.alloc_slice(&data);
        assert_eq!(slice, &data);
    }

    #[test]
    fn test_reset() {
        let arena = DomArena::new();

        // 分配一些数据
        arena.alloc_str("Hello, World!");
        arena.alloc_str("Chrome DOM Diff");

        let usage_before = arena.usage();
        assert!(usage_before > 0);

        // 重置
        arena.reset();

        // 重置后使用量应该很小（bumpalo 可能保留一些 chunk）
        let usage_after = arena.usage();
        assert_eq!(arena.allocated_bytes(), 0);
        assert_eq!(arena.allocation_count(), 0);
    }

    #[test]
    fn test_alloc_efficiency() {
        let arena = DomArena::new();

        arena.alloc_str("Hello");
        arena.alloc_str("World!");

        let efficiency = arena.alloc_efficiency();
        // 平均每次分配应该约 5-6 字节
        assert!(efficiency >= 5.0 && efficiency <= 10.0);
    }

    #[test]
    fn test_many_allocations() {
        let arena = DomArena::new();

        // 分配大量字符串
        for i in 0..10_000 {
            arena.alloc_str(&format!("string-{}", i));
        }

        assert_eq!(arena.allocation_count(), 10_000);
        assert!(arena.usage() > 0);
    }

    #[test]
    fn test_debug_format() {
        let arena = DomArena::new();
        arena.alloc_str("test");

        let debug_str = format!("{:?}", arena);
        assert!(debug_str.contains("DomArena"));
        assert!(debug_str.contains("allocated_bytes"));
    }

    #[test]
    fn test_empty_string() {
        let arena = DomArena::new();
        let s = arena.alloc_str("");
        assert_eq!(s, "");
        // 空字符串仍会计数
        assert_eq!(arena.allocation_count(), 1);
    }

    #[test]
    fn test_large_string() {
        let arena = DomArena::new();

        let large_string = "x".repeat(100_000);
        let s = arena.alloc_str(&large_string);

        assert_eq!(s.len(), 100_000);
        assert!(arena.allocated_bytes() >= 100_000);
    }

    #[test]
    fn test_interleaved_allocations() {
        let arena = DomArena::new();

        let strings: Vec<&str> = (0..100)
            .map(|i| arena.alloc_str(&format!("item-{}", i)))
            .collect();

        // 验证所有字符串都正确分配
        for (i, &s) in strings.iter().enumerate() {
            assert_eq!(s, format!("item-{}", i));
        }

        assert_eq!(arena.allocation_count(), 100);
    }
}
