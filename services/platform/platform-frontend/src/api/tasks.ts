// 艹！任务管理API接口
// 任务列表、创建、更新、删除、执行

import request from '@/utils/request'
import type { ApiResponse, PaginatedResponse, Task, TaskCreateRequest, TaskUpdateRequest } from '@/types/api'

// 获取任务列表
export const getTasks = (params?: { page?: number; pageSize?: number; status?: string; serviceId?: number }) => {
  return request.get<any, ApiResponse<PaginatedResponse<Task>>>('/tasks', { params })
}

// 获取任务详情
export const getTask = (id: number) => {
  return request.get<any, ApiResponse<Task>>(`/tasks/${id}`)
}

// 创建任务
export const createTask = (data: TaskCreateRequest) => {
  return request.post<any, ApiResponse<Task>>('/tasks', data)
}

// 更新任务
export const updateTask = (id: number, data: TaskUpdateRequest) => {
  return request.put<any, ApiResponse<Task>>(`/tasks/${id}`, data)
}

// 删除任务
export const deleteTask = (id: number) => {
  return request.delete<any, ApiResponse<null>>(`/tasks/${id}`)
}

// 执行任务
export const executeTask = (id: number) => {
  return request.post<any, ApiResponse<Task>>(`/tasks/${id}/execute`)
}
