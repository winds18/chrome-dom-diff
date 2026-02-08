/**
 * 任务状态管理 - 老王出品
 * 管理任务列表和状态，别TM乱动
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as taskApi from '@/api/tasks'
import type { Task, TaskQuery } from '@/types/api'

export const useTaskStore = defineStore('task', () => {
  // 任务列表状态
  const tasks = ref<Task[]>([])
  const currentTask = ref<Task | null>(null)
  const loading = ref<boolean>(false)
  const error = ref<string | null>(null)

  // 分页信息
  const total = ref<number>(0)
  const page = ref<number>(1)
  const pageSize = ref<number>(20)
  const totalPages = ref<number>(0)

  /**
   * 加载任务列表
   */
  async function loadTasks(query?: TaskQuery) {
    loading.value = true
    error.value = null
    try {
      const result = await taskApi.getTasks(query)
      tasks.value = result.items
      total.value = result.total
      page.value = result.page
      pageSize.value = result.page_size
      totalPages.value = result.total_page
    } catch (err: any) {
      error.value = err.message || '加载任务列表失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 加载任务详情
   */
  async function loadTask(id: string) {
    loading.value = true
    error.value = null
    try {
      currentTask.value = await taskApi.getTaskById(id)
      return currentTask.value
    } catch (err: any) {
      error.value = err.message || '加载任务详情失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 创建任务
   */
  async function createTask(data: any) {
    loading.value = true
    error.value = null
    try {
      const newTask = await taskApi.createTask(data)
      tasks.value.unshift(newTask)
      total.value += 1
      return newTask
    } catch (err: any) {
      error.value = err.message || '创建任务失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 更新任务
   */
  async function updateTask(id: string, data: any) {
    loading.value = true
    error.value = null
    try {
      const updatedTask = await taskApi.updateTask(id, data)
      const index = tasks.value.findIndex(t => t.id === id)
      if (index !== -1) {
        tasks.value[index] = updatedTask
      }
      if (currentTask.value?.id === id) {
        currentTask.value = updatedTask
      }
      return updatedTask
    } catch (err: any) {
      error.value = err.message || '更新任务失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 删除任务
   */
  async function deleteTask(id: string) {
    loading.value = true
    error.value = null
    try {
      await taskApi.deleteTask(id)
      tasks.value = tasks.value.filter(t => t.id !== id)
      total.value -= 1
      if (currentTask.value?.id === id) {
        currentTask.value = null
      }
    } catch (err: any) {
      error.value = err.message || '删除任务失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 执行任务
   */
  async function executeTask(id: string) {
    loading.value = true
    error.value = null
    try {
      const result = await taskApi.executeTask(id)
      // 刷新任务状态
      await loadTask(id)
      return result
    } catch (err: any) {
      error.value = err.message || '执行任务失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 清除错误
   */
  function clearError() {
    error.value = null
  }

  return {
    tasks,
    currentTask,
    loading,
    error,
    total,
    page,
    pageSize,
    totalPages,
    loadTasks,
    loadTask,
    createTask,
    updateTask,
    deleteTask,
    executeTask,
    clearError
  }
})
