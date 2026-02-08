/**
 * 日志管理相关API
 * 老王注：查询日志、实时日志流这些破事
 * 严格对齐后端 handler/log.go 的接口
 */

import request from '@/utils/request'
import type { ApiResponse, PaginatedResponse, LogQuery, LogLevel } from '@/types/api'
import type { Log } from '@/types/api'
import { API_BASE_URL } from '@/utils/request'

/**
 * 查询日志列表
 * GET /api/v1/logs
 * @param params 查询参数（分页、筛选）
 * @returns 日志列表（分页）
 */
export const getLogs = async (params?: LogQuery): Promise<PaginatedResponse<Log>> => {
  const response = await request.get<ApiResponse<PaginatedResponse<Log>>>('/logs', { params })
  return response.data.data
}

/**
 * 获取实时日志流URL
 * 老王注：返回SSE或WebSocket的URL，用于订阅实时日志
 * @param params 查询参数
 * @returns 日志流URL
 */
export const getLogStreamUrl = (params?: LogQuery): string => {
  const queryString = params ? new URLSearchParams(params as any).toString() : ''
  return `${API_BASE_URL}/logs/stream${queryString ? '?' + queryString : ''}`
}

/**
 * 按服务ID查询日志
 * @param serviceId 服务ID
 * @param params 额外的查询参数
 * @returns 该服务的日志列表
 */
export const getLogsByServiceId = async (serviceId: string, params?: Omit<LogQuery, 'service_id'>): Promise<PaginatedResponse<Log>> => {
  return getLogs({ ...params, service_id: serviceId })
}

/**
 * 按任务ID查询日志
 * @param taskId 任务ID
 * @param params 额外的查询参数
 * @returns 该任务的日志列表
 */
export const getLogsByTaskId = async (taskId: string, params?: Omit<LogQuery, 'task_id'>): Promise<PaginatedResponse<Log>> => {
  return getLogs({ ...params, task_id: taskId })
}

/**
 * 按日志级别查询
 * @param level 日志级别
 * @param params 额外的查询参数
 * @returns 该级别的日志列表
 */
export const getLogsByLevel = async (level: LogLevel, params?: Omit<LogQuery, 'level'>): Promise<PaginatedResponse<Log>> => {
  return getLogs({ ...params, level })
}

/**
 * 按时间范围查询日志
 * @param startTime 开始时间
 * @param endTime 结束时间
 * @param params 额外的查询参数
 * @returns 时间范围内的日志列表
 */
export const getLogsByTimeRange = async (
  startTime: string,
  endTime: string,
  params?: Omit<LogQuery, 'start_time' | 'end_time'>
): Promise<PaginatedResponse<Log>> => {
  return getLogs({ ...params, start_time: startTime, end_time: endTime })
}

/**
 * 创建EventSource连接用于实时日志
 * 老王注：这个是SSE（Server-Sent Events）连接，用于接收实时日志推送
 * @param params 查询参数
 * @returns EventSource实例
 */
export const createLogStream = (params?: LogQuery): EventSource => {
  const url = getLogStreamUrl(params)
  return new EventSource(url)
}

/**
 * WebSocket连接URL
 * 老王注：用于WebSocket实时通信，可以接收服务更新、任务更新等推送
 * @returns WebSocket URL
 */
export const getWebSocketUrl = (): string => {
  // 老王注：把http(s)换成ws(s)
  return API_BASE_URL.replace(/^http/, 'ws') + '/ws'
}

/**
 * 创建WebSocket连接
 * @param protocols WebSocket协议（可选）
 * @returns WebSocket实例
 */
export const createWebSocket = (protocols?: string | string[]): WebSocket => {
  const url = getWebSocketUrl()
  return new WebSocket(url, protocols)
}
