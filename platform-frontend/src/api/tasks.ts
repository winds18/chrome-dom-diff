/**
 * 任务管理相关API
 * 老王注：处理任务的创建、查询、更新、删除这些破事
 * 严格对齐后端 handler/task.go 的接口
 */

import request from '@/utils/request'
import type { ApiResponse, PaginatedResponse, TaskQuery } from '@/types/api'
import type { Task, CreateTaskRequest, UpdateTaskRequest, TaskExecutionResponse } from '@/types/api'
import { TaskStatus, TaskType } from '@/types/api'

/**
 * 创建任务
 * POST /api/v1/tasks
 * @param data 任务创建请求参数
 * @returns 创建后的任务信息
 */
export const createTask = async (data: CreateTaskRequest): Promise<Task> => {
  const response = await request.post<ApiResponse<Task>>('/tasks', data)
  return response.data.data
}

/**
 * 查询任务列表
 * GET /api/v1/tasks
 * @param params 查询参数（分页、筛选）
 * @returns 任务列表（分页）
 */
export const getTasks = async (params?: TaskQuery): Promise<PaginatedResponse<Task>> => {
  const response = await request.get<ApiResponse<PaginatedResponse<Task>>>('/tasks', { params })
  return response.data.data
}

/**
 * 查询任务详情
 * GET /api/v1/tasks/:id
 * @param id 任务ID
 * @returns 任务详情
 */
export const getTaskById = async (id: string): Promise<Task> => {
  const response = await request.get<ApiResponse<Task>>(`/tasks/${id}`)
  return response.data.data
}

/**
 * 更新任务
 * PUT /api/v1/tasks/:id
 * @param id 任务ID
 * @param data 更新请求参数
 * @returns 更新后的任务信息
 */
export const updateTask = async (id: string, data: UpdateTaskRequest): Promise<Task> => {
  const response = await request.put<ApiResponse<Task>>(`/tasks/${id}`, data)
  return response.data.data
}

/**
 * 删除任务
 * DELETE /api/v1/tasks/:id
 * @param id 任务ID
 * @returns 删除结果
 */
export const deleteTask = async (id: string): Promise<void> => {
  await request.delete(`/tasks/${id}`)
}

/**
 * 执行任务
 * POST /api/v1/tasks/:id/execute
 * @param id 任务ID
 * @returns 执行结果
 */
export const executeTask = async (id: string): Promise<TaskExecutionResponse> => {
  const response = await request.post<ApiResponse<TaskExecutionResponse>>(`/tasks/${id}/execute`)
  return response.data.data
}

/**
 * 取消任务
 * 老王注：通过更新任务状态为cancelled来取消
 * @param id 任务ID
 * @returns 更新后的任务信息
 */
export const cancelTask = async (id: string): Promise<Task> => {
  return updateTask(id, { status: TaskStatus.CANCELLED })
}

/**
 * 批量删除任务
 * @param ids 任务ID列表
 * @returns 删除结果
 */
export const batchDeleteTasks = async (ids: string[]): Promise<void> => {
  await Promise.all(ids.map(id => deleteTask(id)))
}

/**
 * 按服务ID查询任务
 * @param targetService 服务ID
 * @param params 额外的查询参数
 * @returns 该服务的任务列表
 */
export const getTasksByServiceId = async (targetService: string, params?: Omit<TaskQuery, 'target_service'>): Promise<PaginatedResponse<Task>> => {
  return getTasks({ ...params, target_service: targetService })
}

/**
 * 按状态查询任务
 * @param status 任务状态
 * @param params 额外的查询参数
 * @returns 该状态的任务列表
 */
export const getTasksByStatus = async (status: TaskStatus, params?: Omit<TaskQuery, 'status'>): Promise<PaginatedResponse<Task>> => {
  return getTasks({ ...params, status })
}

/**
 * 按任务类型查询
 * @param taskType 任务类型
 * @param params 额外的查询参数
 * @returns 该类型的任务列表
 */
export const getTasksByType = async (taskType: TaskType, params?: Omit<TaskQuery, 'task_type'>): Promise<PaginatedResponse<Task>> => {
  return getTasks({ ...params, task_type: taskType })
}
