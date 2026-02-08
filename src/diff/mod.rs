//! # DOM 差分算法模块
//!
//! O(n) 复杂度的 DOM 树差异计算。
//!
//! ## 模块结构
//!
//! - [`ops_generator`] - 操作序列生成器
//! - [`tree_diff`] - 树差异计算
//! - [`hash`] - 快速节点哈希

pub mod ops_generator;
pub mod tree_diff;
pub mod hash;

// 导出核心类型
pub use ops_generator::{DomOp, OpsGenerator, MutationRecord, MutationType, BatchOp};
pub use tree_diff::{DiffChange, TreeDiff, compute_tree_diff};
pub use hash::{hash_node, NodeHash};
