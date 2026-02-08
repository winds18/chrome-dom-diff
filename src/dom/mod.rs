//! # DOM 核心数据结构
//!
//! 内存安全的 DOM 树表示，使用 Arena 分配避免递归引用。
//!
//! ## 设计原则
//!
//! - **零递归引用**：使用索引而非引用，避免循环依赖
//! - **Arena 分配**：批量释放，减少碎片
//! - **O(1) 访问**：通过 HashMap 实现快速节点查找
//! - **借用检查友好**：清晰的生命周期标注

use std::collections::HashMap;

/// 节点 ID（64 位，支持 2^64 个节点）
pub type NodeId = u64;

/// 节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeType {
    /// 元素节点（如 <div>）
    Element,
    /// 文本节点
    Text,
    /// 注释节点
    Comment,
    /// CDATA 节点
    CData,
    /// 文档节点
    Document,
}

/// 属性变更记录
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttrChange {
    pub name: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
}

/// DOM 节点（轻量级，不包含子节点引用）
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DomNode {
    /// 节点 ID
    pub id: NodeId,
    /// 节点类型
    pub node_type: NodeType,
    /// 标签名（仅元素节点）
    pub tag_name: Option<String>,
    /// 文本内容（文本/注释节点）
    pub text_content: Option<String>,
    /// 属性列表
    pub attributes: Vec<(String, String)>,
    /// 父节点 ID
    pub parent: Option<NodeId>,
    /// 子节点 ID 列表（有序）
    pub children: Vec<NodeId>,
    /// 前一个兄弟节点
    pub prev_sibling: Option<NodeId>,
    /// 下一个兄弟节点
    pub next_sibling: Option<NodeId>,
}

impl DomNode {
    /// 创建新的元素节点
    #[must_use]
    pub fn new_element(id: NodeId, tag_name: impl Into<String>) -> Self {
        Self {
            id,
            node_type: NodeType::Element,
            tag_name: Some(tag_name.into()),
            text_content: None,
            attributes: Vec::with_capacity(4),
            parent: None,
            children: Vec::with_capacity(8),
            prev_sibling: None,
            next_sibling: None,
        }
    }

    /// 创建新的文本节点
    #[must_use]
    pub fn new_text(id: NodeId, text: impl Into<String>) -> Self {
        Self {
            id,
            node_type: NodeType::Text,
            tag_name: None,
            text_content: Some(text.into()),
            attributes: Vec::new(),
            parent: None,
            children: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        }
    }

    /// 创建新的注释节点
    #[must_use]
    pub fn new_comment(id: NodeId, text: impl Into<String>) -> Self {
        Self {
            id,
            node_type: NodeType::Comment,
            tag_name: None,
            text_content: Some(text.into()),
            attributes: Vec::new(),
            parent: None,
            children: Vec::new(),
            prev_sibling: None,
            next_sibling: None,
        }
    }

    /// 获取属性值
    #[must_use]
    pub fn get_attr(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }

    /// 设置属性（返回新节点，零拷贝）
    #[must_use]
    pub fn with_attr(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let name = name.into();
        // 移除旧值
        self.attributes.retain(|(k, _)| k != &name);
        // 添加新值
        self.attributes.push((name, value.into()));
        self
    }

    /// 添加子节点 ID
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// 是否为叶子节点
    #[must_use]
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    /// 是否为元素节点
    #[must_use]
    pub fn is_element(&self) -> bool {
        self.node_type == NodeType::Element
    }

    /// 是否为文本节点
    #[must_use]
    pub fn is_text(&self) -> bool {
        self.node_type == NodeType::Text
    }
}

/// DOM 树（使用 Arena 存储）
#[derive(Debug, Clone)]
pub struct DomTree {
    /// 所有节点的存储（使用索引访问）
    nodes: HashMap<NodeId, DomNode>,
    /// 根节点 ID
    root_id: Option<NodeId>,
    /// 下一个可用的节点 ID
    next_id: NodeId,
}

impl Default for DomTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DomTree {
    /// 创建空的 DOM 树
    #[must_use]
    pub fn new() -> Self {
        Self {
            nodes: HashMap::with_capacity(256),
            root_id: None,
            next_id: 1,
        }
    }

    /// 创建新的 DOM 树，带预分配容量
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            nodes: HashMap::with_capacity(capacity),
            root_id: None,
            next_id: 1,
        }
    }

    /// 生成新的节点 ID
    #[must_use]
    pub fn generate_id(&mut self) -> NodeId {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    /// 添加节点到树（O(1)）
    pub fn add_node(&mut self, node: DomNode) -> NodeId {
        let id = node.id;
        self.nodes.insert(id, node);
        id
    }

    /// 获取节点（O(1)）
    #[must_use]
    pub fn get_node(&self, id: NodeId) -> Option<&DomNode> {
        self.nodes.get(&id)
    }

    /// 获取可变节点（O(1)）
    pub fn get_node_mut(&mut self, id: NodeId) -> Option<&mut DomNode> {
        self.nodes.get_mut(&id)
    }

    /// 设置根节点
    pub fn set_root(&mut self, id: NodeId) {
        self.root_id = Some(id);
        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = None;
        }
    }

    /// 获取根节点 ID
    #[must_use]
    pub fn root(&self) -> Option<NodeId> {
        self.root_id
    }

    /// 迭代遍历树（O(n)，使用显式栈，禁止递归）
    pub fn iter(&self) -> DomIter<'_> {
        DomIter::new(self)
    }

    /// 获取节点总数
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 附加子节点到父节点
    /// 附加子节点到父节点
    pub fn append_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        // 分步获取可变引用，避免借用冲突
        
        // 检查节点是否存在
        if !self.nodes.contains_key(&parent_id) || !self.nodes.contains_key(&child_id) {
            return false;
        }

        // 提取 parent 的 children 长度
        let child_index = match self.nodes.get(&parent_id) {
            Some(parent) => parent.children.len(),
            None => return false,
        };

        // 1. 更新 parent 的 children
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
        }

        // 2. 更新 child 的 parent
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = Some(parent_id);
        }

        // 3. 更新兄弟节点关系
        if child_index > 0 {
            if let Some(parent) = self.nodes.get(&parent_id) {
                let prev_id = parent.children[child_index - 1];
                
                // 更新前一个兄弟的 next_sibling
                if let Some(prev) = self.nodes.get_mut(&prev_id) {
                    prev.next_sibling = Some(child_id);
                }
                
                // 更新 child 的 prev_sibling
                if let Some(child) = self.nodes.get_mut(&child_id) {
                    child.prev_sibling = Some(prev_id);
                }
            }
        }

        // 4. 清除 child 的 next_sibling
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.next_sibling = None;
        }

        true
    }

    /// 移除子节点
    pub fn remove_child(&mut self, parent_id: NodeId, child_id: NodeId) -> bool {
        // 先检查 parent 和 child 是否存在
        if !self.nodes.contains_key(&parent_id) || !self.nodes.contains_key(&child_id) {
            return false;
        }

        // 获取 parent 并提取需要的信息
        let (pos, prev_id, next_id) = {
            let parent = match self.nodes.get(&parent_id) {
                Some(p) => p,
                None => return false,
            };

            // 查找子节点位置
            let pos = match parent.children.iter().position(|&id| id == child_id) {
                Some(p) => p,
                None => return false,
            };

            // 提取前一个和后一个兄弟节点的 ID
            let prev_id = if pos > 0 { Some(parent.children[pos - 1]) } else { None };
            let next_id = if pos + 1 < parent.children.len() { Some(parent.children[pos + 1]) } else { None };

            (pos, prev_id, next_id)
        };

        // 现在可以安全地修改多个节点
        // 1. 从 parent 的 children 中移除 child
        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.remove(pos);
        }

        // 2. 更新前一个兄弟节点的 next_sibling
        if let Some(prev_id) = prev_id {
            if let Some(prev) = self.nodes.get_mut(&prev_id) {
                prev.next_sibling = next_id;
            }
        }

        // 3. 更新后一个兄弟节点的 prev_sibling
        if let Some(next_id) = next_id {
            if let Some(next) = self.nodes.get_mut(&next_id) {
                next.prev_sibling = prev_id;
            }
        }

        // 4. 清除子节点的父引用和兄弟引用
        if let Some(child) = self.nodes.get_mut(&child_id) {
            child.parent = None;
            child.prev_sibling = None;
            child.next_sibling = None;
        }

        true
    }

    /// 克隆子树（深拷贝，用于测试）
    pub fn clone_subtree(&mut self, root_id: NodeId) -> Option<NodeId> {
        let root = self.get_node(root_id)?.clone();
        let new_root_id = self.generate_id();

        // 使用迭代实现子树克隆（避免递归）
        let mut stack = vec![(root_id, new_root_id, root)];

        while let Some((old_id, new_id, node)) = stack.pop() {
            // 添加节点
            self.nodes.insert(new_id, node);

            // 处理子节点
            for &child_id in &self.get_node(old_id)?.children.clone() {
                let child = self.get_node(child_id)?.clone();
                let new_child_id = self.generate_id();
                self.nodes.get_mut(&new_id)?.add_child(new_child_id);
                stack.push((child_id, new_child_id, child));
            }
        }

        Some(new_root_id)
    }
}

/// DOM 树迭代器（使用迭代实现，零递归）
pub struct DomIter<'a> {
    tree: &'a DomTree,
    stack: Vec<NodeId>,
}

impl<'a> DomIter<'a> {
    #[must_use]
    pub fn new(tree: &'a DomTree) -> Self {
        let mut stack = Vec::with_capacity(64);
        if let Some(root) = tree.root() {
            stack.push(root);
        }
        Self { tree, stack }
    }
}

impl<'a> Iterator for DomIter<'a> {
    type Item = NodeId;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.stack.pop()?;

        // 将子节点逆序压入栈（保证正序遍历）
        if let Some(node) = self.tree.get_node(id) {
            for &child in node.children.iter().rev() {
                self.stack.push(child);
            }
        }

        Some(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dom_tree_creation() {
        let mut tree = DomTree::new();

        let root_id = tree.generate_id();
        let root = DomNode::new_element(root_id, "div");
        tree.add_node(root);
        tree.set_root(root_id);

        assert_eq!(tree.root(), Some(root_id));
        assert_eq!(tree.node_count(), 1);
    }

    #[test]
    fn test_append_child() {
        let mut tree = DomTree::new();

        let root_id = tree.generate_id();
        let root = DomNode::new_element(root_id, "div");
        tree.add_node(root);
        tree.set_root(root_id);

        let child_id = tree.generate_id();
        let child = DomNode::new_element(child_id, "span");
        tree.add_node(child);

        assert!(tree.append_child(root_id, child_id));

        let root = tree.get_node(root_id).unwrap();
        assert_eq!(root.children, vec![child_id]);

        let child = tree.get_node(child_id).unwrap();
        assert_eq!(child.parent, Some(root_id));
    }

    #[test]
    fn test_remove_child() {
        let mut tree = DomTree::new();

        let root_id = tree.generate_id();
        let root = DomNode::new_element(root_id, "div");
        tree.add_node(root);
        tree.set_root(root_id);

        let child_id = tree.generate_id();
        let child = DomNode::new_element(child_id, "span");
        tree.add_node(child);

        tree.append_child(root_id, child_id);
        assert!(tree.remove_child(root_id, child_id));

        let root = tree.get_node(root_id).unwrap();
        assert!(root.children.is_empty());

        let child = tree.get_node(child_id).unwrap();
        assert_eq!(child.parent, None);
    }

    #[test]
    fn test_attributes() {
        let node = DomNode::new_element(1, "div")
            .with_attr("class", "container")
            .with_attr("id", "main");

        assert_eq!(node.get_attr("class"), Some("container"));
        assert_eq!(node.get_attr("id"), Some("main"));
        assert_eq!(node.get_attr("href"), None);
    }

    #[test]
    fn test_iter_no_recursion() {
        let mut tree = DomTree::new();

        // 创建树结构：
        //     1
        //    / \
        //   2   3
        //  / \
        // 4   5

        let root = DomNode::new_element(1, "div");
        let n2 = DomNode::new_element(2, "span");
        let n3 = DomNode::new_element(3, "p");
        let n4 = DomNode::new_text(4, "hello");
        let n5 = DomNode::new_text(5, "world");

        tree.add_node(root);
        tree.add_node(n2);
        tree.add_node(n3);
        tree.add_node(n4);
        tree.add_node(n5);
        tree.set_root(1);

        tree.append_child(1, 2);
        tree.append_child(1, 3);
        tree.append_child(2, 4);
        tree.append_child(2, 5);

        // 遍历顺序应该是 1, 2, 4, 5, 3（前序）
        let ids: Vec<_> = tree.iter().collect();
        assert_eq!(ids, vec![1, 2, 4, 5, 3]);
    }
}
