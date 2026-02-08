/**
 * 用户状态管理 - 老王出品
 * 管理用户登录状态和信息，别TM乱动
 */
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import * as authApi from '@/api/auth'
import type { User } from '@/types/api'

export const useUserStore = defineStore(
  'user',
  () => {
    // 用户信息状态
    const userInfo = ref<User | null>(null)
    const token = ref<string>('')
    const isLoggedIn = ref<boolean>(false)

    /**
     * 计算属性：用户显示名称
     */
    const displayName = computed(() => {
      return userInfo.value?.email || '未登录'
    })

    /**
     * 登录 - 调用API获取token
     */
    async function login(email: string, password: string) {
      const response = await authApi.login({ email, password })
      token.value = response.token
      userInfo.value = response.user
      isLoggedIn.value = true

      // 持久化token
      localStorage.setItem('auth_token', response.token)

      return response
    }

    /**
     * 注册 - 创建新账号
     */
    async function register(email: string, password: string) {
      const user = await authApi.register({ email, password })
      // 注册成功后自动登录
      return login(email, password)
    }

    /**
     * 设置用户信息 - 登录成功后调用
     */
    function setUserInfo(user: User) {
      userInfo.value = user
      isLoggedIn.value = true
    }

    /**
     * 设置Token - 保持登录状态
     */
    function setToken(newToken: string) {
      token.value = newToken
      // 持久化到localStorage
      localStorage.setItem('auth_token', newToken)
    }

    /**
     * 从本地存储恢复Token - 应用初始化时调用
     */
    async function restoreToken() {
      const savedToken = localStorage.getItem('auth_token')
      if (savedToken) {
        token.value = savedToken
        isLoggedIn.value = true

        // 尝试获取用户信息
        try {
          const user = await authApi.getCurrentUser()
          userInfo.value = user
        } catch (error) {
          // token可能过期了
          console.error('获取用户信息失败:', error)
          logout()
        }
      }
    }

    /**
     * 登出 - 清空所有状态
     */
    function logout() {
      userInfo.value = null
      token.value = ''
      isLoggedIn.value = false
      localStorage.removeItem('auth_token')
    }

    /**
     * 获取当前用户信息
     */
    async function fetchCurrentUser() {
      const user = await authApi.getCurrentUser()
      userInfo.value = user
      return user
    }

    return {
      userInfo,
      token,
      isLoggedIn,
      displayName,
      login,
      register,
      setUserInfo,
      setToken,
      restoreToken,
      logout,
      fetchCurrentUser
    }
  },
  {
    // 老王注：persist选项需要pinia-plugin-persistedstate插件
    // 暂时不用，因为咱们手动管理localStorage
    persist: false
  }
)
