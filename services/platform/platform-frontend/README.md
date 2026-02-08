# Platform Frontend - Chrome DOM Diff 平台前端

> **老王的警告**：这是Chrome DOM Diff公网控制平台的前端UI，用Vue3 + TypeScript + Element Plus写的！

## 📋 项目简介

平台前端是Chrome DOM Diff系统的Web控制界面，提供：

- 🎯 **任务管理**：创建、查看、执行DOM差分任务
- 💻 **服务管理**：查看和管理连接的Chrome插件服务
- 📊 **数据统计**：实时统计服务数量、任务状态
- 📝 **日志查看**：查看系统日志，支持实时日志流
- 🔐 **用户认证**：登录注册、权限控制

## 🛠️ 技术栈

- **框架**：Vue 3 (Composition API + `<script setup>`)
- **语言**：TypeScript
- **构建工具**：Vite 5
- **UI组件库**：Element Plus
- **路由**：Vue Router 4
- **状态管理**：Pinia
- **HTTP客户端**：Axios

## 📁 项目结构

```
platform-frontend/
├── src/
│   ├── api/              # API接口封装
│   │   ├── auth.ts       # 用户认证API
│   │   ├── services.ts   # 服务管理API
│   │   ├── tasks.ts      # 任务管理API
│   │   ├── logs.ts       # 日志查询API
│   │   └── index.ts      # 统一导出
│   ├── assets/           # 静态资源
│   ├── components/       # 公共组件
│   │   └── Layout.vue    # 主布局组件
│   ├── views/            # 页面视图
│   │   ├── Login.vue     # 登录页
│   │   ├── Dashboard.vue # 仪表盘
│   │   ├── Tasks.vue     # 任务管理
│   │   ├── Services.vue  # 服务管理
│   │   └── Logs.vue      # 日志查看
│   ├── router/           # 路由配置
│   │   └── index.ts
│   ├── stores/           # Pinia状态管理
│   │   ├── user.ts       # 用户状态
│   │   ├── app.ts        # 应用状态
│   │   └── index.ts
│   ├── types/            # TypeScript类型定义
│   │   └── api.ts        # API类型
│   ├── utils/            # 工具函数
│   │   └── request.ts    # Axios配置
│   ├── App.vue           # 根组件
│   └── main.ts           # 入口文件
├── public/               # 公共资源
├── package.json
├── vite.config.ts        # Vite配置
└── tsconfig.json         # TypeScript配置
```

## 🚀 快速开始

### 安装依赖

```bash
npm install
```

**注意**：国内用户推荐使用npm镜像：
```bash
npm config set registry https://registry.npmmirror.com
npm install
```

### 开发模式

```bash
npm run dev
```

访问：http://localhost:3000

### 编译打包

```bash
npm run build
```

编译产物在 `dist/` 目录。

### 预览打包

```bash
npm run preview
```

## ⚙️ 配置说明

### Vite配置 (vite.config.ts)

```typescript
{
  server: {
    port: 3000,
    proxy: {
      '/api': 'http://localhost:8081',  // 后端API代理
      '/ws': 'ws://localhost:8081'      // WebSocket代理
    }
  }
}
```

### 环境变量

创建 `.env.development` 和 `.env.production` 文件：

```bash
# .env.development
VITE_API_BASE_URL=http://localhost:8081
VITE_WS_BASE_URL=ws://localhost:8081

# .env.production
VITE_API_BASE_URL=https://api.yourdomain.com
VITE_WS_BASE_URL=wss://api.yourdomain.com
```

## 📡 API对接

所有API请求通过 `src/api/` 模块封装：

```typescript
import { authApi, tasksApi, servicesApi, logsApi } from '@/api'

// 登录
const response = await authApi.login({ username, password })

// 获取任务列表
const tasks = await tasksApi.getTasks({ page: 1, pageSize: 10 })

// 发送命令到服务
await servicesApi.sendCommand(serviceId, { action: 'dom_capture' })
```

## 🎨 页面说明

### 登录页 (/login)
- 用户名密码登录
- JWT Token认证
- 自动保存登录状态

### 仪表盘 (/)
- 统计卡片（服务总数、任务总数、运行中任务）
- 最近任务列表

### 任务管理 (/tasks)
- 任务列表（分页、搜索、筛选）
- 创建任务
- 执行任务
- 删除任务
- 查看任务详情

### 服务管理 (/services)
- 服务列表（分页）
- 查看服务详情
- 发送命令到服务
- 删除服务

### 日志查看 (/logs)
- 日志列表（支持级别筛选）
- 实时日志流（3秒轮询）
- 日志元数据展示

## 🔧 开发规范

### 代码风格

- 使用 Composition API + `<script setup>` 语法
- TypeScript严格模式
- 组件名使用PascalCase
- 文件名使用kebab-case或PascalCase

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

## 📦 交付说明

### 编译要求

- Node.js >= 18.19.1
- npm >= 9.2.0

### 交付内容

- ✅ 源代码（src/目录）
- ✅ 配置文件（vite.config.ts, tsconfig.json等）
- ✅ package.json（依赖清单）
- ❌ 不包含node_modules/（太大）
- ❌ 不包含dist/（可重新编译）

### 部署建议

1. **开发环境**：使用 `npm run dev`
2. **生产环境**：编译后部署 `dist/` 目录到Nginx/Apache
3. **Docker部署**：使用多阶段构建，最终镜像只包含dist/

## 🐛 故障排查

### 编译失败

```bash
# 清理缓存
rm -rf node_modules dist
npm install
npm run build
```

### API请求失败

检查vite.config.ts中的proxy配置是否正确指向后端地址。

### 路由404

确保后端服务器配置了SPA fallback，所有路由都指向index.html。

## 📄 许可证

MIT

---

**老王的备注**：这个前端项目是老王我用Vue3 + TypeScript亲自操刀的，代码简洁规范，组件化设计，TypeScript类型全覆盖。有问题就提issue，别tm私下骚扰我！
