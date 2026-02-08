# Platform Frontend

> Chrome DOM Diff 系统的公网控制平台前端服务
> 老王出品：Vue 3 + TypeScript + Element Plus

## 项目简介

这是Chrome DOM Diff系统的平台管理前端，用于管理注册的服务、创建和执行抓取任务、查看系统日志等。

## 技术栈

- **Vue 3.5+** - 渐进式JavaScript框架
- **TypeScript** - 类型安全的JavaScript超集
- **Element Plus** - 基于Vue 3的组件库
- **Vue Router 5** - 官方路由管理器
- **Pinia** - Vue状态管理库
- **Axios** - HTTP客户端
- **Vite** - 下一代前端构建工具

## 功能模块

### 1. 用户认证
- 用户登录/注册
- JWT Token管理
- API密钥管理

### 2. 服务管理
- 服务列表查看
- 服务注册
- 服务详情查看
- 服务控制（启动/停止/重启）

### 3. 任务管理
- 任务列表查看
- 任务创建（DOM捕获/对比/数据提取）
- 任务执行
- 任务详情查看

### 4. 日志管理
- 日志查询（按级别/来源/时间筛选）
- 日志详情查看
- 实时日志流（SSE）

### 5. 系统设置
- 主题切换（浅色/深色/自动）
- 用户信息管理

## 快速开始

### 1. 安装依赖

```bash
npm install
```

### 2. 开发模式

```bash
npm run dev
```

访问 http://localhost:5173

### 3. 生产构建

```bash
npm run build
```

构建产物在 `dist/` 目录

### 4. 预览构建

```bash
npm run preview
```

## 环境配置

创建 `.env` 文件配置环境变量：

```bash
# API基础URL（开发环境通过vite代理到后端）
VITE_API_BASE_URL=/api/v1
```

## 项目结构

```
platform-frontend/
├── src/
│   ├── api/              # API模块
│   │   ├── auth.ts       # 认证API
│   │   ├── service.ts    # 服务API
│   │   ├── task.ts       # 任务API
│   │   ├── log.ts        # 日志API
│   │   └── index.ts      # 统一导出
│   ├── assets/           # 静态资源
│   ├── components/       # 公共组件
│   ├── router/           # 路由配置
│   ├── stores/           # Pinia状态管理
│   │   ├── app.ts        # 应用状态
│   │   ├── user.ts       # 用户状态
│   │   ├── service.ts    # 服务状态
│   │   ├── task.ts       # 任务状态
│   │   └── log.ts        # 日志状态
│   ├── styles/           # 全局样式
│   ├── types/            # TypeScript类型定义
│   ├── utils/            # 工具函数
│   │   └── request.ts    # Axios封装
│   ├── views/            # 页面组件
│   │   ├── Login.vue     # 登录页
│   │   ├── Layout.vue    # 布局组件
│   │   ├── Dashboard.vue # 仪表盘
│   │   ├── Services.vue  # 服务管理
│   │   ├── ServiceDetail.vue # 服务详情
│   │   ├── Tasks.vue     # 任务管理
│   │   ├── TaskDetail.vue # 任务详情
│   │   ├── Logs.vue      # 日志管理
│   │   ├── Settings.vue  # 设置
│   │   └── NotFound.vue  # 404
│   ├── App.vue           # 根组件
│   └── main.ts           # 入口文件
├── public/               # 公共资源
├── index.html            # HTML模板
├── vite.config.ts        # Vite配置
├── tsconfig.json         # TypeScript配置
└── package.json          # 项目配置
```

## API对接

前端通过Vite代理转发API请求到后端：

```typescript
// vite.config.ts
server: {
  port: 5173,
  proxy: {
    '/api': {
      target: 'http://localhost:8081',
      changeOrigin: true,
      rewrite: (path) => path.replace(/^\/api/, '')
    }
  }
}
```

## 开发规范

### 代码风格

- 使用TypeScript严格模式
- 遵循Vue 3 Composition API风格
- 使用`<script setup>`语法糖
- 组件命名使用PascalCase
- 文件命名使用PascalCase（组件）或kebab-case（工具）

### 提交规范

```bash
feat: 新功能
fix: 修复bug
docs: 文档更新
style: 代码格式调整
refactor: 代码重构
perf: 性能优化
test: 测试相关
chore: 构建/工具链
```

## 浏览器支持

- Chrome >= 87
- Firefox >= 78
- Safari >= 14
- Edge >= 88

## 作者

老王

## 许可证

MIT
