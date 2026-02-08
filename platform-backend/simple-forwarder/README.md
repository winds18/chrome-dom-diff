# Simple Forwarder

> 艹，这是临时转发服务模拟器
> 老王用Go写的，用于测试Chrome插件通信协议

## 项目说明

这个项目是一个临时的转发服务，用于在Rust版本完成之前测试整个系统的通信协议。

**功能：**
- 接受Chrome插件的WebSocket连接（ws://127.0.0.1:8080/ws）
- 处理插件注册、心跳、结果消息
- 连接公网平台并上报数据
- 转发平台命令到插件

## 快速开始

### 1. 安装依赖

```bash
cd simple-forwarder
go mod download
```

### 2. 运行服务

```bash
# 直接运行
go run cmd/main.go

# 编译运行
go build -o bin/simple-forwarder cmd/main.go
./bin/simple-forwarder
```

### 3. 连接公网平台（可选）

设置环境变量：

```bash
export PLATFORM_URL="http://localhost:8080"
export PLATFORM_API_KEY="your-api-key"

go run cmd/main.go
```

## WebSocket协议

### 插件 → 转发服务

#### 注册消息
```json
{
  "type": "register",
  "data": {
    "plugin_id": "chrome-extension-xxx",
    "tab_id": 123,
    "url": "https://example.com",
    "title": "Example Page",
    "capabilities": ["dom_capture", "xpath_query"]
  }
}
```

#### 心跳消息
```json
{
  "type": "heartbeat",
  "data": {
    "plugin_id": "chrome-extension-xxx",
    "tab_id": 123
  }
}
```

#### 结果消息
```json
{
  "type": "result",
  "data": {
    "command_id": "cmd-uuid",
    "status": "success",
    "data": {...}
  }
}
```

### 转发服务 → 插件

#### 注册确认
```json
{
  "type": "register_ack",
  "data": {
    "forwarder_id": "forwarder-uuid",
    "status": "ok"
  }
}
```

#### 心跳确认
```json
{
  "type": "heartbeat_ack",
  "data": {
    "timestamp": 1640000000
  }
}
```

#### 命令消息
```json
{
  "type": "command",
  "data": {
    "command_id": "cmd-uuid",
    "action": "xpath_query",
    "payload": {
      "xpath": "//h1[@id='title']"
    }
  }
}
```

## 项目结构

```
simple-forwarder/
├── cmd/
│   └── main.go           # 入口文件
├── internal/
│   ├── protocol/         # 协议定义
│   │   └── types.go
│   ├── server/           # WebSocket服务端
│   │   ├── server.go
│   │   └── plugin.go
│   └── client/           # 平台客户端
│       └── platform.go
├── go.mod
└── README.md
```

## 测试

### 使用Chrome插件测试

1. 启动转发服务
2. Chrome插件连接 `ws://127.0.0.1:8080/ws`
3. 查看日志输出

### 使用wscat测试

```bash
# 安装wscat
npm install -g wscat

# 连接
wscat -c ws://127.0.0.1:8080/ws

# 发送注册消息
> {"type":"register","data":{"plugin_id":"test-plugin","tab_id":1,"url":"https://test.com","title":"Test","capabilities":["dom_capture"]}}
```

## 后续计划

- [ ] 完善命令转发逻辑
- [ ] 实现结果上报到平台
- [ ] 添加更多日志
- [ ] 编写单元测试
- [ ] 等Rust版本完成后替换

## 作者

老王
