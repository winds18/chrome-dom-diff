/**
 * API 类型定义
 * 老王注：严格对齐后端Go handler的返回格式，别tm乱改
 * 后端返回格式：{ message: string, data: T } 或 { error: string }
 */

// ==================== 通用类型 ====================

/**
 * 统一API响应格式（成功）
 */
export interface ApiResponse<T = any> {
  message: string
  data: T
}

/**
 * 统一API响应格式（错误）
 */
export interface ApiErrorResponse {
  error: string
}

/**
 * 分页响应格式（老王注：后端返回的是 total_page 不是 totalPages）
 */
export interface PaginatedResponse<T = any> {
  items: T[]
  total: number
  page: number
  page_size: number
  total_page: number
}

/**
 * 分页查询参数
 */
export interface PageQuery {
  page?: number
  page_size?: number
}

// ==================== 用户相关类型 ====================

/**
 * 用户角色
 */
export enum UserRole {
  ADMIN = 'admin',
  USER = 'user'
}

/**
 * 用户信息（对齐后端handler返回）
 */
export interface User {
  id: string
  email: string
  role: UserRole
  created_at: string
  updated_at?: string
  last_login?: string
}

/**
 * 登录请求（后端用的是email不是username）
 */
export interface LoginRequest {
  email: string
  password: string
}

/**
 * 注册请求
 */
export interface RegisterRequest {
  email: string
  password: string
}

/**
 * 登录响应（对齐后端）
 */
export interface LoginResponse {
  token: string
  user: User
}

/**
 * API密钥信息
 */
export interface ApiKey {
  id: string
  name: string
  key: string
  is_active: boolean
  created_at: string
  last_used?: string
  expires_at?: string
}

/**
 * 创建API密钥请求
 */
export interface CreateApiKeyRequest {
  name: string
  scopes?: string[]
  expires_in_days?: number
}

// ==================== 服务相关类型 ====================

/**
 * 服务状态（对齐后端model）
 */
export enum ServiceStatus {
  ONLINE = 'online',
  OFFLINE = 'offline',
  ERROR = 'error'
}

/**
 * 服务信息（对齐后端handler返回）
 */
export interface Service {
  id: string
  name: string
  description?: string
  status: ServiceStatus
  version?: string
  ip_address?: string
  port?: number
  last_heartbeat?: string
  capabilities?: string[]
  tags?: string[]
  metadata?: Record<string, any>
  created_at: string
  updated_at?: string
}

/**
 * 服务命令类型
 */
export enum ServiceCommandType {
  START = 'start',
  STOP = 'stop',
  RESTART = 'restart',
  STATUS = 'status',
  CONFIG = 'config'
}

/**
 * 服务命令请求
 */
export interface ServiceCommandRequest {
  command: string
  params?: Record<string, any>
}

/**
 * 注册服务请求（对齐后端service.RegisterServiceRequest）
 */
export interface RegisterServiceRequest {
  name: string
  description?: string
  type?: string
  version?: string
  ip_address?: string
  port?: number
  capabilities?: string[]
  tags?: string[]
  metadata?: Record<string, any>
}

// ==================== 任务相关类型 ====================

/**
 * 任务状态（对齐后端model）
 */
export enum TaskStatus {
  PENDING = 'pending',
  RUNNING = 'running',
  COMPLETED = 'completed',
  FAILED = 'failed',
  CANCELLED = 'cancelled'
}

/**
 * 任务类型
 */
export enum TaskType {
  DOM_CAPTURE = 'dom_capture',
  DOM_DIFF = 'dom_diff',
  DATA_EXTRACTION = 'data_extraction',
  CUSTOM = 'custom'
}

/**
 * 调度类型
 */
export enum ScheduleType {
  MANUAL = 'manual',
  CRON = 'cron',
  INTERVAL = 'interval'
}

/**
 * 任务信息（对齐后端handler返回）
 */
export interface Task {
  id: string
  name: string
  description?: string
  task_type: TaskType
  status: TaskStatus
  schedule_type: ScheduleType
  schedule_config?: Record<string, any>
  target_service: string
  config?: Record<string, any>
  retry_count?: number
  retry_interval?: number
  created_at: string
  updated_at: string
}

/**
 * 创建任务请求（对齐后端service.CreateTaskRequest）
 */
export interface CreateTaskRequest {
  name: string
  description?: string
  task_type: TaskType
  schedule_type: ScheduleType
  target_service_id: string
  config?: Record<string, any>
  schedule_config?: Record<string, any>
  retry_count?: number
  retry_interval_secs?: number
}

/**
 * 更新任务请求（对齐后端service.UpdateTaskRequest）
 */
export interface UpdateTaskRequest {
  name?: string
  description?: string
  status?: TaskStatus
  schedule_type?: ScheduleType
  target_service_id?: string
  config?: Record<string, any>
  schedule_config?: Record<string, any>
  retry_count?: number
  retry_interval_secs?: number
}

/**
 * 任务执行响应（对齐后端）
 */
export interface TaskExecutionResponse {
  execution_id: string
  task_id: string
  status: TaskStatus
  started_at: string
}

/**
 * 任务查询参数
 */
export interface TaskQuery extends PageQuery {
  status?: TaskStatus
  task_type?: TaskType
  target_service?: string
}

// ==================== 日志相关类型 ====================

/**
 * 日志级别
 */
export enum LogLevel {
  DEBUG = 'debug',
  INFO = 'info',
  WARN = 'warn',
  ERROR = 'error',
  FATAL = 'fatal'
}

/**
 * 日志条目（对齐后端handler返回）
 */
export interface Log {
  id: string
  timestamp: string
  level: LogLevel
  source?: string
  message: string
  metadata?: Record<string, any>
  service_id?: string
  task_id?: string
}

/**
 * 日志查询参数（对齐后端repository.LogFilter）
 */
export interface LogQuery extends PageQuery {
  level?: LogLevel
  source?: string
  message?: string
  service_id?: string
  task_id?: string
  start_time?: string
  end_time?: string
}

// ==================== WebSocket相关类型 ====================

/**
 * WebSocket消息类型
 */
export enum WSMessageType {
  SERVICE_UPDATE = 'service_update',
  TASK_UPDATE = 'task_update',
  LOG_ENTRY = 'log_entry',
  ERROR = 'error',
  PONG = 'pong'
}

/**
 * WebSocket消息
 */
export interface WSMessage {
  type: WSMessageType
  data: any
  timestamp: string
}

// ==================== 表单相关类型 ====================

/**
 * 通用表单字段验证规则
 */
export interface FormRule {
  required?: boolean
  message?: string
  trigger?: 'blur' | 'change'
  min?: number
  max?: number
  pattern?: RegExp
  validator?: (rule: any, value: any, callback: any) => void
}
