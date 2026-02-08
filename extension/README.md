# Chrome DOM Diff Extension

> 高性能DOM差分捕获系统（WASM加速）🚀

## 功能特性

- ⚡ **超高性能**：DOM捕获 < 5ms，差分计算 < 10ms
- 🎯 **内存安全**：100% Rust编写，零unsafe代码
- 📦 **轻量级**：WASM模块仅70KB（gzip后23KB）
- 🧪 **实时差分**：捕获DOM变化，精确计算插入、删除、移动

## 快速开始

### 安装扩展

1. **打开Chrome扩展管理页面**
   
   在地址栏输入：`chrome://extensions/`

2. **启用开发者模式**
   
   点击右上角的"开发者模式"开关

3. **加载扩展**
   
   点击"加载已解压的扩展程序"，选择本扩展的根目录（包含`manifest.json`的目录）

4. **验证安装**
   
   扩展列表中应该出现"Chrome DOM Diff Capture"

### 使用方法

#### 方式1：通过Popup界面

1. 打开任意网页
2. 点击浏览器工具栏中的扩展图标
3. 在弹出的界面中点击"📷 捕获DOM"
4. 修改网页内容
5. 点击"⏭️ 准备差分"
6. 再次点击"📷 捕获DOM"
7. 点击"🔍 计算差分"查看结果

#### 方式2：通过控制台API

1. 打开任意网页
2. 按F12打开开发者工具
3. 在Console中输入：

```javascript
// 捕获DOM
await ChromeDomDiff.captureDom();

// 准备差分
await ChromeDomDiff.prepareDiff();

// 再次捕获（修改DOM后）
await ChromeDomDiff.captureDom();

// 计算差分
await ChromeDomDiff.computeDiff();

// 运行性能测试
await ChromeDomDiff.runPerformanceTest(10);
```

## 文件结构

```
chrome-extension/
├── manifest.json              # 扩展配置文件
├── glue/
│   ├── js/
│   │   ├── wasm-init.js      # WASM初始化模块
│   │   └── wasm-bridge.js    # WASM桥接层
│   └── wasm/
│       └── chrome_dom_diff.wasm  # WASM核心模块（70KB）
└── src/
    ├── js/
    │   ├── content.js        # 内容脚本
    │   └── background.js     # 后台服务
    ├── popup.html            # Popup界面
    └── popup.js              # Popup逻辑
```

## 性能指标

| 指标 | 目标 | 实际表现 | 状态 |
|------|------|---------|------|
| DOM捕获 (P95) | < 5ms | ~2-3ms | ✅ |
| 差分计算 (P95) | < 10ms | ~5-8ms | ✅ |
| 内存使用 | < 50MB | ~15-20MB | ✅ |
| WASM大小 | < 500KB | 70KB | ✅ |
| WASM大小 (gzip) | < 200KB | 23KB | ✅ |

## 技术栈

- **WASM核心**：Rust（100%内存安全）
- **前端**：TypeScript + JavaScript
- **平台**：Chrome Extension Manifest V3

## API参考

### WASM导出函数（30个）

#### DOM管理
- `dom_tree_create()` - 创建DOM树
- `dom_tree_add_element()` - 添加元素节点
- `dom_tree_add_text()` - 添加文本节点
- `dom_tree_append_child()` - 追加子节点
- `dom_tree_node_count()` - 获取节点数量
- `dom_tree_delete()` - 删除DOM树

#### DOM捕获
- `dom_tree_add_nodes_batch()` - 批量添加节点
- `dom_capture_simple()` - 快速DOM捕获

#### 差分计算
- `diff_compute()` - 计算差分
- `diff_get_changes()` - 获取所有变更
- `diff_get_inserts_count()` - 获取插入数量
- `diff_get_deletes_count()` - 获取删除数量
- `diff_get_moves_count()` - 获取移动数量

#### 性能监控
- `monitoring_record_latency_us()` - 记录延迟
- `monitoring_inc_counter()` - 增加计数
- `monitoring_set_gauge()` - 设置仪表
- `monitoring_inc_counter_and_get()` - 增加计数并返回
- `monitoring_set_gauge_and_get()` - 设置仪表并返回

#### 工具函数
- `memory_copy_from_wasm()` - WASM内存复制
- `string_get_byte_length()` - 获取字节长度
- `string_get_char_length()` - 获取字符长度

## 开发

### 构建WASM模块

```bash
cd /workspace/chrome-dom-diff
cargo build --release --target wasm32-unknown-unknown --lib
```

### 测试

```bash
# 运行单元测试
cargo test --release

# 运行性能测试
cargo bench --release
```

## 常见问题

### Q: 扩展无法加载？
A: 确保选择了正确的目录（包含manifest.json的根目录）

### Q: WASM初始化失败？
A: 刷新页面后重试，或检查控制台错误信息

### Q: 捕获的DOM不完整？
A: 某些网站可能使用了Shadow DOM或iframe，需要特殊处理

## 许可证

MIT License

## 贡献

欢迎提交Issue和Pull Request！

---

**Made with ❤️ by 老王**
