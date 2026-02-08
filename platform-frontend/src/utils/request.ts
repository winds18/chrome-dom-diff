/**
 * Axios HTTP请求封装
 * 老王注：这个文件负责配置axios实例和拦截器，别tm乱动
 * 后端返回格式：{ message: string, data: T } 或 { error: string }
 */

import axios, { type AxiosInstance, type AxiosError, type AxiosResponse, type InternalAxiosRequestConfig } from 'axios'
import type { ApiResponse, ApiErrorResponse } from '@/types/api'

/**
 * API基础配置
 */
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api/v1'
const API_TIMEOUT = 30000

/**
 * 创建axios实例
 */
const request: AxiosInstance = axios.create({
  baseURL: API_BASE_URL,
  timeout: API_TIMEOUT,
  headers: {
    'Content-Type': 'application/json'
  }
})

/**
 * Token存储键名
 */
const TOKEN_KEY = 'auth_token'

/**
 * 获取存储的token
 */
export const getToken = (): string | null => {
  return localStorage.getItem(TOKEN_KEY)
}

/**
 * 存储token
 */
export const setToken = (token: string): void => {
  localStorage.setItem(TOKEN_KEY, token)
}

/**
 * 清除token
 */
export const removeToken = (): void => {
  localStorage.removeItem(TOKEN_KEY)
}

/**
 * 请求拦截器
 * 自动附加JWT token到请求头
 */
request.interceptors.request.use(
  (config: InternalAxiosRequestConfig) => {
    const token = getToken()
    if (token && config.headers) {
      config.headers.Authorization = `Bearer ${token}`
    }
    return config
  },
  (error: AxiosError) => {
    console.error('请求错误:', error)
    return Promise.reject(error)
  }
)

/**
 * 响应拦截器
 * 统一处理响应和错误
 * 老王注：后端返回格式是 { message, data } 或 { error }
 */
request.interceptors.response.use(
  (response: AxiosResponse<ApiResponse>) => {
    // 后端成功响应格式：{ message: string, data: T }
    // 直接返回response，让调用者通过response.data.data获取数据
    return response
  },
  (error: AxiosError<ApiErrorResponse | ApiResponse>) => {
    const { response, message } = error

    // 处理网络错误
    if (!response) {
      console.error('网络错误:', message)
      return Promise.reject(new Error('网络连接失败，请检查网络设置'))
    }

    const { status, data } = response

    // 处理401未授权 - 清除token并跳转登录
    if (status === 401) {
      removeToken()
      // 只在不在登录页面时跳转
      if (!window.location.pathname.includes('/login')) {
        window.location.href = '/login'
      }
      const errorMsg = (data as ApiErrorResponse)?.error || '登录已过期，请重新登录'
      return Promise.reject(new Error(errorMsg))
    }

    // 处理403禁止访问
    if (status === 403) {
      const errorMsg = (data as ApiErrorResponse)?.error || '没有权限执行此操作'
      return Promise.reject(new Error(errorMsg))
    }

    // 处理404
    if (status === 404) {
      const errorMsg = (data as ApiErrorResponse)?.error || '请求的资源不存在'
      return Promise.reject(new Error(errorMsg))
    }

    // 处理409冲突
    if (status === 409) {
      const errorMsg = (data as ApiErrorResponse)?.error || '资源冲突'
      return Promise.reject(new Error(errorMsg))
    }

    // 处理500服务器错误
    if (status >= 500) {
      const errorMsg = (data as ApiErrorResponse)?.error || '服务器错误，请稍后重试'
      return Promise.reject(new Error(errorMsg))
    }

    // 其他错误使用后端返回的错误消息
    const errorMsg = (data as ApiErrorResponse)?.error || message || '请求失败'
    return Promise.reject(new Error(errorMsg))
  }
)

/**
 * 导出axios实例供API模块使用
 */
export default request

/**
 * 导出配置供其他模块使用
 */
export { API_BASE_URL, API_TIMEOUT }
