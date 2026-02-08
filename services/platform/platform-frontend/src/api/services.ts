// 艹！服务管理API接口
// 服务列表、详情、命令发送

import request from '@/utils/request'
import type { ApiResponse, PaginatedResponse, Service, ServiceCommand } from '@/types/api'

// 获取服务列表
export const getServices = (params?: { page?: number; pageSize?: number; status?: string }) => {
  return request.get<any, ApiResponse<PaginatedResponse<Service>>>('/services', { params })
}

// 获取服务详情
export const getService = (id: number) => {
  return request.get<any, ApiResponse<Service>>(`/services/${id}`)
}

// 删除服务
export const deleteService = (id: number) => {
  return request.delete<any, ApiResponse<null>>(`/services/${id}`)
}

// 发送命令到服务
export const sendCommand = (id: number, command: ServiceCommand) => {
  return request.post<any, ApiResponse<any>>(`/services/${id}/command`, command)
}
