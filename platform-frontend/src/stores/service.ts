/**
 * 服务状态管理 - 老王出品
 * 管理服务列表和状态，别TM乱动
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'
import * as serviceApi from '@/api/services'
import type { Service } from '@/types/api'

export const useServiceStore = defineStore('service', () => {
  // 服务列表状态
  const services = ref<Service[]>([])
  const currentService = ref<Service | null>(null)
  const loading = ref<boolean>(false)
  const error = ref<string | null>(null)

  /**
   * 加载服务列表
   */
  async function loadServices() {
    loading.value = true
    error.value = null
    try {
      const result = await serviceApi.getServices()
      services.value = result.items
    } catch (err: any) {
      error.value = err.message || '加载服务列表失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 加载服务详情
   */
  async function loadService(id: string) {
    loading.value = true
    error.value = null
    try {
      currentService.value = await serviceApi.getServiceById(id)
      return currentService.value
    } catch (err: any) {
      error.value = err.message || '加载服务详情失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 注册新服务
   */
  async function registerService(data: any) {
    loading.value = true
    error.value = null
    try {
      const newService = await serviceApi.registerService(data)
      services.value.push(newService)
      return newService
    } catch (err: any) {
      error.value = err.message || '注册服务失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 删除服务
   */
  async function deleteService(id: string) {
    loading.value = true
    error.value = null
    try {
      await serviceApi.deleteService(id)
      services.value = services.value.filter(s => s.id !== id)
      if (currentService.value?.id === id) {
        currentService.value = null
      }
    } catch (err: any) {
      error.value = err.message || '删除服务失败'
      throw err
    } finally {
      loading.value = false
    }
  }

  /**
   * 发送命令到服务
   */
  async function sendCommand(id: string, command: string, params?: Record<string, any>) {
    loading.value = true
    error.value = null
    try {
      await serviceApi.sendServiceCommand(id, { command, params })
    } catch (err: any) {
      error.value = err.message || '发送命令失败'
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
    services,
    currentService,
    loading,
    error,
    loadServices,
    loadService,
    registerService,
    deleteService,
    sendCommand,
    clearError
  }
})
