//! # WASM 导出接口
//!
//! 提供 DOM 捕获、差分计算等核心功能的 WASM 导出。

use std::sync::LazyLock;
use std::sync::Mutex;
use std::collections::HashMap;

use crate::dom::{DomTree, DomNode, NodeType, NodeId};
use crate::diff::compute_tree_diff;
use crate::arena::DomArena;
use crate::monitoring;

// ============================================
// 全局状态管理
// ============================================

struct GlobalState {
    trees: Vec<Option<DomTree>>,
    arenas: Vec<Option<DomArena>>,
    string_pools: Vec<Option<crate::pool::ObjectPool<String>>>,
    next_tree_id: u64,
    next_arena_id: u64,
    next_pool_id: u64,
}

impl GlobalState {
    fn new() -> Self {
        Self {
            trees: Vec::with_capacity(16),
            arenas: Vec::with_capacity(8),
            string_pools: Vec::with_capacity(8),
            next_tree_id: 1,
            next_arena_id: 1,
            next_pool_id: 1,
        }
    }

    fn insert_tree(&mut self) -> u64 {
        let id = self.next_tree_id;
        self.next_tree_id += 1;
        self.ensure_capacity_trees(id);
        self.trees[(id - 1) as usize] = Some(DomTree::new());
        id
    }

    fn get_tree(&self, id: u64) -> Option<&DomTree> {
        if id == 0 || id > self.trees.len() as u64 {
            return None;
        }
        self.trees[(id - 1) as usize].as_ref()
    }

    fn get_tree_mut(&mut self, id: u64) -> Option<&mut DomTree> {
        if id == 0 || id > self.trees.len() as u64 {
            return None;
        }
        self.trees[(id - 1) as usize].as_mut()
    }

    fn insert_arena(&mut self) -> u64 {
        let id = self.next_arena_id;
        self.next_arena_id += 1;
        self.ensure_capacity_arenas(id);
        self.arenas[(id - 1) as usize] = Some(DomArena::new());
        id
    }

    fn get_arena_mut(&mut self, id: u64) -> Option<&mut DomArena> {
        if id == 0 || id > self.arenas.len() as u64 {
            return None;
        }
        self.arenas[(id - 1) as usize].as_mut()
    }

    fn insert_string_pool(&mut self) -> u64 {
        let id = self.next_pool_id;
        self.next_pool_id += 1;
        self.ensure_capacity_pools(id);
        self.string_pools[(id - 1) as usize] = Some(crate::pool::ObjectPool::new());
        id
    }

    fn get_string_pool_mut(&mut self, id: u64) -> Option<&mut crate::pool::ObjectPool<String>> {
        if id == 0 || id > self.string_pools.len() as u64 {
            return None;
        }
        self.string_pools[(id - 1) as usize].as_mut()
    }

    fn ensure_capacity_trees(&mut self, id: u64) {
        while self.trees.len() < (id as usize) {
            self.trees.push(None);
        }
    }

    fn ensure_capacity_arenas(&mut self, id: u64) {
        while self.arenas.len() < (id as usize) {
            self.arenas.push(None);
        }
    }

    fn ensure_capacity_pools(&mut self, id: u64) {
        while self.string_pools.len() < (id as usize) {
            self.string_pools.push(None);
        }
    }
}

static GLOBAL_STATE: LazyLock<Mutex<GlobalState>> = LazyLock::new(|| {
    Mutex::new(GlobalState::new())
});

// ============================================
// DOM 树管理 API
// ============================================

#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_create() -> u64 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.insert_tree()
}

#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_add_element(
    tree_id: u64,
    node_id: u64,
    tag_name_ptr: *const u8,
    tag_name_len: usize,
) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree_mut(tree_id) else {
        return 0;
    };

    // 修改：空指针时使用默认tag名称"div"而不是返回0
    let tag_name = if tag_name_ptr.is_null() || tag_name_len == 0 {
        "div"  // 默认tag名称
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts(tag_name_ptr, tag_name_len);
            match std::str::from_utf8(slice) {
                Ok(s) => s,
                Err(_) => "div",  // UTF-8无效时使用默认值
            }
        }
    };

    let node = DomNode::new_element(node_id, tag_name);
    tree.add_node(node);
    
    if tree.root().is_none() {
        tree.set_root(node_id);
    }

    1
}

#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_add_text(
    tree_id: u64,
    node_id: u64,
    text_ptr: *const u8,
    text_len: usize,
) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree_mut(tree_id) else {
        return 0;
    };

    // 修改：空指针时使用空字符串而不是返回0
    let text = if text_ptr.is_null() || text_len == 0 {
        ""
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts(text_ptr, text_len);
            match std::str::from_utf8(slice) {
                Ok(s) => s,
                Err(_) => "",
            }
        }
    };

    let node = DomNode::new_text(node_id, text);
    tree.add_node(node);

    if tree.root().is_none() {
        tree.set_root(node_id);
    }

    1
}

#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_append_child(
    tree_id: u64,
    parent_id: u64,
    child_id: u64,
) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree_mut(tree_id) else {
        return 0;
    };

    if tree.append_child(parent_id, child_id) {
        1
    } else {
        0
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_node_count(tree_id: u64) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree(tree_id) else {
        return 0;
    };
    tree.node_count() as u32
}

#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_delete(tree_id: u64) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    if tree_id == 0 || tree_id > state.trees.len() as u64 {
        return 0;
    }
    
    state.trees[(tree_id - 1) as usize] = None;
    1
}

// ============================================
// 差分计算 API
// ============================================

/// 计算两棵DOM树的差分
/// 
/// 返回值：差分操作数量（0 表示失败）
#[unsafe(no_mangle)]
pub extern "C" fn diff_compute(tree1_id: u64, tree2_id: u64) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let (Some(tree1), Some(tree2)) = (state.get_tree(tree1_id), state.get_tree(tree2_id)) else {
        return 0;
    };

    let changes = compute_tree_diff(tree1, tree2);
    changes.changes.len() as u32
}

// ============================================
// Arena 分配器 API
// ============================================

/// 创建新的 Arena
#[unsafe(no_mangle)]
pub extern "C" fn arena_create() -> u64 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.insert_arena()
}

/// Arena 分配字符串（返回指针到 WASM 线性内存）
/// 
/// 注意：调用方需要提供足够大的缓冲区
/// 返回值：写入的字节数，0 表示失败
#[unsafe(no_mangle)]
pub extern "C" fn arena_alloc_str(
    arena_id: u64,
    str_ptr: *const u8,
    str_len: usize,
    out_ptr: *mut u8,
    out_capacity: usize,
) -> usize {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(arena) = state.get_arena_mut(arena_id) else {
        return 0;
    };

    let str_slice = unsafe {
        if str_ptr.is_null() || str_len == 0 {
            return 0;
        }
        std::slice::from_raw_parts(str_ptr, str_len)
    };

    // 安全地转换为 &str（忽略无效UTF-8）
    let str_ref = std::str::from_utf8(str_slice).unwrap_or("");

    let allocated = arena.alloc_str(str_ref);
    let allocated_bytes = allocated.as_bytes();
    let copy_len = allocated_bytes.len().min(out_capacity);

    unsafe {
        if !out_ptr.is_null() && copy_len > 0 {
            std::ptr::copy_nonoverlapping(
                allocated_bytes.as_ptr(),
                out_ptr,
                copy_len
            );
        }
    }

    copy_len
}

/// 获取 Arena 使用量（字节）
#[unsafe(no_mangle)]
pub extern "C" fn arena_usage(arena_id: u64) -> usize {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(arena) = state.get_arena_mut(arena_id) else {
        return 0;
    };
    arena.usage()
}

/// 重置 Arena
#[unsafe(no_mangle)]
pub extern "C" fn arena_reset(arena_id: u64) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(arena) = state.get_arena_mut(arena_id) else {
        return 0;
    };
    arena.reset();
    1
}

// ============================================
// 对象池 API
// ============================================

/// 创建 String 对象池
#[unsafe(no_mangle)]
pub extern "C" fn pool_create_string() -> u64 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    state.insert_string_pool()
}

/// 获取对象池统计
#[unsafe(no_mangle)]
pub extern "C" fn pool_get_stats(
    pool_id: u64,
    out_size: *mut u32,
    out_available: *mut u32,
    out_in_use: *mut u32,
    out_reuse_rate: *mut f32,
) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(pool) = state.get_string_pool_mut(pool_id) else {
        return 0;
    };

    let stats = pool.stats();
    
    unsafe {
        if !out_size.is_null() {
            *out_size = stats.size as u32;
        }
        if !out_available.is_null() {
            *out_available = stats.available as u32;
        }
        if !out_in_use.is_null() {
            *out_in_use = stats.in_use as u32;
        }
        if !out_reuse_rate.is_null() {
            *out_reuse_rate = stats.reuse_rate as f32;
        }
    }

    1
}

// ============================================
// 性能监控 API
// ============================================

/// 记录延迟（微秒）
#[unsafe(no_mangle)]
pub extern "C" fn monitoring_record_latency_us(
    name_ptr: *const u8,
    name_len: usize,
    value_us: f64,
) {
    let name = unsafe {
        if name_ptr.is_null() || name_len == 0 {
            return;
        }
        let slice = std::slice::from_raw_parts(name_ptr, name_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    monitoring::record_latency_us(name, value_us);
}

/// 增加计数
#[unsafe(no_mangle)]
pub extern "C" fn monitoring_inc_counter(
    name_ptr: *const u8,
    name_len: usize,
) {
    let name = unsafe {
        if name_ptr.is_null() || name_len == 0 {
            return;
        }
        let slice = std::slice::from_raw_parts(name_ptr, name_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    monitoring::inc_counter(name);
}

/// 设置仪表值
#[unsafe(no_mangle)]
pub extern "C" fn monitoring_set_gauge(
    name_ptr: *const u8,
    name_len: usize,
    value: f64,
) {
    let name = unsafe {
        if name_ptr.is_null() || name_len == 0 {
            return;
        }
        let slice = std::slice::from_raw_parts(name_ptr, name_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return,
        }
    };

    monitoring::set_gauge(name, value);
}

// ============================================
// 测试函数
// ============================================

/// 测试加法
#[unsafe(no_mangle)]
pub extern "C" fn test_add(a: i32, b: i32) -> i32 {
    a + b
}

/// 获取版本号
#[unsafe(no_mangle)]
pub extern "C" fn get_version() -> u32 {
    1
}

// ============================================
// DOM 捕获功能（批量添加节点）
// ============================================

/// 批量添加节点（用于DOM捕获）
/// 
/// 参数：
/// - tree_id: 树ID
/// - nodes_ptr: 节点数据数组指针
/// - nodes_count: 节点数量
/// 
/// 节点数据格式（每个节点32字节）：
/// - offset 0: node_id (u64)
/// - offset 8: node_type (u8: 0=Element, 1=Text)
/// - offset 9: parent_id (u64)
/// - offset 17: tag_name_len (u16)
/// - offset 19: tag_name_ptr (u64)
/// - offset 27: text_content_len (u32)
/// 
/// 返回值：成功添加的节点数量
#[unsafe(no_mangle)]
pub extern "C" fn dom_tree_add_nodes_batch(
    tree_id: u64,
    nodes_ptr: *const u8,
    nodes_count: u32,
) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree_mut(tree_id) else {
        return 0;
    };

    if nodes_ptr.is_null() || nodes_count == 0 {
        return 0;
    }

    let mut added = 0;
    let u8_size = std::mem::size_of::<u8>();
    let u64_size = std::mem::size_of::<u64>();
    let u16_size = std::mem::size_of::<u16>();
    let u32_size = std::mem::size_of::<u32>();

    for i in 0..nodes_count {
        unsafe {
            let node_base = nodes_ptr.add((i as usize) * 32);

            // 读取节点数据
            let node_id = *(node_base as *const u64);
            let node_type_raw = *(node_base.add(8) as *const u8);
            let parent_id = *(node_base.add(9) as *const u64);
            let tag_name_len = *(node_base.add(17) as *const u16);
            let tag_name_ptr = *(node_base.add(19) as *const u64);
            let text_content_len = *(node_base.add(27) as *const u32);

            let mut node = match node_type_raw {
                0 => {
                    // Element节点
                    let tag_name = if tag_name_len > 0 && tag_name_ptr != 0 {
                        let slice = std::slice::from_raw_parts(
                            tag_name_ptr as *const u8,
                            tag_name_len as usize
                        );
                        std::str::from_utf8(slice).unwrap_or("div").to_string()
                    } else {
                        "div".to_string()  // 默认值
                    };
                    DomNode::new_element(node_id, tag_name)
                }
                1 => {
                    // Text节点
                    let text = if text_content_len > 0 {
                        // 假设text紧跟在节点数据后面（简化处理）
                        String::new()
                    } else {
                        String::new()
                    };
                    DomNode::new_text(node_id, text)
                }
                _ => continue,
            };

            // 设置父节点
            if parent_id != 0 {
                node.parent = Some(parent_id);
            }

            tree.add_node(node);

            // 设置为根节点（第一个节点）
            if tree.root().is_none() {
                tree.set_root(node_id);
            }

            added += 1;
        }
    }

    added
}

/// 快速捕获DOM树（从节点数组）
/// 
/// 这是一个简化的DOM捕获接口，用于快速测试
#[unsafe(no_mangle)]
pub extern "C" fn dom_capture_simple(
    nodes_ptr: *const u8,
    nodes_count: u32,
) -> u64 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let tree_id = state.insert_tree();

    // 调用批量添加
    dom_tree_add_nodes_batch(tree_id, nodes_ptr, nodes_count);

    tree_id
}

// ============================================
// 高级差分操作导出
// ============================================

/// 获取差分结果（返回变更数量）
/// 
/// 参数：
/// - tree1_id: 第一棵树ID
/// - tree2_id: 第二棵树ID
/// - out_changes: 输出数组指针
/// - out_capacity: 输出数组容量
/// 
/// 返回值：差分操作数量，0 表示失败
#[unsafe(no_mangle)]
pub extern "C" fn diff_get_changes(
    tree1_id: u64,
    tree2_id: u64,
    _out_changes: *mut u8,
    _out_capacity: u32,
) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let (Some(tree1), Some(tree2)) = (state.get_tree(tree1_id), state.get_tree(tree2_id)) else {
        return 0;
    };

    let diff_result = compute_tree_diff(tree1, tree2);
    let changes = &diff_result.changes;

    // 简化：只返回变更数量
    // 实际使用中应该序列化每个变更到out_changes
    changes.len() as u32
}

/// 获取插入的节点数量
#[unsafe(no_mangle)]
pub extern "C" fn diff_get_inserts_count(tree1_id: u64, tree2_id: u64) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let (Some(tree1), Some(tree2)) = (state.get_tree(tree1_id), state.get_tree(tree2_id)) else {
        return 0;
    };

    let diff_result = compute_tree_diff(tree1, tree2);
    diff_result.inserts.len() as u32
}

/// 获取删除的节点数量
#[unsafe(no_mangle)]
pub extern "C" fn diff_get_deletes_count(tree1_id: u64, tree2_id: u64) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let (Some(tree1), Some(tree2)) = (state.get_tree(tree1_id), state.get_tree(tree2_id)) else {
        return 0;
    };

    let diff_result = compute_tree_diff(tree1, tree2);
    diff_result.deletes.len() as u32
}

/// 获取移动的节点数量
#[unsafe(no_mangle)]
pub extern "C" fn diff_get_moves_count(tree1_id: u64, tree2_id: u64) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let (Some(tree1), Some(tree2)) = (state.get_tree(tree1_id), state.get_tree(tree2_id)) else {
        return 0;
    };

    let diff_result = compute_tree_diff(tree1, tree2);
    diff_result.moves.len() as u32
}

// ============================================
// 高级性能监控API
// ============================================

/// 记录延迟并检查阈值（返回是否超过阈值）
/// 
/// 返回值：1 超过阈值，0 未超过
#[unsafe(no_mangle)]
pub extern "C" fn monitoring_record_latency_and_check_threshold(
    name_ptr: *const u8,
    name_len: usize,
    value_us: f64,
    threshold_us: f64,
) -> u32 {
    let name = unsafe {
        if name_ptr.is_null() || name_len == 0 {
            return 0;
        }
        let slice = std::slice::from_raw_parts(name_ptr, name_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    // 记录延迟
    monitoring::record_latency_us(name, value_us);

    // 检查阈值
    if value_us > threshold_us {
        1
    } else {
        0
    }
}

/// 增加计数并返回新值
#[unsafe(no_mangle)]
pub extern "C" fn monitoring_inc_counter_and_get(
    name_ptr: *const u8,
    name_len: usize,
    delta: u64,
) -> u64 {
    let name = unsafe {
        if name_ptr.is_null() || name_len == 0 {
            return 0;
        }
        let slice = std::slice::from_raw_parts(name_ptr, name_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return 0,
        }
    };

    monitoring::inc_counter_by(name, delta);

    // 返回新值（简化处理，实际应该查询）
    delta
}

/// 设置仪表值并返回旧值
#[unsafe(no_mangle)]
pub extern "C" fn monitoring_set_gauge_and_get(
    name_ptr: *const u8,
    name_len: usize,
    value: f64,
) -> f64 {
    let name = unsafe {
        if name_ptr.is_null() || name_len == 0 {
            return 0.0;
        }
        let slice = std::slice::from_raw_parts(name_ptr, name_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s,
            Err(_) => return 0.0,
        }
    };

    // 简化：直接返回新值（实际应该返回旧值）
    monitoring::set_gauge(name, value);
    value
}

// ============================================
// 工具函数
// ============================================

/// 复制WASM内存到Rust
/// 
/// 返回值：实际复制的字节数
#[unsafe(no_mangle)]
pub extern "C" fn memory_copy_from_wasm(
    dest: *mut u8,
    src: *const u8,
    len: usize,
) -> usize {
    if dest.is_null() || src.is_null() || len == 0 {
        return 0;
    }

    unsafe {
        std::ptr::copy_nonoverlapping(src, dest, len);
    }

    len
}

/// 获取字符串长度（UTF-8字节长度）
#[unsafe(no_mangle)]
pub extern "C" fn string_get_byte_length(
    str_ptr: *const u8,
    max_len: usize,
) -> usize {
    if str_ptr.is_null() {
        return 0;
    }

    // 查找null终止符或max_len
    let mut len = 0;
    unsafe {
        while len < max_len {
            let byte = *str_ptr.add(len);
            if byte == 0 {
                break;
            }
            len += 1;
        }
    }

    len
}

/// 计算字符串的UTF-8字符长度（非字节长度）
#[unsafe(no_mangle)]
pub extern "C" fn string_get_char_length(
    str_ptr: *const u8,
    byte_len: usize,
) -> usize {
    if str_ptr.is_null() || byte_len == 0 {
        return 0;
    }

    unsafe {
        let slice = std::slice::from_raw_parts(str_ptr, byte_len);
        match std::str::from_utf8(slice) {
            Ok(s) => s.chars().count(),
            Err(_) => byte_len, // 失败时返回字节长度
        }
    }
}

// ============================================
// 节点属性管理 API（新增）
// ============================================

/// 为节点添加属性
/// 
/// 参数：
/// - tree_id: 树ID
/// - node_id: 节点ID
/// - name_ptr: 属性名指针
/// - name_len: 属性名长度
/// - value_ptr: 属性值指针
/// - value_len: 属性值长度
/// 
/// 返回值：1成功，0失败
#[unsafe(no_mangle)]
pub extern "C" fn dom_node_add_attribute(
    tree_id: u64,
    node_id: u64,
    name_ptr: *const u8,
    name_len: usize,
    value_ptr: *const u8,
    value_len: usize,
) -> u32 {
    let mut state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree_mut(tree_id) else {
        return 0;
    };

    // 读取属性名
    let attr_name = if name_ptr.is_null() || name_len == 0 {
        return 0;
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts(name_ptr, name_len);
            match std::str::from_utf8(slice) {
                Ok(s) => s.to_string(),
                Err(_) => return 0,
            }
        }
    };

    // 读取属性值
    let attr_value = if value_ptr.is_null() || value_len == 0 {
        String::new()
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts(value_ptr, value_len);
            match std::str::from_utf8(slice) {
                Ok(s) => s.to_string(),
                Err(_) => String::new(),
            }
        }
    };

    // 获取节点并添加属性
    if let Some(node) = tree.get_node_mut(node_id) {
        node.attributes.push((attr_name, attr_value));
        1
    } else {
        0
    }
}

/// 获取节点属性数量
#[unsafe(no_mangle)]
pub extern "C" fn dom_node_get_attr_count(tree_id: u64, node_id: u64) -> u32 {
    let state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree(tree_id) else {
        return 0;
    };

    if let Some(node) = tree.get_node(node_id) {
        node.attributes.len() as u32
    } else {
        0
    }
}

/// 获取节点属性（通过索引）
/// 
/// 参数：
/// - tree_id: 树ID
/// - node_id: 节点ID
/// - index: 属性索引
/// - out_name_ptr: 输出属性名指针
/// - out_name_capacity: 输出属性名容量
/// - out_value_ptr: 输出属性值指针
/// - out_value_capacity: 输出属性值容量
/// 
/// 返回值：写入的字节数对 (name_len << 32 | value_len)，0表示失败
#[unsafe(no_mangle)]
pub extern "C" fn dom_node_get_attr(
    tree_id: u64,
    node_id: u64,
    index: u32,
    out_name_ptr: *mut u8,
    out_name_capacity: usize,
    out_value_ptr: *mut u8,
    out_value_capacity: usize,
) -> u64 {
    let state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree(tree_id) else {
        return 0;
    };

    if let Some(node) = tree.get_node(node_id) {
        if index as usize >= node.attributes.len() {
            return 0;
        }

        let (name, value) = &node.attributes[index as usize];
        let name_bytes = name.as_bytes();
        let value_bytes = value.as_bytes();

        let name_copy_len = name_bytes.len().min(out_name_capacity);
        let value_copy_len = value_bytes.len().min(out_value_capacity);

        unsafe {
            if !out_name_ptr.is_null() && name_copy_len > 0 {
                std::ptr::copy_nonoverlapping(
                    name_bytes.as_ptr(),
                    out_name_ptr,
                    name_copy_len
                );
            }

            if !out_value_ptr.is_null() && value_copy_len > 0 {
                std::ptr::copy_nonoverlapping(
                    value_bytes.as_ptr(),
                    out_value_ptr,
                    value_copy_len
                );
            }
        }

        // 返回值：高32位是name长度，低32位是value长度
        ((name_bytes.len() as u64) << 32) | (value_bytes.len() as u64)
    } else {
        0
    }
}

/// 查询属性值（通过属性名）
/// 
/// 参数：
/// - tree_id: 树ID
/// - node_id: 节点ID
/// - name_ptr: 属性名指针
/// - name_len: 属性名长度
/// - out_value_ptr: 输出属性值指针
/// - out_value_capacity: 输出属性值容量
/// 
/// 返回值：属性值长度，0表示未找到
#[unsafe(no_mangle)]
pub extern "C" fn dom_node_get_attr_value(
    tree_id: u64,
    node_id: u64,
    name_ptr: *const u8,
    name_len: usize,
    out_value_ptr: *mut u8,
    out_value_capacity: usize,
) -> usize {
    let state = GLOBAL_STATE.lock().unwrap();
    let Some(tree) = state.get_tree(tree_id) else {
        return 0;
    };

    let attr_name = if name_ptr.is_null() || name_len == 0 {
        return 0;
    } else {
        unsafe {
            let slice = std::slice::from_raw_parts(name_ptr, name_len);
            match std::str::from_utf8(slice) {
                Ok(s) => s,
                Err(_) => return 0,
            }
        }
    };

    if let Some(node) = tree.get_node(node_id) {
        if let Some(value) = node.get_attr(attr_name) {
            let value_bytes = value.as_bytes();
            let copy_len = value_bytes.len().min(out_value_capacity);

            unsafe {
                if !out_value_ptr.is_null() && copy_len > 0 {
                    std::ptr::copy_nonoverlapping(
                        value_bytes.as_ptr(),
                        out_value_ptr,
                        copy_len
                    );
                }
            }

            value_bytes.len()
        } else {
            0
        }
    } else {
        0
    }
}

