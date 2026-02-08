# Chrome DOM Diff - 端到端测试场景文档

> **版本**: v1.0
> **日期**: 2024-02-08
> **作者**: 老王

---

## 📋 测试场景概览

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

## E2E-001: 插件注册流程

### 场景描述
验证Chrome插件连接到转发服务后能成功注册，并获得正确的配置参数。

### 前置条件
- 转发服务运行在 `ws://127.0.0.1:18080`
- Chrome插件已加载

### 测试步骤
```
1. 插件建立WebSocket连接
2. 插件发送register消息
3. 转发服务回复register_ack
4. 插件存储heartbeat_interval配置
```

### 消息流程

**步骤1: 插件发送注册消息**
```json
{
  "type": "register",
  "plugin_id": "chrome-extension-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "tab_id": 123,
  "url": "https://www.amazon.com/product-page",
  "title": "Amazon Product Page",
  "capabilities": [
    "dom_capture",
    "xpath_query",
    "page_navigate",
    "dom_diff"
  ]
}
```

**步骤2: 转发服务回复确认**
```json
{
  "type": "register_ack",
  "plugin_id": "chrome-extension-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "heartbeat_interval": 30,
  "timestamp": 1640000000
}
```

### 预期结果
- ✅ 插件成功连接到服务器
- ✅ register消息格式正确
- ✅ 服务器返回register_ack
- ✅ heartbeat_interval为30（秒）

### 验证方法
```javascript
// 在插件background.js中
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'getConnectionStatus') {
    sendResponse({
      connected: ws && ws.readyState === WebSocket.OPEN,
      registered: isRegistered,
      heartbeatInterval: config.heartbeatInterval
    });
  }
});
```

---

## E2E-002: 心跳保活机制

### 场景描述
验证插件能按配置间隔发送心跳，服务器能正确回复心跳确认。

### 前置条件
- 插件已成功注册
- 心跳间隔配置为30秒

### 测试步骤
```
1. 等待30秒
2. 插件自动发送heartbeat消息
3. 服务器回复heartbeat_ack
4. 重复3次验证稳定性
```

### 消息流程

**插件发送心跳**
```json
{
  "type": "heartbeat",
  "plugin_id": "chrome-extension-xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx",
  "tab_id": 123,
  "timestamp": 1640030000
}
```

**服务器回复**
```json
{
  "type": "heartbeat_ack",
  "timestamp": 1640030000
}
```

### 预期结果
- ✅ 每30秒发送一次心跳
- ✅ 服务器正确回复heartbeat_ack
- ✅ 心跳超时后触发重连

### 验证方法
```javascript
// 记录心跳时间
let lastHeartbeatTime = Date.now();

ws.addEventListener('message', (event) => {
  const data = JSON.parse(event.data);
  if (data.type === 'heartbeat_ack') {
    const interval = Date.now() - lastHeartbeatTime;
    console.log(`心跳间隔: ${interval}ms (预期: ~30000ms)`);
    lastHeartbeatTime = Date.now();
  }
});
```

---

## E2E-003: DOM捕获指令执行

### 场景描述
验证转发服务能下发DOM捕获指令，插件能正确执行并返回结果。

### 前置条件
- 插件已注册并在线
- 测试页面已加载

### 测试步骤
```
1. 服务器下发dom_capture指令
2. 插件接收指令
3. 插件调用WASM模块捕获DOM
4. 插件返回result消息
5. 验证结果数据完整性
```

### 消息流程

**服务器下发指令**
```json
{
  "type": "command",
  "command_id": "cmd-uuid-xxxx",
  "action": "dom_capture",
  "payload": {},
  "timestamp": 1640000000
}
```

**插件返回结果**
```json
{
  "type": "result",
  "command_id": "cmd-uuid-xxxx",
  "status": "success",
  "timestamp": 1640000002,
  "data": {
    "tree_id": 1,
    "node_count": 1234,
    "duration": "2.45",
    "url": "https://www.amazon.com/product-page",
    "title": "Amazon Product Page"
  }
}
```

### 预期结果
- ✅ command_id与请求匹配
- ✅ status为success
- ✅ data.node_count > 0
- ✅ data.duration < 5000ms
- ✅ URL和Title正确

### 性能要求
- DOM捕获时间 < 5秒
- 内存使用 < 100MB

---

## E2E-004: XPath查询指令执行

### 场景描述
验证XPath查询指令能正确执行，返回匹配的节点信息。

### 测试用例

| 用例ID | XPath表达式 | 预期匹配数 |
|--------|-------------|-----------|
| XP-001 | `//h1` | 1 |
| XP-002 | `//p` | ≥2 |
| XP-003 | `//*[@id='productTitle']` | 1 |
| XP-004 | `//div[@class='content']//p` | ≥1 |
| XP-005 | `//a[@href]` | ≥1 |
| XP-006 | `//*[contains(text(),'Amazon')]` | ≥1 |

### 消息流程

**服务器下发指令**
```json
{
  "type": "command",
  "command_id": "cmd-uuid-xxxx",
  "action": "xpath_query",
  "payload": {
    "xpath": "//h1[@id='productTitle']"
  },
  "timestamp": 1640000000
}
```

**插件返回结果**
```json
{
  "type": "result",
  "command_id": "cmd-uuid-xxxx",
  "status": "success",
  "timestamp": 1640000001,
  "data": {
    "xpath": "//h1[@id='productTitle']",
    "count": 1,
    "results": [
      {
        "id": 42,
        "type": "element",
        "tag_name": "h1",
        "xpath": "//*[@id='productTitle']",
        "text_content": "Amazon Product Title",
        "attributes": {
          "id": "productTitle",
          "class": "product-title"
        },
        "attr_count": 2
      }
    ],
    "url": "https://www.amazon.com/product-page",
    "title": "Amazon Product Page"
  }
}
```

### 预期结果
- ✅ 每个测试用例的匹配数正确
- ✅ 返回的节点属性完整
- ✅ text_content正确
- ✅ 查询时间 < 1秒

---

## E2E-005: 页面跳转指令执行

### 场景描述
验证插件能执行页面跳转指令，并等待页面加载完成。

### 消息流程

**服务器下发指令**
```json
{
  "type": "command",
  "command_id": "cmd-uuid-xxxx",
  "action": "page_navigate",
  "payload": {
    "url": "https://www.amazon.com/other-product",
    "wait_for_load": true
  },
  "timestamp": 1640000000
}
```

**插件返回结果**
```json
{
  "type": "result",
  "command_id": "cmd-uuid-xxxx",
  "status": "success",
  "timestamp": 1640000050,
  "data": {
    "url": "https://www.amazon.com/other-product",
    "title": "Other Product Page",
    "tab_id": 123
  }
}
```

### 预期结果
- ✅ 页面成功跳转
- ✅ final_url与请求URL匹配
- ✅ 返回正确的页面标题
- ✅ 跳转时间 < 30秒

---

## E2E-006: DOM差分计算

### 场景描述
验证DOM差分功能能正确检测页面变化。

### 测试步骤
```
1. 执行dom_diff_prepare（准备基准DOM）
2. 模拟页面变化（动态内容更新）
3. 执行dom_diff_compute
4. 验证变化检测结果
```

### 消息流程

**步骤1: 准备差分**
```json
{
  "type": "command",
  "command_id": "cmd-uuid-001",
  "action": "dom_diff_prepare",
  "payload": {},
  "timestamp": 1640000000
}
```

**步骤2: 计算差分**
```json
{
  "type": "command",
  "command_id": "cmd-uuid-002",
  "action": "dom_diff_compute",
  "payload": {},
  "timestamp": 1640000050
}
```

**返回结果**
```json
{
  "type": "result",
  "command_id": "cmd-uuid-002",
  "status": "success",
  "timestamp": 1640000055,
  "data": {
    "changes": 15,
    "inserts": 8,
    "deletes": 5,
    "moves": 2,
    "duration": "5.23",
    "url": "https://www.amazon.com/product-page",
    "title": "Amazon Product Page"
  }
}
```

### 预期结果
- ✅ 能检测到插入的节点
- ✅ 能检测到删除的节点
- ✅ 能检测到移动的节点
- ✅ 差分计算时间 < 10秒

---

## E2E-007: 断线重连机制

### 场景描述
验证网络中断后插件能自动重连。

### 测试步骤
```
1. 正常通信中
2. 停止转发服务（模拟网络中断）
3. 等待插件检测到断开
4. 重启转发服务
5. 验证插件自动重连
6. 验证重新注册
```

### 预期结果
- ✅ 插件能检测到连接断开
- ✅ 插件尝试重连（指数退避）
- ✅ 重连成功后重新注册
- ✅ 重连不影响功能

### 重连策略
| 次数 | 间隔时间 |
|------|----------|
| 1 | 5秒 |
| 2 | 10秒 |
| 3 | 20秒 |
| 4 | 40秒 |
| 5+ | 最大60秒 |

---

## E2E-008: 错误处理验证

### 场景描述
验证各种错误情况下插件能正确处理。

### 测试用例

| 用例ID | 错误场景 | 预期行为 |
|--------|----------|----------|
| ERR-001 | 无效的action | 返回错误: Unknown action |
| ERR-002 | XPath为空 | 返回错误: XPath is required |
| ERR-003 | URL为空 | 返回错误: URL is required |
| ERR-004 | 无效的XPath | 返回错误: XPath query failed |
| ERR-005 | 无活动标签页 | 返回错误: No active tab |

### 错误响应格式
```json
{
  "type": "result",
  "command_id": "cmd-uuid-xxxx",
  "status": "error",
  "timestamp": 1640000000,
  "data": {
    "error": "错误描述信息"
  }
}
```

---

## 🧪 测试执行

### 自动化测试
```bash
# 1. 启动测试服务器
cd /workspace/output/chrome-dom-diff/tests
node test_websocket_server.js

# 2. 运行协议测试
node protocol-test.js

# 3. 在Chrome中加载插件进行完整测试
```

### 手动测试
1. 打开Chrome扩展管理页面
2. 加载 `/workspace/output/chrome-dom-diff/chrome-extension/`
3. 访问测试页面
4. 观察服务器日志
5. 验证各项功能

---

## 📊 测试报告

测试完成后，填写测试报告模板：
- [ ] 更新TEST_REPORT.md
- [ ] 记录通过/失败的测试
- [ ] 记录发现的问题
- [ ] 跟踪问题修复状态

---

**文档版本**: v1.0
**最后更新**: 2024-02-08
**维护者**: 老王
