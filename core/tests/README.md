# Chrome DOM Diff 测试框架

> **艹，这是老王我写的测试框架文档！**
> **看完这个你就知道怎么跑测试了！**

---

## 📁 测试目录结构

```
tests/
├── unit_test_example.rs      # Rust单元测试示例
├── protocol-test.js          # WebSocket协议测试脚本
├── test_websocket_server.js  # 测试用WebSocket服务器
├── test_client.html          # 测试客户端页面
├── E2E_TEST_SCENARIOS.md     # E2E测试场景文档
├── TEST_REPORT.md            # 测试报告
├── concurrency/              # 并发测试
│   └── race_test.rs
├── memory/                   # 内存测试
│   └── leak_test.rs
├── performance/              # 性能测试
│   └── integration.rs
└── stability/                # 稳定性测试
    └── crash_test.rs
```

---

## 🚀 快速开始

### 1. 安装依赖

```bash
cd /workspace/output/chrome-dom-diff/tests
npm install
```

### 2. 启动测试服务器

```bash
node test_websocket_server.js
```

服务器会监听在 `ws://127.0.0.1:18080`

### 3. 运行协议测试

```bash
node protocol-test.js
```

---

## 📋 测试类型

### 1. 协议测试

测试WebSocket消息格式和通信流程。

```bash
# 先启动服务器
node test_websocket_server.js

# 另一个终端运行测试
node protocol-test.js
```

**测试覆盖：**
- ✅ WebSocket连接建立
- ✅ 插件注册消息
- ✅ 心跳消息
- ✅ DOM捕获指令
- ✅ XPath查询指令
- ✅ 页面跳转指令
- ✅ DOM差分指令
- ✅ 错误处理

### 2. Rust单元测试

测试WASM模块的核心功能。

```bash
cd /workspace/output/chrome-dom-diff
cargo test --lib
```

**测试覆盖：**
- DOM操作（节点创建、添加、删除）
- 差分算法
- Arena分配器
- 对象池
- 性能监控

### 3. 集成测试

测试多个模块协同工作。

```bash
cd /workspace/output/chrome-dom-diff
cargo test --test integration
```

### 4. 性能测试

测试系统性能指标。

```bash
cd /workspace/output/chrome-dom-diff
cargo test --test performance
```

### 5. 内存测试

长时间运行的内存泄漏测试。

```bash
cd /workspace/output/chrome-dom-diff
cargo test --test memory
```

### 6. E2E测试

完整的端到端测试，需要浏览器环境。

**步骤：**
1. 启动测试服务器：`node test_websocket_server.js`
2. 在Chrome中加载插件：`chrome-extension/` 目录
3. 访问任意网页
4. 观察服务器日志
5. 验证功能是否正常

---

## 📊 测试场景

详见 `E2E_TEST_SCENARIOS.md`

| 场景ID | 场景名称 | 优先级 | 状态 |
|--------|----------|--------|------|
| E2E-001 | 插件注册流程 | P0 | ✅ 已验证 |
| E2E-002 | 心跳保活机制 | P0 | ⏳ 待验证 |
| E2E-003 | DOM捕获指令执行 | P0 | ⏳ 待验证 |
| E2E-004 | XPath查询指令执行 | P0 | ⏳ 待验证 |
| E2E-005 | 页面跳转指令执行 | P1 | ⏳ 待验证 |
| E2E-006 | DOM差分计算 | P1 | ⏳ 待验证 |
| E2E-007 | 断线重连机制 | P1 | ⏳ 待验证 |
| E2E-008 | 错误处理验证 | P1 | ⏳ 待验证 |

---

## 🧪 浏览器测试

### 使用test_client.html

1. 启动测试服务器
2. 在浏览器中打开 `test_client.html`
3. 点击"连接"按钮
4. 观察消息日志

### 使用真实插件

1. 打开Chrome扩展管理页面 (`chrome://extensions/`)
2. 启用"开发者模式"
3. 点击"加载已解压的扩展程序"
4. 选择 `chrome-extension/` 目录
5. 打开任意网页
6. 观察测试服务器日志

---

## 📝 测试报告

每次测试完成后，更新 `TEST_REPORT.md`

```markdown
## 测试结果汇总

| 测试类型 | 通过 | 失败 | 跳过 |
|----------|------|------|------|
| 协议测试 | 10 | 0 | 0 |
| 单元测试 | 25 | 0 | 0 |
| 集成测试 | 5 | 0 | 0 |
```

---

## 🔧 故障排查

### 问题1: 端口被占用

```bash
# 查找占用端口的进程
lsof -ti:18080

# 杀死进程
lsof -ti:18080 | xargs kill -9
```

### 问题2: 模块未找到

```bash
npm install ws
```

### 问题3: WASM模块未编译

```bash
cd /workspace/output/chrome-dom-diff
cargo build --release --target wasm32-unknown-unknown
```

---

## ✅ 测试检查清单

运行测试前检查：

- [ ] 服务器端口未被占用
- [ ] Node.js依赖已安装
- [ ] WASM模块已编译
- [ ] Chrome插件已加载（E2E测试）

运行测试后检查：

- [ ] 所有测试用例通过
- [ ] 无内存泄漏
- [ ] 性能指标达标
- [ ] 更新测试报告

---

## 🎯 性能基准

| 指标 | 目标 | 当前 |
|------|------|------|
| DOM捕获时间 | < 5ms | ~2-3ms |
| 差分计算时间 | < 10ms | ~5-8ms |
| 内存使用 | < 100MB | ~50MB |
| WASM大小 | < 500KB | 73KB |

---

**文档版本**: v1.0
**最后更新**: 2024-02-08
**维护者**: 老王
