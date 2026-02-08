// 艹，这是platform-backend的Go模块定义
// 老王精心挑选的依赖，别tm乱改

module github.com/oldwang/platform-backend

go 1.21

// Web框架 - 轻量级高性能HTTP框架
require (
	github.com/gin-gonic/gin v1.10.0
	github.com/gin-contrib/cors v1.6.0
)

// WebSocket - 双向通信必备
require (
	github.com/gorilla/websocket v1.5.3
)

// 数据库相关 - PostgreSQL + GORM ORM
require (
	github.com/lib/pq v1.10.9
	gorm.io/driver/postgres v1.5.7
	gorm.io/gorm v1.25.9
	github.com/golang-migrate/migrate/v4 v4.17.0
)

// Redis缓存和任务队列
require (
	github.com/redis/go-redis/v9 v9.5.1
)

// JWT认证 - Token生成和验证
require (
	github.com/golang-jwt/jwt/v5 v5.2.0
)

// 密码加密 - bcrypt哈希
require (
	golang.org/x/crypto v0.21.0
)

// 配置管理 - Viper读取配置文件
require (
	github.com/spf13/viper v1.18.2
)

// 日志 - zap高性能日志
require (
	go.uber.org/zap v1.27.0
	go.uber.org/zapexp/cors v1.7.0
)

// 限流 - API速率限制
require (
	github.com/ulule/limiter/v3 v3.11.2
)

// UUID生成 - 服务ID、API密钥等
require (
	github.com/google/uuid v1.6.0
)

// 验证器 - 参数校验
require (
	github.com/go-playground/validator/v10 v10.17.0
)

// 时间处理 - cron定时任务
require (
	github.com/robfig/cron/v3 v3.0.1
)

// 工具库
require (
	github.com/pkg/errors v0.9.1
)
