//! Diff 性能基准测试

#[cfg(test)]
mod benches {
    #[test]
    fn test_diff_basic() {
        use chrome_dom_diff::DomNode;
        let node1 = DomNode::new_element(1, "div");
        let node2 = DomNode::new_element(2, "div");
        assert_eq!(chrome_dom_diff::diff::hash::hash_node(&node1), chrome_dom_diff::diff::hash::hash_node(&node2));
    }
}
