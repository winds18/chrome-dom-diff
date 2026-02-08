/**
 * API模块统一导出入口
 * 老王注：别tm到处乱import，统一从这儿导出
 *
 * 使用方式：
 * import { login, register } from '@/api'
 * 或
 * import { login } from '@/api/auth'
 */

// ==================== 认证相关 ====================
export {
  login,
  register,
  getCurrentUser,
  updateCurrentUser,
  createApiKey,
  listApiKeys,
  revokeApiKey,
  logout
} from './auth'

// ==================== 服务管理 ====================
export {
  registerService,
  getServices,
  getServiceById,
  deleteService,
  sendServiceCommand,
  startService,
  stopService,
  restartService,
  getServiceStatus
} from './services'

// ==================== 任务管理 ====================
export {
  createTask,
  getTasks,
  getTaskById,
  updateTask,
  deleteTask,
  executeTask,
  cancelTask,
  batchDeleteTasks,
  getTasksByServiceId,
  getTasksByStatus,
  getTasksByType
} from './tasks'

// ==================== 日志管理 ====================
export {
  getLogs,
  getLogStreamUrl,
  getLogsByServiceId,
  getLogsByTaskId,
  getLogsByLevel,
  getLogsByTimeRange,
  createLogStream,
  getWebSocketUrl,
  createWebSocket
} from './logs'
