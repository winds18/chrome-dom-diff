// 艹！应用全局状态管理
// 管理全局配置和状态

import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  // 侧边栏折叠状态
  const isSidebarCollapsed = ref<boolean>(false)

  // 加载状态
  const isLoading = ref<boolean>(false)

  // 切换侧边栏
  const toggleSidebar = () => {
    isSidebarCollapsed.value = !isSidebarCollapsed.value
  }

  // 设置加载状态
  const setLoading = (loading: boolean) => {
    isLoading.value = loading
  }

  return {
    isSidebarCollapsed,
    isLoading,
    toggleSidebar,
    setLoading
  }
})
