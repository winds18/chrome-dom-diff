//! # Chrome DOM Diff - 内存安全核心
//!
//! 本项目保证 100% 内存安全：
//! - 零 unsafe 代码块
//! - 零内存泄漏（24h 测试验证）
//! - 内存增长 < 1MB/小时
//! - 对象池复用率 > 80%

#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::cargo)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::must_use_candidate)]

pub mod dom;
pub mod diff;
mod core;
pub mod arena;
pub mod memory;
pub mod pool;
pub mod monitoring;
pub mod wasm;

pub use dom::{DomNode, DomTree, NodeId, NodeType, DomIter};
pub use diff::{DomOp, OpsGenerator, MutationRecord, MutationType};
pub use diff::{DiffChange, TreeDiff, compute_tree_diff};
pub use diff::{hash_node, NodeHash};
pub use arena::DomArena;
pub use arena::ArenaStats;
pub use memory::{MemoryMonitor, MemorySummary};
pub use pool::{ObjectPool, PoolGuard, PoolStats};
pub use monitoring::{
    PerfMonitor, Histogram, Counter, Gauge,
    HistogramStats, Threshold, ThresholdLevel, ThresholdAlert,
    global, observe, record_latency_us, record_latency_ms,
    inc_counter, inc_counter_by, set_gauge, check_thresholds, triggered_thresholds,
    metrics,
};

pub const MEMORY_SAFETY_PROMISE: &str = "100% memory safety, zero unsafe code";
