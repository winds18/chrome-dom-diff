/**
 * 日志状态管理 - 老王出品
 * 管理日志查询，别TM乱动
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as logApi from '@/api/logs'
import type { Log, LogQuery } from '@/types/api'

export const useLogStore = defineStore('log', () => {
  // 日志列表状态
  const logs = ref<Log[]>([])
  const loading = ref<boolean>(false)
  const error = ref<string | null>(null)

  // 分页信息
  const total = ref<number>(0)
  const page = ref<number>(1)
  const pageSize = ref<number>(50)
  const totalPages = ref<number>(0)

  // 查询条件
  const currentQuery = ref<LogQuery>({})

  /**
   * 查询日志
   */
  async function queryLogs(query: LogQuery) {
    loading.value = true
    error.value = null
    currentQuery.value = { ...query }
    try {
      const result = await logApi.getLogs(query)
      logs.value = result.items
      total.value = result.total
      page.value = result.page
      pageSize.value = result.page_size
      totalPages.value = result.total_page
    } catch (err: any) {
      error.value = err.message || '查询日志失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 刷新日志（使用当前查询条件）
   */
  async function refreshLogs() {
    await queryLogs(currentQuery.value)
  }

  /**
   * 清除日志列表
   */
  function clearLogs() {
    logs.value = []
    total.value = 0
    currentQuery.value = {}
  }

  /**
   * 清除错误
   */
  function clearError() {
    error.value = null
  }

  return {
    logs,
    loading,
    error,
    total,
    page,
    pageSize,
    totalPages,
    currentQuery,
    queryLogs,
    refreshLogs,
    clearLogs,
    clearError
  }
})
