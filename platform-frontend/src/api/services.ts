/**
 * 服务管理相关API
 * 老王注：管理那些该死的服务，注册、查询、发命令这些破事
 * 严格对齐后端 handler/service.go 的接口
 */

import request from '@/utils/request'
import type { ApiResponse, PaginatedResponse, PageQuery } from '@/types/api'
import type { Service, ServiceCommandRequest, RegisterServiceRequest } from '@/types/api'

/**
 * 服务注册
 * POST /api/v1/services/register
 * @param data 注册请求参数
 * @returns 注册后的服务信息
 */
export const registerService = async (data: RegisterServiceRequest): Promise<Service> => {
  const response = await request.post<ApiResponse<Service>>('/services/register', data)
  return response.data.data
}

/**
 * 查询服务列表
 * GET /api/v1/services
 * @param params 分页和筛选参数
 * @returns 服务列表（分页）
 */
export const getServices = async (params?: PageQuery & { status?: string }): Promise<PaginatedResponse<Service>> => {
  const response = await request.get<ApiResponse<PaginatedResponse<Service>>>('/services', { params })
  return response.data.data
}

/**
 * 查询服务详情
 * @param id 服务ID
 * @returns 服务详情
 */
export const getServiceById = async (id: string): Promise<Service> => {
  const response = await request.get<ApiResponse<Service>>(`/services/${id}`)
  return response.data.data
}

/**
 * 删除服务
 * @param id 服务ID
 * @returns 删除结果
 */
export const deleteService = async (id: string): Promise<void> => {
  await request.delete<ApiResponse<void>>(`/services/${id}`)
}

/**
 * 发送服务命令
 * POST /api/v1/services/:id/command
 * @param id 服务ID
 * @param command 命令参数
 * @returns 命令执行响应
 */
export const sendServiceCommand = async (id: string, command: ServiceCommandRequest): Promise<any> => {
  const response = await request.post<ApiResponse<any>>(`/services/${id}/command`, command)
  return response.data.data
}

/**
 * 启动服务快捷方法
 * @param id 服务ID
 * @returns 命令执行响应
 */
export const startService = async (id: string): Promise<any> => {
  return sendServiceCommand(id, { command: 'start' })
}

/**
 * 停止服务快捷方法
 * @param id 服务ID
 * @returns 命令执行响应
 */
export const stopService = async (id: string): Promise<any> => {
  return sendServiceCommand(id, { command: 'stop' })
}

/**
 * 重启服务快捷方法
 * @param id 服务ID
 * @returns 命令执行响应
 */
export const restartService = async (id: string): Promise<any> => {
  return sendServiceCommand(id, { command: 'restart' })
}

/**
 * 查询服务状态快捷方法
 * @param id 服务ID
 * @returns 命令执行响应
 */
export const getServiceStatus = async (id: string): Promise<any> => {
  return sendServiceCommand(id, { command: 'status' })
}
