/**
 * 认证相关API
 * 老王注：处理用户注册、登录、登出这些认证相关的破事
 * 严格对齐后端 handler/user.go 的接口
 */

import request, { removeToken } from '@/utils/request'
import type { ApiResponse } from '@/types/api'
import type { LoginRequest, RegisterRequest, LoginResponse, User, ApiKey, CreateApiKeyRequest } from '@/types/api'

/**
 * 用户登录
 * POST /api/v1/users/login
 * @param data 登录请求参数 { email, password }
 * @returns 登录响应，包含token和用户信息
 */
export const login = async (data: LoginRequest): Promise<LoginResponse> => {
  const response = await request.post<ApiResponse<LoginResponse>>('/users/login', data)
  return response.data.data
}

/**
 * 用户注册
 * POST /api/v1/users/register
 * @param data 注册请求参数 { email, password }
 * @returns 注册后的用户信息
 */
export const register = async (data: RegisterRequest): Promise<User> => {
  const response = await request.post<ApiResponse<User>>('/users/register', data)
  return response.data.data
}

/**
 * 获取当前用户信息
 * GET /api/v1/users/me
 * @returns 当前用户信息
 */
export const getCurrentUser = async (): Promise<User> => {
  const response = await request.get<ApiResponse<User>>('/users/me')
  return response.data.data
}

/**
 * 更新当前用户信息
 * PUT /api/v1/users/me
 * @param data 更新数据
 * @returns 更新后的用户信息
 */
export const updateCurrentUser = async (data: Record<string, any>): Promise<User> => {
  const response = await request.put<ApiResponse<User>>('/users/me', data)
  return response.data.data
}

/**
 * 创建API密钥
 * POST /api/v1/api-keys
 * @param data API密钥信息
 * @returns 创建的API密钥
 */
export const createApiKey = async (data: CreateApiKeyRequest): Promise<ApiKey> => {
  const response = await request.post<ApiResponse<ApiKey>>('/api-keys', data)
  return response.data.data
}

/**
 * 列出API密钥
 * GET /api/v1/api-keys
 * @returns API密钥列表
 */
export const listApiKeys = async (): Promise<ApiKey[]> => {
  const response = await request.get<ApiResponse<ApiKey[]>>('/api-keys')
  return response.data.data
}

/**
 * 撤销API密钥
 * DELETE /api/v1/api-keys/:id
 * @param id API密钥ID
 */
export const revokeApiKey = async (id: string): Promise<void> => {
  await request.delete(`/api-keys/${id}`)
}

/**
 * 用户登出（前端清除token）
 * 老王注：这个只是前端的登出，后端没有专门的登出接口
 * 调用后清除本地存储的token
 */
export const logout = (): void => {
  removeToken()
}
