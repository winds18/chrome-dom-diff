// 艹！用户认证API接口
// 登录、注册、登出

import request from '@/utils/request'
import type { ApiResponse, LoginRequest, RegisterRequest, LoginResponse } from '@/types/api'

// 用户登录
export const login = (data: LoginRequest) => {
  return request.post<any, ApiResponse<LoginResponse>>('/users/login', data)
}

// 用户注册
export const register = (data: RegisterRequest) => {
  return request.post<any, ApiResponse<LoginResponse>>('/users/register', data)
}

// 用户登出
export const logout = () => {
  return request.post<any, ApiResponse<null>>('/users/logout')
}

// 获取当前用户信息
export const getCurrentUser = () => {
  return request.get<any, ApiResponse<any>>('/users/me')
}
