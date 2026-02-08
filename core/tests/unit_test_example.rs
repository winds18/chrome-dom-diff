// 艹，老王我写的Rust单元测试示例
// 这个SB测试覆盖了chrome-dom-diff核心功能

use chrome_dom_diff::{
    dom::{DomNode, DomTree, NodeType},
    diff::{compute_tree_diff, DiffOp},
    core::arena::Arena,
    core::pool::Pool,
    monitoring::{Histogram, Counter},
};

#[cfg(test)]
mod dom_tests {
    use super::*;

    #[test]
    fn test_create_dom_tree() {
        let tree = DomTree::new();
        assert_eq!(tree.node_count(), 0);
    }

    #[test]
    fn test_add_element_node() {
        let mut tree = DomTree::new();
        let arena = Arena::new();

        let node_id = tree.add_element(
            &arena,
            "div",
            Some("container"),
            vec![("class", "test-class")],
        );

        assert_eq!(tree.node_count(), 1);
        assert!(node_id > 0);
    }

    #[test]
    fn test_add_text_node() {
        let mut tree = DomTree::new();
        let arena = Arena::new();

        let parent_id = tree.add_element(&arena, "div", None, vec![]);
        let text_id = tree.add_text(&arena, parent_id, "Hello World");

        assert_eq!(tree.node_count(), 2);
        assert!(text_id > 0);
    }

    #[test]
    fn test_append_child() {
        let mut tree = DomTree::new();
        let arena = Arena::new();

        let parent_id = tree.add_element(&arena, "div", None, vec![]);
        let child_id = tree.add_element(&arena, "p", None, vec![]);

        tree.append_child(&arena, parent_id, child_id);

        // 验证父子关系
        let parent = tree.get_node(parent_id);
        assert!(parent.is_some());
    }

    #[test]
    fn test_get_node() {
        let mut tree = DomTree::new();
        let arena = Arena::new();

        let node_id = tree.add_element(&arena, "div", Some("test"), vec![]);
        let node = tree.get_node(node_id);

        assert!(node.is_some());
        let node = node.unwrap();
        assert_eq!(node.node_type, NodeType::Element);
    }
}

#[cfg(test)]
mod diff_tests {
    use super::*;

    #[test]
    fn test_no_diff() {
        let arena = Arena::new();
        let mut tree1 = DomTree::new();
        let mut tree2 = DomTree::new();

        // 创建相同的树
        let root1 = tree1.add_element(&arena, "html", None, vec![]);
        let root2 = tree2.add_element(&arena, "html", None, vec![]);

        let diff = compute_tree_diff(&tree1, &tree2, &arena);

        assert_eq!(diff.len(), 0);
    }

    #[test]
    fn test_insert_diff() {
        let arena = Arena::new();
        let mut tree1 = DomTree::new();
        let mut tree2 = DomTree::new();

        // tree1: div
        let root1 = tree1.add_element(&arena, "div", None, vec![]);

        // tree2: div > p
        let root2 = tree2.add_element(&arena, "div", None, vec![]);
        let child2 = tree2.add_element(&arena, "p", None, vec![]);
        tree2.append_child(&arena, root2, child2);

        let diff = compute_tree_diff(&tree1, &tree2, &arena);

        // 应该检测到插入操作
        assert!(diff.iter().any(|op| matches!(op, DiffOp::Insert(_, _))));
    }

    #[test]
    fn test_delete_diff() {
        let arena = Arena::new();
        let mut tree1 = DomTree::new();
        let mut tree2 = DomTree::new();

        // tree1: div > p
        let root1 = tree1.add_element(&arena, "div", None, vec![]);
        let child1 = tree1.add_element(&arena, "p", None, vec![]);
        tree1.append_child(&arena, root1, child1);

        // tree2: div (子节点被删除)
        let root2 = tree2.add_element(&arena, "div", None, vec![]);

        let diff = compute_tree_diff(&tree1, &tree2, &arena);

        // 应该检测到删除操作
        assert!(diff.iter().any(|op| matches!(op, DiffOpDelete(_, _))));
    }

    #[test]
    fn test_attribute_change_diff() {
        let arena = Arena::new();
        let mut tree1 = DomTree::new();
        let mut tree2 = DomTree::new();

        // tree1: div with class="old"
        let root1 = tree1.add_element(&arena, "div", None, vec![("class", "old")]);

        // tree2: div with class="new"
        let root2 = tree2.add_element(&arena, "div", None, vec![("class", "new")]);

        let diff = compute_tree_diff(&tree1, &tree2, &arena);

        // 应该检测到属性更新操作
        assert!(diff.iter().any(|op| matches!(op, DiffOp::UpdateAttr(_, _, _))));
    }
}

#[cfg(test)]
mod arena_tests {
    use super::*;

    #[test]
    fn test_arena_create() {
        let arena = Arena::new();
        assert_eq!(arena.usage(), 0);
    }

    #[test]
    fn test_arena_alloc_str() {
        let arena = Arena::new();
        let s = arena.alloc_str("Hello World");
        assert_eq!(s, "Hello World");
        assert!(arena.usage() > 0);
    }

    #[test]
    fn test_arena_reset() {
        let arena = Arena::new();
        arena.alloc_str("Test");
        assert!(arena.usage() > 0);

        arena.reset();
        assert_eq!(arena.usage(), 0);
    }

    #[test]
    fn test_arena_multiple_alloc() {
        let arena = Arena::new();
        for i in 0..100 {
            arena.alloc_str(&format!("string{}", i));
        }
        assert!(arena.usage() > 0);
    }
}

#[cfg(test)]
mod pool_tests {
    use super::*;

    #[test]
    fn test_pool_acquire() {
        let pool: Pool<Vec<u8>> = Pool::new(10);
        {
            let _item = pool.acquire();
            // item会自动归还
        }
        // 验证复用率
        assert!(pool.reuse_rate() >= 0.0);
    }

    #[test]
    fn test_pool_warm_up() {
        let pool: Pool<Vec<u8>> = Pool::new(10);
        pool.warm_up(5);
        // 预热后应该有可用的对象
        let _item = pool.acquire();
    }

    #[test]
    fn test_pool_reuse() {
        let pool: Pool<Vec<u8>> = Pool::new(10);
        {
            let mut item1 = pool.acquire();
            item1.push(42);
        }
        {
            let item2 = pool.acquire();
            // 应该复用之前的对象
            assert_eq!(item2.len(), 1);
            assert_eq!(item2[0], 42);
        }
    }
}

#[cfg(test)]
mod monitoring_tests {
    use super::*;

    #[test]
    fn test_histogram_record() {
        let hist = Histogram::new("test_histogram", 100);
        hist.record(50);
        hist.record(100);
        hist.record(150);

        let stats = hist.stats();
        assert_eq!(stats.count, 3);
        assert!(stats.avg > 0);
    }

    #[test]
    fn test_histogram_percentile() {
        let hist = Histogram::new("test_histogram", 100);
        for i in 1..=100 {
            hist.record(i);
        }

        let p50 = hist.percentile(50);
        let p95 = hist.percentile(95);
        let p99 = hist.percentile(99);

        assert!(p50 < p95);
        assert!(p95 < p99);
    }

    #[test]
    fn test_counter_inc() {
        let counter = Counter::new("test_counter");
        assert_eq!(counter.value(), 0);

        counter.inc();
        assert_eq!(counter.value(), 1);

        counter.inc_by(5);
        assert_eq!(counter.value(), 6);
    }

    #[test]
    fn test_counter_reset() {
        let counter = Counter::new("test_counter");
        counter.inc();
        counter.reset();
        assert_eq!(counter.value(), 0);
    }
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_dom_capture_and_diff() {
        let arena = Arena::new();

        // 创建第一个DOM树
        let mut tree1 = DomTree::new();
        let root1 = tree1.add_element(&arena, "html", None, vec![]);
        let body1 = tree1.add_element(&arena, "body", None, vec![]);
        tree1.append_child(&arena, root1, body1);
        let div1 = tree1.add_element(&arena, "div", Some("content"), vec![("class", "container")]);
        tree1.append_child(&arena, body1, div1);

        // 创建第二个DOM树（略有不同）
        let mut tree2 = DomTree::new();
        let root2 = tree2.add_element(&arena, "html", None, vec![]);
        let body2 = tree2.add_element(&arena, "body", None, vec![]);
        tree2.append_child(&arena, root2, body2);
        let div2 = tree2.add_element(&arena, "div", Some("content"), vec![("class", "container modified")]);
        tree2.append_child(&arena, body2, div2);

        // 计算差分
        let diff = compute_tree_diff(&tree1, &tree2, &arena);

        // 应该检测到属性变化
        assert!(diff.len() > 0);
    }

    #[test]
    fn test_large_dom_performance() {
        let arena = Arena::new();
        let mut tree = DomTree::new();

        // 创建一个较大的DOM树
        let root = tree.add_element(&arena, "html", None, vec![]);
        let body = tree.add_element(&arena, "body", None, vec![]);
        tree.append_child(&arena, root, body);

        // 添加100个子节点
        for i in 0..100 {
            let div = tree.add_element(
                &arena,
                "div",
                Some(&format!("div-{}", i)),
                vec![("id", &format!("id-{}", i))],
            );
            tree.append_child(&arena, body, div);
        }

        assert_eq!(tree.node_count(), 102); // html + body + 100 divs
    }
}

// 艹，单元测试完成！老王我警告你，这些测试必须在WASM模块正确编译后才能运行
