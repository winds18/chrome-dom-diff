<!--
  Layout.vue - 主布局组件
  老王出品：包含顶部导航、侧边栏和内容区域
-->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useAppStore } from '@/stores/app'
import { useUserStore } from '@/stores/user'
import { ElMessageBox } from 'element-plus'

const router = useRouter()
const appStore = useAppStore()
const userStore = useUserStore()

// 当前激活的菜单
const activeMenu = computed(() => router.currentRoute.value.path)

/**
 * 退出登录
 */
async function handleLogout() {
  try {
    await ElMessageBox.confirm('确定要退出登录吗？', '提示', {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    })

    userStore.logout()
    router.push('/login')
  } catch {
    // 用户取消
  }
}

/**
 * 跳转到用户设置
 */
function goToSettings() {
  router.push('/settings')
}
</script>

<template>
  <div class="layout-container" :class="{ dark: appStore.theme === 'dark' }">
    <!-- 侧边栏 -->
    <aside class="layout-sidebar" :class="{ collapsed: appStore.isSidebarCollapsed }">
      <div class="sidebar-header">
        <h1 v-if="!appStore.isSidebarCollapsed" class="sidebar-title">DOM Diff</h1>
        <span v-else class="sidebar-logo">D</span>
      </div>

      <el-menu
        :default-active="activeMenu"
        :collapse="appStore.isSidebarCollapsed"
        router
        class="sidebar-menu"
      >
        <el-menu-item
          v-for="menu in appStore.sidebarMenus"
          :key="menu.path"
          :index="menu.path"
        >
          <el-icon><component :is="menu.icon" /></el-icon>
          <template #title>{{ menu.title }}</template>
        </el-menu-item>
      </el-menu>
    </aside>

    <!-- 主体区域 -->
    <div class="layout-main">
      <!-- 顶部导航栏 -->
      <header class="layout-header">
        <div class="header-left">
          <el-button
            :icon="appStore.isSidebarCollapsed ? 'Expand' : 'Fold'"
            text
            @click="appStore.toggleSidebar"
          />
          <el-breadcrumb separator="/">
            <el-breadcrumb-item :to="{ path: '/dashboard' }">首页</el-breadcrumb-item>
            <el-breadcrumb-item>{{ $route.meta.title }}</el-breadcrumb-item>
          </el-breadcrumb>
        </div>

        <div class="header-right">
          <el-button :icon="'Moon'" circle text @click="appStore.setTheme(appStore.theme === 'dark' ? 'light' : 'dark')" />

          <el-dropdown @command="handleLogout">
            <div class="user-info">
              <el-avatar :size="32" :icon="'User'" />
              <span class="user-email">{{ userStore.displayName }}</span>
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item @click="goToSettings">
                  <el-icon><icon-setting /></el-icon>
                  设置
                </el-dropdown-item>
                <el-dropdown-item divided command="logout">
                  <el-icon><icon-switch-button /></el-icon>
                  退出登录
                </el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </header>

      <!-- 内容区域 -->
      <main class="layout-content">
        <router-view v-slot="{ Component }">
          <transition name="fade" mode="out-in">
            <component :is="Component" />
          </transition>
        </router-view>
      </main>
    </div>
  </div>
</template>

<style scoped lang="scss">
.layout-container {
  display: flex;
  width: 100%;
  height: 100vh;
  background-color: var(--el-bg-color-page);
}

.layout-sidebar {
  width: 200px;
  background-color: #001529;
  transition: width 0.3s ease;
  display: flex;
  flex-direction: column;

  &.collapsed {
    width: 64px;
  }

  .sidebar-header {
    height: 60px;
    display: flex;
    align-items: center;
    justify-content: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);

    .sidebar-title {
      color: #fff;
      font-size: 18px;
      font-weight: 600;
      margin: 0;
    }

    .sidebar-logo {
      color: #fff;
      font-size: 24px;
      font-weight: 600;
    }
  }

  .sidebar-menu {
    flex: 1;
    border-right: none;
    background-color: transparent;

    :deep(.el-menu-item) {
      color: rgba(255, 255, 255, 0.65);

      &:hover {
        background-color: rgba(255, 255, 255, 0.08);
        color: #fff;
      }

      &.is-active {
        background-color: #1890ff;
        color: #fff;
      }
    }
  }
}

.layout-main {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.layout-header {
  height: 60px;
  background-color: #fff;
  border-bottom: 1px solid var(--el-border-color);
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 20px;

  .dark & {
    background-color: #141414;
  }

  .header-left {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .header-right {
    display: flex;
    align-items: center;
    gap: 16px;

    .user-info {
      display: flex;
      align-items: center;
      gap: 8px;
      cursor: pointer;
      padding: 4px 8px;
      border-radius: 4px;
      transition: background-color 0.3s;

      &:hover {
        background-color: var(--el-fill-color-light);
      }

      .user-email {
        font-size: 14px;
        color: var(--el-text-color-primary);
      }
    }
  }
}

.layout-content {
  flex: 1;
  overflow-y: auto;
  padding: 20px;
}

// 页面切换动画
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}
</style>
