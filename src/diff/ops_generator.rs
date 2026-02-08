//! # 操作序列生成器
//!
//! 将 DOM 变更记录转换为操作序列，O(n) 复杂度。
//!
//! ## 算法说明
//!
//! - **时间复杂度**：O(n)，n 为变更记录数
//! - **空间复杂度**：O(k)，k 为实际操作数
//! - **实现方式**：单次遍历，使用迭代而非递归

use crate::dom::{DomNode, DomTree, NodeId};

/// DOM 操作类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomOp {
    /// 追加子节点
    AppendChild { parent: NodeId, child: NodeId },

    /// 移除子节点
    RemoveChild { parent: NodeId, child: NodeId },

    /// 替换子节点
    ReplaceChild {
        parent: NodeId,
        old: NodeId,
        new: NodeId,
    },

    /// 设置属性
    SetAttribute {
        node: NodeId,
        name: String,
        value: String,
    },

    /// 移除属性
    RemoveAttribute { node: NodeId, name: String },

    /// 设置文本内容
    SetTextContent { node: NodeId, text: String },

    /// 插入节点到指定位置
    InsertBefore {
        parent: NodeId,
        new: NodeId,
        reference: NodeId,
    },
}

impl DomOp {
    /// 获取操作涉及的节点 ID 列表
    #[must_use]
    pub fn node_ids(&self) -> Vec<NodeId> {
        match self {
            Self::AppendChild { parent, child } => vec![*parent, *child],
            Self::RemoveChild { parent, child } => vec![*parent, *child],
            Self::ReplaceChild { parent, old, new } => vec![*parent, *old, *new],
            Self::SetAttribute { node, .. } => vec![*node],
            Self::RemoveAttribute { node, .. } => vec![*node],
            Self::SetTextContent { node, .. } => vec![*node],
            Self::InsertBefore { parent, new, reference } => vec![*parent, *new, *reference],
        }
    }

    /// 是否影响子树结构
    #[must_use]
    pub fn is_structure_change(&self) -> bool {
        matches!(
            self,
            Self::AppendChild { .. }
                | Self::RemoveChild { .. }
                | Self::ReplaceChild { .. }
                | Self::InsertBefore { .. }
        )
    }
}

/// 变更记录（模拟 MutationRecord）
#[derive(Debug, Clone)]
pub struct MutationRecord {
    /// 变更类型
    pub type_: MutationType,
    /// 目标节点 ID
    pub target: NodeId,
    /// 相关节点 ID
    pub related_node: Option<NodeId>,
    /// 属性名（用于属性变更）
    pub attribute_name: Option<String>,
    /// 旧值
    pub old_value: Option<String>,
    /// 新值
    pub new_value: Option<String>,
    /// 前一个兄弟节点
    pub previous_sibling: Option<NodeId>,
    /// 下一个兄弟节点
    pub next_sibling: Option<NodeId>,
}

/// 变更类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MutationType {
    /// 子节点列表变更
    ChildList,
    /// 属性变更
    Attributes,
    /// 字符数据变更
    CharacterData,
    /// 子树变更
    Subtree,
}

/// 操作序列生成器
pub struct OpsGenerator<'a> {
    tree: &'a DomTree,
}

impl<'a> OpsGenerator<'a> {
    /// 创建新的生成器
    #[must_use]
    pub const fn new(tree: &'a DomTree) -> Self {
        Self { tree }
    }

    /// 生成操作序列（O(n) 复杂度）
    ///
    /// # Arguments
    ///
    /// * `mutations` - 变更记录列表
    ///
    /// # Returns
    ///
    /// 操作序列
    ///
    /// # Complexity
    ///
    /// - 时间：O(n)，n 为变更记录数
    /// - 空间：O(k)，k 为实际操作数
    pub fn generate_ops_sequence(&self, mutations: &[MutationRecord]) -> Vec<DomOp> {
        let mut ops = Vec::with_capacity(mutations.len());

        // 单次遍历所有变更记录
        for mutation in mutations {
            self.process_mutation(mutation, &mut ops);
        }

        ops
    }

    /// 处理单个变更记录（O(1)）
    fn process_mutation(&self, mutation: &MutationRecord, ops: &mut Vec<DomOp>) {
        match mutation.type_ {
            MutationType::ChildList => {
                self.process_child_list_mutation(mutation, ops);
            }
            MutationType::Attributes => {
                self.process_attribute_mutation(mutation, ops);
            }
            MutationType::CharacterData => {
                self.process_character_data_mutation(mutation, ops);
            }
            MutationType::Subtree => {
                // 子树变更在处理父节点时已经涵盖
            }
        }
    }

    /// 处理子节点列表变更（O(1)）
    fn process_child_list_mutation(&self, mutation: &MutationRecord, ops: &mut Vec<DomOp>) {
        let target = mutation.target;
        let related = match mutation.related_node {
            Some(id) => id,
            None => return,
        };

        // 检查是添加还是移除
        let target_node = match self.tree.get_node(target) {
            Some(node) => node,
            None => return,
        };

        // 检查相关节点是否在当前子节点列表中
        let is_added = target_node.children.contains(&related);

        if is_added {
            // 节点被添加
            if let Some(ref_id) = mutation.next_sibling {
                ops.push(DomOp::InsertBefore {
                    parent: target,
                    new: related,
                    reference: ref_id,
                });
            } else {
                ops.push(DomOp::AppendChild {
                    parent: target,
                    child: related,
                });
            }
        } else {
            // 节点被移除
            ops.push(DomOp::RemoveChild {
                parent: target,
                child: related,
            });
        }
    }

    /// 处理属性变更（O(1)）
    fn process_attribute_mutation(&self, mutation: &MutationRecord, ops: &mut Vec<DomOp>) {
        let target = mutation.target;
        let attr_name = match &mutation.attribute_name {
            Some(name) => name.clone(),
            None => return,
        };

        if let Some(new_value) = &mutation.new_value {
            ops.push(DomOp::SetAttribute {
                node: target,
                name: attr_name,
                value: new_value.clone(),
            });
        } else {
            ops.push(DomOp::RemoveAttribute {
                node: target,
                name: attr_name,
            });
        }
    }

    /// 处理字符数据变更（O(1)）
    fn process_character_data_mutation(&self, mutation: &MutationRecord, ops: &mut Vec<DomOp>) {
        let target = mutation.target;

        if let Some(new_value) = &mutation.new_value {
            ops.push(DomOp::SetTextContent {
                node: target,
                text: new_value.clone(),
            });
        }
    }

    /// 批量生成操作（用于测试）
    #[must_use]
    pub fn generate_batch_ops(&self, changes: &[(NodeId, BatchOp)]) -> Vec<DomOp> {
        let mut ops = Vec::with_capacity(changes.len());

        for &(node_id, ref change) in changes {
            match change {
                BatchOp::AddAttr(name, value) => {
                    ops.push(DomOp::SetAttribute {
                        node: node_id,
                        name: name.clone(),
                        value: value.clone(),
                    });
                }
                BatchOp::RemoveAttr(name) => {
                    ops.push(DomOp::RemoveAttribute {
                        node: node_id,
                        name: name.clone(),
                    });
                }
                BatchOp::SetText(text) => {
                    ops.push(DomOp::SetTextContent {
                        node: node_id,
                        text: text.clone(),
                    });
                }
                BatchOp::Append(child) => {
                    ops.push(DomOp::AppendChild {
                        parent: node_id,
                        child: *child,
                    });
                }
                BatchOp::Remove(child) => {
                    ops.push(DomOp::RemoveChild {
                        parent: node_id,
                        child: *child,
                    });
                }
            }
        }

        ops
    }
}

/// 批量操作类型（用于测试）
#[derive(Debug, Clone)]
pub enum BatchOp {
    AddAttr(String, String),
    RemoveAttr(String),
    SetText(String),
    Append(NodeId),
    Remove(NodeId),
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_tree() -> DomTree {
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
    fn test_append_child_op() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        let mutations = vec![MutationRecord {
            type_: MutationType::ChildList,
            target: 1,
            related_node: Some(2),
            attribute_name: None,
            old_value: None,
            new_value: None,
            previous_sibling: None,
            next_sibling: None,
        }];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0],
            DomOp::AppendChild {
                parent: 1,
                child: 2
            }
        );
    }

    #[test]
    fn test_set_attribute_op() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        let mutations = vec![MutationRecord {
            type_: MutationType::Attributes,
            target: 1,
            related_node: None,
            attribute_name: Some("class".to_string()),
            old_value: None,
            new_value: Some("container".to_string()),
            previous_sibling: None,
            next_sibling: None,
        }];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0],
            DomOp::SetAttribute {
                node: 1,
                name: "class".to_string(),
                value: "container".to_string()
            }
        );
    }

    #[test]
    fn test_set_text_content_op() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        let mutations = vec![MutationRecord {
            type_: MutationType::CharacterData,
            target: 3,
            related_node: None,
            attribute_name: None,
            old_value: Some("hello".to_string()),
            new_value: Some("world".to_string()),
            previous_sibling: None,
            next_sibling: None,
        }];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0],
            DomOp::SetTextContent {
                node: 3,
                text: "world".to_string()
            }
        );
    }

    #[test]
    fn test_insert_before_op() {
        let mut tree = DomTree::new();

        let root = DomNode::new_element(1, "div");
        let child1 = DomNode::new_element(2, "span");
        let child2 = DomNode::new_text(3, "hello");
        let child3 = DomNode::new_text(4, "world");

        tree.add_node(root);
        tree.add_node(child1);
        tree.add_node(child2);
        tree.add_node(child3);
        tree.set_root(1);

        tree.append_child(1, 2);
        tree.append_child(1, 3);

        let generator = OpsGenerator::new(&tree);

        // 在 3 之前插入 4
        let mutations = vec![MutationRecord {
            type_: MutationType::ChildList,
            target: 1,
            related_node: Some(4),
            attribute_name: None,
            old_value: None,
            new_value: None,
            previous_sibling: Some(2),
            next_sibling: Some(3),
        }];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0],
            DomOp::InsertBefore {
                parent: 1,
                new: 4,
                reference: 3
            }
        );
    }

    #[test]
    fn test_remove_child_op() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        // 节点 2 不在子节点列表中（已被移除）
        let mutations = vec![MutationRecord {
            type_: MutationType::ChildList,
            target: 1,
            related_node: Some(2),
            attribute_name: None,
            old_value: None,
            new_value: None,
            previous_sibling: None,
            next_sibling: None,
        }];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0],
            DomOp::RemoveChild {
                parent: 1,
                child: 2
            }
        );
    }

    #[test]
    fn test_remove_attribute_op() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        let mutations = vec![MutationRecord {
            type_: MutationType::Attributes,
            target: 1,
            related_node: None,
            attribute_name: Some("class".to_string()),
            old_value: Some("container".to_string()),
            new_value: None,
            previous_sibling: None,
            next_sibling: None,
        }];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 1);
        assert_eq!(
            ops[0],
            DomOp::RemoveAttribute {
                node: 1,
                name: "class".to_string()
            }
        );
    }

    #[test]
    fn test_multiple_mutations() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        let mutations = vec![
            MutationRecord {
                type_: MutationType::Attributes,
                target: 1,
                related_node: None,
                attribute_name: Some("id".to_string()),
                old_value: None,
                new_value: Some("main".to_string()),
                previous_sibling: None,
                next_sibling: None,
            },
            MutationRecord {
                type_: MutationType::CharacterData,
                target: 3,
                related_node: None,
                attribute_name: None,
                old_value: Some("hello".to_string()),
                new_value: Some("world".to_string()),
                previous_sibling: None,
                next_sibling: None,
            },
        ];

        let ops = generator.generate_ops_sequence(&mutations);

        assert_eq!(ops.len(), 2);
        assert_eq!(
            ops[0],
            DomOp::SetAttribute {
                node: 1,
                name: "id".to_string(),
                value: "main".to_string()
            }
        );
        assert_eq!(
            ops[1],
            DomOp::SetTextContent {
                node: 3,
                text: "world".to_string()
            }
        );
    }

    #[test]
    fn test_batch_ops() {
        let tree = create_test_tree();
        let generator = OpsGenerator::new(&tree);

        let changes = vec![
            (1, BatchOp::AddAttr("class".to_string(), "container".to_string())),
            (3, BatchOp::SetText("world".to_string())),
        ];

        let ops = generator.generate_batch_ops(&changes);

        assert_eq!(ops.len(), 2);
    }

    #[test]
    fn test_dom_op_node_ids() {
        let op = DomOp::AppendChild {
            parent: 1,
            child: 2,
        };

        assert_eq!(op.node_ids(), vec![1, 2]);
        assert!(op.is_structure_change());

        let op = DomOp::SetAttribute {
            node: 1,
            name: "class".to_string(),
            value: "container".to_string(),
        };

        assert_eq!(op.node_ids(), vec![1]);
        assert!(!op.is_structure_change());
    }
}
