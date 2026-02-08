// 艹！日志管理API接口
// 日志查询、实时日志流

import request from '@/utils/request'
import type { ApiResponse, Log, LogQuery } from '@/types/api'

// 获取日志列表
export const getLogs = (params?: LogQuery) => {
  return request.get<any, ApiResponse<Log[]>>('/logs', { params })
}

// 获取实时日志流
export const getLogStream = () => {
  return request.get<any, ApiResponse<Log[]>>('/logs/stream')
}
