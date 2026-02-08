# Chrome DOM Diff 测试框架

> **艹，这是老王我写的测试框架文档！**
> **看完这个文档你就知道怎么跑测试了！**

---

## 📋 测试框架概览

```
tests/
├── unit/                           # 单元测试
│   ├── handler_test.go            # Go后端处理器测试
│   └── message_handler_test.rs    # Rust转发服务消息处理测试
├── integration/                    # 集成测试
│   └── websocket_integration_test.rs
├── protocol/                       # 协议测试
│   └── websocket_protocol_test.sh # WebSocket协议测试脚本
├── e2e/                           # 端到端测试
│   ├── docker-compose.test.yml   # Docker测试环境配置
│   └── dom-capture.spec.ts       # Playwright E2E测试
└── performance/                   # 性能测试
    └── load_test.js              # k6性能测试脚本
```

---

## 🚀 快速开始

### 前置条件

```bash
# 安装依赖工具
go install github.com/golang/mock/mockgen@latest
npm install -g wscat
npm install -g @playwright/test
go install github.com/grafana/k6/cmd/k6@latest
```

### 运行所有测试

```bash
# Rust转发服务测试
cd forwarding-service
cargo test --all
cargo test --release

# Go后端测试
cd artifacts/platform-backend
go test ./... -v
go test ./... -race -cover

# E2E测试
cd artifacts/platform-backend/tests/e2e
npm install
npm run test:e2e

# 性能测试
cd forwarding-service/tests/performance
k6 run load_test.js
```

---

## 📝 单元测试

### Rust转发服务单元测试

```bash
cd forwarding-service

# 运行所有单元测试
cargo test --lib

# 运行特定测试
cargo test test_server_creation

# 运行测试并显示输出
cargo test -- --nocapture

# 运行测试并生成覆盖率报告
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

### Go后端单元测试

```bash
cd artifacts/platform-backend

# 运行所有单元测试
go test ./tests/unit/... -v

# 运行特定测试
go test ./tests/unit/... -run TestHealthCheck -v

# 运行测试并生成覆盖率报告
go test ./... -coverprofile=coverage.out
go tool cover -html=coverage.out -o coverage.html
```

---

## 🔗 集成测试

### WebSocket集成测试

```bash
cd forwarding-service

# 先启动转发服务
cargo run &

# 运行集成测试
cargo test --test integration

# 或者使用协议测试脚本
./tests/protocol/websocket_protocol_test.sh
```

### 协议测试

```bash
cd forwarding-service/tests/protocol

# 确保转发服务正在运行
# 然后运行协议测试
./websocket_protocol_test.sh

# 或者手动测试
wscat -c "ws://localhost:8080"
# 然后输入测试消息
```

---

## 🎭 E2E测试

### 使用Docker Compose启动测试环境

```bash
cd artifacts/platform-backend/tests/e2e

# 启动测试环境
docker-compose -f docker-compose.test.yml up -d

# 查看日志
docker-compose -f docker-compose.test.yml logs -f

# 运行E2E测试
npm run test:e2e

# 停止环境
docker-compose -f docker-compose.test.yml down
```

### 使用Playwright运行E2E测试

```bash
cd artifacts/platform-backend/tests/e2e

# 安装依赖
npm install

# 运行所有E2E测试
npm run test:e2e

# 运行特定测试
npm run test:e2e -- --grep "E2E-001"

# 调试模式（打开浏览器窗口）
npm run test:e2e -- --debug

# 生成测试报告
npm run test:e2e -- --reporter=html
```

---

## ⚡ 性能测试

### 使用k6运行性能测试

```bash
cd forwarding-service/tests/performance

# 运行性能测试
k6 run load_test.js

# 指定并发用户数
PLUGIN_COUNT=100 k6 run load_test.js

# 指定测试持续时间
TEST_DURATION=10m k6 run load_test.js

# 生成HTML报告
k6 run --out json=test-results.json load_test.js
```

### 性能测试指标

| 指标 | 目标 | 测试方法 |
|------|------|----------|
| 并发连接数 | 100 | k6并发测试 |
| 消息吞吐量 | 1000 msg/s | k6压力测试 |
| 响应时间 (P95) | < 100ms | k6延迟测试 |
| 内存使用 | < 100MB | 监控工具 |
| CPU使用率 | < 50% | 监控工具 |

---

## 📊 测试报告

### 生成测试报告

```bash
# 单元测试覆盖率报告
cd forwarding-service
cargo tarpaulin --out Html

# Go测试覆盖率报告
cd artifacts/platform-backend
go test ./... -coverprofile=coverage.out
go tool cover -html=coverage.out

# E2E测试报告
cd tests/e2e
npm run test:e2e -- --reporter=html

# 性能测试报告
cd tests/performance
k6 run --out json=test-results.json load_test.js
```

### 填写测试报告

每次测试完成后，填写测试报告：

```bash
# 复制模板
cp docs/TEST_REPORT_TEMPLATE.md test-reports/TEST_REPORT_$(date +%Y%m%d).md

# 填写测试结果
vim test-reports/TEST_REPORT_$(date +%Y%m%d).md
```

---

## 🧪 测试场景

### 单元测试覆盖

- [x] 消息序列化/反序列化
- [x] 消息类型验证
- [x] 错误处理
- [x] 配置加载
- [x] HTTP接口处理
- [x] WebSocket连接管理

### 集成测试覆盖

- [x] WebSocket连接建立
- [x] 消息发送接收
- [x] 心跳机制
- [x] 断线重连
- [x] 并发连接

### E2E测试覆盖

- [x] 插件注册流程
- [x] DOM捕获命令执行
- [x] XPath查询命令执行
- [x] 页面跳转命令执行
- [x] 日志上报和聚合
- [x] 多插件并发连接

---

## 🐛 调试测试

### 调试Rust测试

```bash
# 使用lldb调试
rust-lldb target/debug/deps/test_name

# 添加调试输出
cargo test -- --nocapture

# 只运行失败的测试
cargo test -- --fail-fast
```

### 调试Go测试

```bash
# 使用delve调试
dlv test ./tests/unit/...

# 添加调试输出
go test ./tests/unit/... -v

# 只运行失败的测试
go test ./tests/unit/... -run TestFailed
```

### 调试E2E测试

```bash
# 使用Playwright Inspector
npm run test:e2e -- --debug

# 使用 headed 模式
npm run test:e2e -- --headed

# 慢动作模式
npm run test:e2e -- --slow-mo=1000
```

---

## 📌 持续集成

### GitHub Actions示例

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Setup Go
        uses: actions/setup-go@v4
        with:
          go-version: '1.21'

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '20'

      - name: Run Rust tests
        run: |
          cd forwarding-service
          cargo test --all

      - name: Run Go tests
        run: |
          cd artifacts/platform-backend
          go test ./... -v

      - name: Run E2E tests
        run: |
          cd artifacts/platform-backend/tests/e2e
          npm install
          npm run test:e2e
```

---

## 🚨 常见问题

### Q: wscat连接失败？

A: 检查转发服务是否正在运行：
```bash
ps aux | grep forwarding-service
netstat -tlnp | grep 8080
```

### Q: E2E测试超时？

A: 增加测试超时时间：
```typescript
test.setTimeout(60000); // 60秒
```

### Q: 性能测试内存不足？

A: 减少并发用户数：
```bash
PLUGIN_COUNT=50 k6 run load_test.js
```

---

**艹，看完这个文档你应该知道怎么跑测试了！有问题问老王我！**
