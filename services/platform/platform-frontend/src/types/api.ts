// 艹！这是老王的前端API类型定义
// 别tm乱改这些类型，必须和后端保持一致！

// ========== 通用类型 ==========
export interface ApiResponse<T = any> {
  code: number
  message: string
  data: T
}

export interface PaginatedResponse<T> {
  items: T[]
  total: number
  page: number
  pageSize: number
}

// ========== 用户认证类型 ==========
export interface LoginRequest {
  username: string
  password: string
}

export interface RegisterRequest {
  username: string
  password: string
  email?: string
}

export interface LoginResponse {
  token: string
  user: UserInfo
}

export interface UserInfo {
  id: number
  username: string
  email?: string
  createdAt: string
}

// ========== 服务管理类型 ==========
export interface Service {
  id: number
  pluginId: string
  tabId: number
  url: string
  title: string
  capabilities: string[]
  status: 'online' | 'offline'
  lastSeen: string
  createdAt: string
}

export interface ServiceCommand {
  action: string
  payload?: any
}

// ========== 任务管理类型 ==========
export interface Task {
  id: number
  serviceId: number
  action: string
  payload?: any
  status: 'pending' | 'running' | 'completed' | 'failed'
  result?: any
  error?: string
  createdAt: string
  updatedAt: string
  startedAt?: string
  completedAt?: string
}

export interface TaskCreateRequest {
  serviceId: number
  action: string
  payload?: any
}

export interface TaskUpdateRequest {
  status?: Task['status']
  result?: any
  error?: string
}

// ========== 日志管理类型 ==========
export interface Log {
  id: number
  level: 'debug' | 'info' | 'warn' | 'error'
  message: string
  metadata?: any
  createdAt: string
}

export interface LogQuery {
  level?: Log['level']
  startTime?: string
  endTime?: string
  limit?: number
}

// ========== API密钥类型 ==========
export interface ApiKey {
  id: number
  name: string
  key: string
  createdAt: string
  lastUsed?: string
}
