/**
 * 应用全局状态管理 - 老王出品
 * 管理应用级别的配置和状态，别瞎搞
 */
import { defineStore } from 'pinia'
import { ref } from 'vue'

export interface SidebarItem {
  path: string
  name: string
  icon: string
  title: string
}

export const useAppStore = defineStore('app', () => {
  // 侧边栏折叠状态
  const isSidebarCollapsed = ref<boolean>(false)

  // 主题模式 - 支持 light/dark/auto
  const theme = ref<'light' | 'dark' | 'auto'>('light')

  // 加载状态
  const isLoading = ref<boolean>(false)

  // 当前页面标题
  const pageTitle = ref<string>('仪表盘')

  // 侧边栏菜单配置 - 平台管理系统菜单
  const sidebarMenus = ref<SidebarItem[]>([
    {
      path: '/dashboard',
      name: 'Dashboard',
      icon: 'House',
      title: '仪表盘'
    },
    {
      path: '/services',
      name: 'Services',
      icon: 'Monitor',
      title: '服务管理'
    },
    {
      path: '/tasks',
      name: 'Tasks',
      icon: 'List',
      title: '任务管理'
    },
    {
      path: '/logs',
      name: 'Logs',
      icon: 'Document',
      title: '日志管理'
    },
    {
      path: '/settings',
      name: 'Settings',
      icon: 'Setting',
      title: '设置'
    }
  ])

  /**
   * 切换侧边栏折叠状态
   */
  function toggleSidebar() {
    isSidebarCollapsed.value = !isSidebarCollapsed.value
  }

  /**
   * 设置主题
   */
  function setTheme(newTheme: 'light' | 'dark' | 'auto') {
    theme.value = newTheme
    // 持久化到localStorage
    localStorage.setItem('theme', newTheme)
    applyTheme(newTheme)
  }

  /**
   * 应用主题到DOM
   */
  function applyTheme(themeMode: 'light' | 'dark' | 'auto') {
    const html = document.documentElement
    if (themeMode === 'auto') {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      html.classList.toggle('dark', prefersDark)
    } else {
      html.classList.toggle('dark', themeMode === 'dark')
    }
  }

  /**
   * 设置加载状态
   */
  function setLoading(loading: boolean) {
    isLoading.value = loading
  }

  /**
   * 设置页面标题
   */
  function setPageTitle(title: string) {
    pageTitle.value = title
  }

  /**
   * 初始化应用 - 从本地存储恢复状态
   */
  function initApp() {
    const savedTheme = localStorage.getItem('theme') as 'light' | 'dark' | 'auto' | null
    if (savedTheme) {
      theme.value = savedTheme
      applyTheme(savedTheme)
    }
  }

  return {
    isSidebarCollapsed,
    theme,
    isLoading,
    pageTitle,
    sidebarMenus,
    toggleSidebar,
    setTheme,
    setLoading,
    setPageTitle,
    initApp
  }
}, {
  persist: true
})
