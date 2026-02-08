// 艹，这是simple-forwarder的Go模块定义
// 老王写的临时转发服务，用于测试协议

module github.com/oldwang/simple-forwarder

go 1.21

require (
	github.com/google/uuid v1.6.0
	github.com/gorilla/websocket v1.5.3
	go.uber.org/zap v1.27.0
)

require go.uber.org/multierr v1.10.0 // indirect
