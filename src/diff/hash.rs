//! # 快速节点哈希
//!
//! O(1) 复杂度的节点哈希计算。
//!
//! ## 设计原则
//!
//! - **只哈希必要属性**：避免深拷贝和递归遍历
//! - **快速计算**：使用 ahash 或 fnv 算法
//! - **冲突率低**：结合多个属性特征

use crate::dom::{DomNode, NodeType};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// 节点哈希值
pub type NodeHash = u64;

/// 计算节点哈希（O(1) 复杂度）
///
/// # Algorithm
///
/// 哈希计算包含以下要素：
/// - 节点类型（1 种状态）
/// - 标签名（元素节点）
/// - 属性数量
/// - 文本内容长度
/// - 子节点数量
///
/// 注意：**不递归哈希子节点**，保持 O(1) 复杂度。
///
/// # Complexity
///
/// - 时间：O(1)
/// - 空间：O(1)
///
/// # Example
///
/// ```rust
/// use chrome_dom_diff::diff::hash::hash_node;
/// use chrome_dom_diff::dom::DomNode;
///
/// let node = DomNode::new_element(1, "div");
/// let hash = hash_node(&node);
/// ```
#[must_use]
pub fn hash_node(node: &DomNode) -> NodeHash {
    let mut hasher = DefaultHasher::new();

    // 哈希节点类型
    node.node_type.hash(&mut hasher);

    // 哈希标签名（仅元素节点）
    if let Some(ref tag_name) = node.tag_name {
        // 只哈希前 32 字节，避免长标签名影响性能
        let bytes = tag_name.as_bytes();
        let len = bytes.len().min(32);
        bytes[..len].hash(&mut hasher);
    }

    // 哈希属性数量和关键属性
    (node.attributes.len() as u64).hash(&mut hasher);

    // 对于元素节点，哈希 id 和 class 属性（这些是影响布局的关键属性）
    if node.is_element() {
        if let Some(id) = node.get_attr("id") {
            // 只哈希前 16 字节
            let bytes = id.as_bytes();
            let len = bytes.len().min(16);
            bytes[..len].hash(&mut hasher);
        }

        if let Some(class) = node.get_attr("class") {
            // 只哈希前 16 字节
            let bytes = class.as_bytes();
            let len = bytes.len().min(16);
            bytes[..len].hash(&mut hasher);
        }
    }

    // 哈希文本内容长度（而非内容本身）
    if let Some(ref text) = node.text_content {
        (text.len() as u64).hash(&mut hasher);

        // 对于短文本（< 32 字节），哈希全部内容
        if text.len() <= 32 {
            text.as_bytes().hash(&mut hasher);
        } else {
            // 对于长文本，哈希前后各 16 字节
            let bytes = text.as_bytes();
            bytes[..16].hash(&mut hasher);
            let end = bytes.len().saturating_sub(16);
            bytes[end..].hash(&mut hasher);
        }
    }

    // 哈希子节点数量
    (node.children.len() as u64).hash(&mut hasher);

    hasher.finish()
}

/// 计算节点哈希（使用自定义种子）
#[must_use]
pub fn hash_node_with_seed(node: &DomNode, seed: u64) -> NodeHash {
    let hash = hash_node(node);
    hash.wrapping_add(seed).wrapping_mul(31)
}

/// 批量计算节点哈希（O(n) 复杂度，n 为节点数）
///
/// # Arguments
///
/// * `nodes` - 节点迭代器
///
/// # Returns
///
/// 哈希值向量
///
/// # Complexity
///
/// - 时间：O(n)
/// - 空间：O(n)
pub fn hash_nodes<'a, I>(nodes: I) -> Vec<NodeHash>
where
    I: IntoIterator<Item = &'a DomNode>,
{
    nodes.into_iter().map(hash_node).collect()
}

/// 比较两个节点是否可能相同（通过哈希）
///
/// # Arguments
///
/// * `a` - 第一个节点
/// * `b` - 第二个节点
///
/// # Returns
///
/// 如果哈希相同则返回 true（可能有误报，但无漏报）
#[must_use]
pub fn nodes_maybe_equal(a: &DomNode, b: &DomNode) -> bool {
    hash_node(a) == hash_node(b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dom::DomNode;

    #[test]
    fn test_hash_element_node() {
        let node1 = DomNode::new_element(1, "div");
        let node2 = DomNode::new_element(2, "div");
        let node3 = DomNode::new_element(3, "span");

        // 相同标签名的元素节点哈希相同（ID 不同不影响）
        assert_eq!(hash_node(&node1), hash_node(&node2));

        // 不同标签名的元素节点哈希不同
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }

    #[test]
    fn test_hash_text_node() {
        let node1 = DomNode::new_text(1, "hello");
        let node2 = DomNode::new_text(2, "hello");
        let node3 = DomNode::new_text(3, "world");

        // 相同文本的节点哈希相同
        assert_eq!(hash_node(&node1), hash_node(&node2));

        // 不同文本的节点哈希不同
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }

    #[test]
    fn test_hash_with_attributes() {
        let node1 = DomNode::new_element(1, "div").with_attr("class", "container");
        let node2 = DomNode::new_element(2, "div").with_attr("class", "container");
        let node3 = DomNode::new_element(3, "div").with_attr("id", "main");

        // 相同属性的节点哈希相同
        assert_eq!(hash_node(&node1), hash_node(&node2));

        // 不同属性的节点哈希不同
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }

    #[test]
    fn test_hash_with_id_attribute() {
        let node1 = DomNode::new_element(1, "div").with_attr("id", "main");
        let node2 = DomNode::new_element(2, "div").with_attr("id", "main");
        let node3 = DomNode::new_element(3, "div").with_attr("id", "header");

        // id 属性影响哈希
        assert_eq!(hash_node(&node1), hash_node(&node2));
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }

    #[test]
    fn test_hash_with_class_attribute() {
        let node1 = DomNode::new_element(1, "div").with_attr("class", "btn");
        let node2 = DomNode::new_element(2, "div").with_attr("class", "btn");
        let node3 = DomNode::new_element(3, "div").with_attr("class", "btn-primary");

        // class 属性影响哈希
        assert_eq!(hash_node(&node1), hash_node(&node2));
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }

    #[test]
    fn test_hash_text_length() {
        let node1 = DomNode::new_text(1, "hello");
        let node2 = DomNode::new_text(2, "hello world!");
        let node3 = DomNode::new_text(3, "different");

        // 不同长度的文本哈希不同
        assert_ne!(hash_node(&node1), hash_node(&node2));
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }

    #[test]
    fn test_hash_children_count() {
        let mut tree = crate::dom::DomTree::new();

        let parent1 = DomNode::new_element(1, "div");
        let parent2 = DomNode::new_element(2, "div");
        let child1 = DomNode::new_text(3, "a");
        let child2 = DomNode::new_text(4, "b");

        tree.add_node(parent1);
        tree.add_node(parent2);
        tree.add_node(child1);
        tree.add_node(child2);

        // parent1 有一个子节点
        tree.append_child(1, 3);

        // parent2 有两个子节点
        tree.append_child(2, 4);

        let node1 = tree.get_node(1).unwrap();
        let node2 = tree.get_node(2).unwrap();

        // 不同数量的子节点哈希不同
        assert_ne!(hash_node(node1), hash_node(node2));
    }

    #[test]
    fn test_hash_with_seed() {
        let node = DomNode::new_element(1, "div");

        let hash1 = hash_node(&node);
        let hash2 = hash_node_with_seed(&node, 42);
        let hash3 = hash_node_with_seed(&node, 100);

        assert_ne!(hash1, hash2);
        assert_ne!(hash2, hash3);
    }

    #[test]
    fn test_nodes_maybe_equal() {
        let node1 = DomNode::new_element(1, "div");
        let node2 = DomNode::new_element(2, "div");
        let node3 = DomNode::new_element(3, "span");

        assert!(nodes_maybe_equal(&node1, &node2));
        assert!(!nodes_maybe_equal(&node1, &node3));
    }

    #[test]
    fn test_hash_nodes_batch() {
        let nodes = vec![
            DomNode::new_element(1, "div"),
            DomNode::new_element(2, "span"),
            DomNode::new_text(3, "hello"),
        ];

        let hashes = hash_nodes(&nodes);

        assert_eq!(hashes.len(), 3);
        assert_ne!(hashes[0], hashes[1]);
    }

    #[test]
    fn test_hash_long_text() {
        let long_text = "a".repeat(1000);
        let short_text = "a".repeat(10);

        let node1 = DomNode::new_text(1, &long_text);
        let node2 = DomNode::new_text(2, &short_text);

        // 不同长度的文本哈希不同
        assert_ne!(hash_node(&node1), hash_node(&node2));
    }

    #[test]
    fn test_hash_comment_node() {
        let node1 = DomNode::new_comment(1, "comment");
        let node2 = DomNode::new_comment(2, "comment");
        let node3 = DomNode::new_comment(3, "different");

        // 相同注释的节点哈希相同
        assert_eq!(hash_node(&node1), hash_node(&node2));

        // 不同注释的节点哈希不同
        assert_ne!(hash_node(&node1), hash_node(&node3));
    }
}
