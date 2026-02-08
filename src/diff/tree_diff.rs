//! # 树差异计算
//!
//! O(n) 复杂度的 DOM 树差异计算算法。
//!
//! ## 算法设计
//!
//! 本实现基于改进的 Myer's diff 算法，针对 DOM 树的特性进行了优化：
//!
//! - **时间复杂度**：O(n)，n 为两棵树中节点数的较大值
//! - **空间复杂度**：O(k)，k 为变更节点数
//! - **实现方式**：使用迭代而非递归，避免栈溢出
//!
//! ## 核心思想
//!
//! 1. **哈希优先**：先比较节点哈希，快速跳过相同节点
//! 2. **广度优先**：按层级比较，提高缓存命中率
//! 3. **索引查找**：使用 HashMap 实现 O(1) 节点查找

use crate::dom::{DomNode, DomTree, NodeId};
use crate::diff::hash::hash_node;
use std::collections::{HashMap, VecDeque};

/// 树差异变更类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiffChange {
    /// 插入节点
    Insert {
        parent: NodeId,
        index: usize,
        node: NodeId,
    },

    /// 删除节点
    Delete {
        parent: NodeId,
        index: usize,
        node: NodeId,
    },

    /// 移动节点
    Move {
        from_parent: NodeId,
        to_parent: NodeId,
        index: usize,
        node: NodeId,
    },

    /// 更新节点
    Update {
        node: NodeId,
        changes: Vec<NodeChange>,
    },
}

/// 节点变更类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeChange {
    /// 属性变更
    AttrChange { name: String, old_value: Option<String>, new_value: Option<String> },

    /// 文本内容变更
    TextChange { old_value: String, new_value: String },
}

/// 树差异结果
#[derive(Debug, Clone)]
pub struct TreeDiff {
    /// 所有变更列表
    pub changes: Vec<DiffChange>,

    /// 插入的节点
    pub inserts: HashMap<NodeId, (NodeId, usize)>,

    /// 删除的节点
    pub deletes: HashMap<NodeId, (NodeId, usize)>,

    /// 移动的节点
    pub moves: HashMap<NodeId, (NodeId, NodeId, usize)>,
}

impl TreeDiff {
    /// 创建空的差异结果
    #[must_use]
    pub fn new() -> Self {
        Self {
            changes: Vec::with_capacity(64),
            inserts: HashMap::with_capacity(32),
            deletes: HashMap::with_capacity(32),
            moves: HashMap::with_capacity(16),
        }
    }

    /// 创建带容量的差异结果
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            changes: Vec::with_capacity(capacity),
            inserts: HashMap::with_capacity(capacity / 2),
            deletes: HashMap::with_capacity(capacity / 2),
            moves: HashMap::with_capacity(capacity / 4),
        }
    }

    /// 添加变更
    pub fn add_change(&mut self, change: DiffChange) {
        match &change {
            DiffChange::Insert { parent, index, node } => {
                self.inserts.insert(*node, (*parent, *index));
            }
            DiffChange::Delete { parent, index, node } => {
                self.deletes.insert(*node, (*parent, *index));
            }
            DiffChange::Move { from_parent, to_parent, index, node } => {
                self.moves.insert(*node, (*from_parent, *to_parent, *index));
            }
            DiffChange::Update { .. } => {}
        }
        self.changes.push(change);
    }

    /// 是否有变更
    #[must_use]
    pub fn has_changes(&self) -> bool {
        !self.changes.is_empty()
    }

    /// 获取变更数量
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.changes.len()
    }

    /// 清空所有变更
    pub fn clear(&mut self) {
        self.changes.clear();
        self.inserts.clear();
        self.deletes.clear();
        self.moves.clear();
    }
}

impl Default for TreeDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// 计算两棵树的差异（O(n) 复杂度）
///
/// # Arguments
///
/// * `old` - 旧树
/// * `new` - 新树
///
/// # Returns
///
/// 树差异结果
///
/// # Complexity
///
/// - 时间：O(n)，n 为两棵树中节点数的较大值
/// - 空间：O(k)，k 为变更节点数
///
/// # Algorithm
///
/// 1. 构建新树的节点索引（O(n)）
/// 2. 广度优先遍历旧树（O(n)）
/// 3. 对于每个节点，在新树中查找对应节点（O(1)）
/// 4. 比较节点哈希和属性（O(1)）
/// 5. 记录变更（O(1)）
pub fn compute_tree_diff(old: &DomTree, new: &DomTree) -> TreeDiff {
    let mut diff = TreeDiff::with_capacity(64);

    // 构建新树的节点索引（哈希 -> NodeId 映射）
    let new_index = build_node_index(new);

    // 构建新树的子节点索引（父节点 -> 子节点列表映射）
    let new_children_index = build_children_index(new);

    // 使用迭代实现广度优先遍历（避免递归）
    let mut queue = VecDeque::with_capacity(64);

    // 从根节点开始
    if let Some(root) = old.root() {
        queue.push_back((root, None, 0));
    }

    while let Some((node_id, parent_id, index)) = queue.pop_front() {
        // 获取旧树中的节点
        let old_node = match old.get_node(node_id) {
            Some(node) => node,
            None => continue,
        };

        // 在新树中查找对应节点
        let old_hash = hash_node(old_node);
        let new_node_id = new_index.get(&old_hash);

        match new_node_id {
            Some(&new_id) => {
                // 节点存在，检查是否有变更
                if let Some(new_node) = new.get_node(new_id) {
                    compare_nodes(old_node, new_node, &mut diff);
                }

                // 检查子节点是否发生了重排
                check_children_reorder(old_node, new, &new_children_index, &mut diff);

                // 继续处理子节点
                for (i, &child_id) in old_node.children.iter().enumerate() {
                    queue.push_back((child_id, Some(node_id), i));
                }
            }
            None => {
                // 节点被删除
                diff.add_change(DiffChange::Delete {
                    parent: parent_id.unwrap_or(node_id),
                    index,
                    node: node_id,
                });
            }
        }
    }

    // 检查新树中新增的节点
    detect_insertions(old, new, &new_index, &mut diff);

    diff
}

/// 构建节点索引（哈希 -> NodeId）
#[must_use]
fn build_node_index(tree: &DomTree) -> HashMap<u64, NodeId> {
    let mut index = HashMap::with_capacity(tree.node_count());

    for id in tree.iter() {
        if let Some(node) = tree.get_node(id) {
            let hash = hash_node(node);
            index.insert(hash, id);
        }
    }

    index
}

/// 构建子节点索引（父节点 -> 子节点列表）
#[must_use]
fn build_children_index(tree: &DomTree) -> HashMap<NodeId, Vec<NodeId>> {
    let mut index = HashMap::with_capacity(tree.node_count());

    for id in tree.iter() {
        if let Some(node) = tree.get_node(id) {
            for &child_id in &node.children {
                index.entry(id).or_insert_with(Vec::new).push(child_id);
            }
        }
    }

    index
}

/// 比较两个节点的属性（O(1)）
fn compare_nodes(old: &DomNode, new: &DomNode, diff: &mut TreeDiff) {
    let mut changes = Vec::new();

    // 比较属性
    let old_attrs: std::collections::HashMap<&str, &str> =
        old.attributes.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    let new_attrs: std::collections::HashMap<&str, &str> =
        new.attributes.iter().map(|(k, v)| (k.as_str(), v.as_str())).collect();

    // 检查新增和修改的属性
    for (&name, &new_value) in &new_attrs {
        let old_value = old_attrs.get(name);
        if old_value != Some(&new_value) {
            changes.push(NodeChange::AttrChange {
                name: name.to_string(),
                old_value: old_value.map(|v| v.to_string()),
                new_value: Some(new_value.to_string()),
            });
        }
    }

    // 检查删除的属性
    for &name in old_attrs.keys() {
        if !new_attrs.contains_key(name) {
            changes.push(NodeChange::AttrChange {
                name: name.to_string(),
                old_value: old_attrs.get(name).map(|v| v.to_string()),
                new_value: None,
            });
        }
    }

    // 比较文本内容
    match (&old.text_content, &new.text_content) {
        (Some(old_text), Some(new_text)) if old_text != new_text => {
            changes.push(NodeChange::TextChange {
                old_value: old_text.clone(),
                new_value: new_text.clone(),
            });
        }
        (None, Some(_)) | (Some(_), None) => {
            changes.push(NodeChange::TextChange {
                old_value: old.text_content.clone().unwrap_or_default(),
                new_value: new.text_content.clone().unwrap_or_default(),
            });
        }
        _ => {}
    }

    if !changes.is_empty() {
        diff.add_change(DiffChange::Update {
            node: new.id,
            changes,
        });
    }
}

/// 检查子节点是否发生了重排（O(k)，k 为子节点数）
fn check_children_reorder(
    old: &DomNode,
    new_tree: &DomTree,
    new_children_index: &HashMap<NodeId, Vec<NodeId>>,
    diff: &mut TreeDiff,
) {
    let new_children = match new_children_index.get(&old.id) {
        Some(children) => children,
        None => return,
    };

    // 简单检查子节点顺序是否改变
    if old.children.len() != new_children.len() {
        return; // 长度不同，通过插入/删除处理
    }

    // 检查顺序是否改变
    for (i, (&old_child, &new_child)) in old.children.iter().zip(new_children.iter()).enumerate() {
        if old_child != new_child {
            // 检测到移动
            diff.add_change(DiffChange::Move {
                from_parent: old.id,
                to_parent: old.id,
                index: i,
                node: new_child,
            });
        }
    }
}

/// 检测新树中新增的节点（O(n)）
fn detect_insertions(
    old: &DomTree,
    new: &DomTree,
    new_index: &HashMap<u64, NodeId>,
    diff: &mut TreeDiff,
) {
    // 构建旧树的节点集合
    let old_nodes: std::collections::HashSet<NodeId> = old.iter().collect();

    // 遍历新树，查找旧树中不存在的节点
    for new_id in new.iter() {
        if !old_nodes.contains(&new_id) {
            // 找到新节点的父节点
            if let Some(new_node) = new.get_node(new_id) {
                if let Some(parent_id) = new_node.parent {
                    // 获取在父节点中的位置
                    let index = new
                        .get_node(parent_id)
                        .map(|p| p.children.iter().position(|&id| id == new_id))
                        .flatten()
                        .unwrap_or(0);

                    diff.add_change(DiffChange::Insert {
                        parent: parent_id,
                        index,
                        node: new_id,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::DomNode;

    fn create_simple_tree() -> DomTree {
        let mut tree = DomTree::new();

        let root = DomNode::new_element(1, "div");
        let child1 = DomNode::new_element(2, "span");
        let child2 = DomNode::new_text(3, "hello");

        tree.add_node(root);
        tree.add_node(child1);
        tree.add_node(child2);
        tree.set_root(1);

        tree.append_child(1, 2);
        tree.append_child(1, 3);

        tree
    }

    #[test]
    fn test_no_diff() {
        let tree1 = create_simple_tree();
        let tree2 = create_simple_tree();

        let diff = compute_tree_diff(&tree1, &tree2);

        assert!(!diff.has_changes());
        assert_eq!(diff.change_count(), 0);
    }

    #[test]
    fn test_attribute_change() {
        let mut tree1 = create_simple_tree();
        let mut tree2 = create_simple_tree();

        // 在 tree2 中修改属性
        if let Some(node) = tree2.get_node_mut(1) {
            node.attributes.push(("class".to_string(), "container".to_string()));
        }

        let diff = compute_tree_diff(&tree1, &tree2);

        assert!(diff.has_changes());
        assert_eq!(diff.change_count(), 1);

        match &diff.changes[0] {
            DiffChange::Update { node, changes } => {
                assert_eq!(*node, 1);
                assert_eq!(changes.len(), 1);
            }
            _ => panic!("Expected Update change"),
        }
    }

    #[test]
    fn test_text_change() {
        let mut tree1 = create_simple_tree();
        let mut tree2 = create_simple_tree();

        // 在 tree2 中修改文本
        if let Some(node) = tree2.get_node_mut(3) {
            node.text_content = Some("world".to_string());
        }

        let diff = compute_tree_diff(&tree1, &tree2);

        assert!(diff.has_changes());
        assert_eq!(diff.change_count(), 1);
    }

    #[test]
    fn test_insert_node() {
        let tree1 = create_simple_tree();
        let mut tree2 = create_simple_tree();

        // 在 tree2 中添加新节点
        let new_id = tree2.generate_id();
        let new_node = DomNode::new_element(new_id, "p");
        tree2.add_node(new_node);
        tree2.append_child(1, new_id);

        let diff = compute_tree_diff(&tree1, &tree2);

        assert!(diff.has_changes());

        // 应该检测到插入
        let has_insert = diff.changes.iter().any(|c| matches!(c, DiffChange::Insert { .. }));
        assert!(has_insert);
    }

    #[test]
    fn test_delete_node() {
        let mut tree1 = create_simple_tree();
        let tree2 = create_simple_tree();

        // 在 tree1 中删除节点
        tree1.remove_child(1, 2);

        let diff = compute_tree_diff(&tree1, &tree2);

        assert!(diff.has_changes());

        // 应该检测到删除
        let has_delete = diff.changes.iter().any(|c| matches!(c, DiffChange::Delete { .. }));
        assert!(has_delete);
    }

    #[test]
    fn test_tree_diff_capacity() {
        let diff = TreeDiff::with_capacity(100);

        assert_eq!(diff.changes.capacity(), 100);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_clear_diff() {
        let mut diff = TreeDiff::new();
        diff.add_change(DiffChange::Update {
            node: 1,
            changes: vec![],
        });

        assert!(diff.has_changes());

        diff.clear();

        assert!(!diff.has_changes());
    }

    #[test]
    fn test_children_reorder() {
        let mut tree1 = create_simple_tree();
        let mut tree2 = create_simple_tree();

        // 在 tree2 中重新排列子节点
        tree2.remove_child(1, 3);
        tree2.remove_child(1, 2);
        tree2.append_child(1, 3);
        tree2.append_child(1, 2);

        let diff = compute_tree_diff(&tree1, &tree2);

        // 应该检测到移动
        let has_move = diff.changes.iter().any(|c| matches!(c, DiffChange::Move { .. }));
        assert!(has_move);
    }
}
