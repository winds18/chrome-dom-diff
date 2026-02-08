# Platform Backend

> 艹，这是Chrome DOM Diff系统的公网控制平台后端服务
> 老王用Go + Gin写的，简洁优雅，别tm乱改

## 项目结构

```
platform-backend/
├── cmd/                    # 入口文件
│   └── main.go
├── internal/               # 内部包
│   ├── handler/           # HTTP处理器
│   ├── middleware/        # 中间件
│   ├── model/             # 数据模型
│   ├── repository/        # 数据仓储
│   ├── service/           # 业务服务
│   └── ws/                # WebSocket服务
├── pkg/                    # 公共包
│   ├── auth/              # 认证工具
│   ├── config/            # 配置管理
│   └── logger/            # 日志工具
├── configs/                # 配置文件
│   └── config.yaml
├── migrations/             # 数据库迁移
├── docs/                   # 文档
├── go.mod
└── README.md
```

## 技术栈

- **Go 1.21+** - 编程语言
- **Gin** - Web框架
- **GORM** - ORM框架
- **PostgreSQL** - 数据库
- **Redis** - 缓存和任务队列
- **JWT** - 认证
- **WebSocket** - 双向通信

## 快速开始

### 1. 安装依赖

```bash
go mod download
```

### 2. 配置数据库

创建PostgreSQL数据库：

```sql
CREATE DATABASE platform_db;
```

### 3. 配置环境

编辑 `configs/config.yaml`：

```yaml
database:
  host: localhost
  port: 5432
  user: postgres
  password: postgres
  database: platform_db

redis:
  host: localhost
  port: 6379

jwt:
  secret: "your-secret-key-change-me"
```

### 4. 运行服务

```bash
# 开发模式
go run cmd/main.go

# 编译
CGO_ENABLED=0 go build -o bin/platform-backend cmd/main.go

# 运行
./bin/platform-backend
```

## API文档

### 用户认证

- `POST /api/v1/users/register` - 用户注册
- `POST /api/v1/users/login` - 用户登录

### 服务管理

- `POST /api/v1/services/register` - 注册服务
- `GET /api/v1/services` - 查询服务列表
- `GET /api/v1/services/:id` - 查询服务详情
- `DELETE /api/v1/services/:id` - 删除服务
- `POST /api/v1/services/:id/command` - 发送命令

### 任务管理

- `POST /api/v1/tasks` - 创建任务
- `GET /api/v1/tasks` - 查询任务列表
- `GET /api/v1/tasks/:id` - 查询任务详情
- `PUT /api/v1/tasks/:id` - 更新任务
- `DELETE /api/v1/tasks/:id` - 删除任务
- `POST /api/v1/tasks/:id/execute` - 执行任务

### 日志管理

- `GET /api/v1/logs` - 查询日志
- `GET /api/v1/logs/stream` - 实时日志流

### WebSocket

- `WS /api/v1/ws` - WebSocket连接

## 开发规范

### 代码风格

- 遵循Go官方代码风格
- 使用`gofmt`格式化代码
- 函数注释要清晰

### 提交规范

- `feat:` 新功能
- `fix:` 修复bug
- `docs:` 文档更新
- `refactor:` 代码重构

## 作者

老王 - <oldwang@example.com>

## 许可证

MIT
