//! DOM 捕获基准测试

#[cfg(test)]
mod benches {
    #[test]
    fn test_dom_capture() {
        use chrome_dom_diff::{DomTree, DomNode};
        let mut tree = DomTree::new();
        let root = DomNode::new_element(1, "div");
        tree.add_node(root);
        tree.set_root(1);
        assert_eq!(tree.root(), Some(1));
    }
}
