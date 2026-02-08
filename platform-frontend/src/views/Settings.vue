<!--
  Settings.vue - 设置页面
  老王出品：管理系统配置和用户偏好
-->
<script setup lang="ts">
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'
import { useUserStore } from '@/stores/user'

const router = useRouter()
const appStore = useAppStore()
const userStore = useUserStore()

// 设置项
const settings = ref({
  theme: appStore.theme,
  sidebarCollapsed: appStore.isSidebarCollapsed
})

// 用户信息
const userInfo = computed(() => userStore.userInfo)

// 保存设置
const handleSave = () => {
  appStore.setTheme(settings.value.theme)
  if (settings.value.sidebarCollapsed !== appStore.isSidebarCollapsed) {
    appStore.toggleSidebar()
  }
  ElMessage.success('设置已保存')
}

// 重置设置
const handleReset = () => {
  settings.value = {
    theme: 'light',
    sidebarCollapsed: false
  }
  appStore.setTheme('light')
}

// 退出登录
async function handleLogout() {
  userStore.logout()
  router.push('/login')
}
</script>

<template>
  <div class="settings-page">
    <el-row :gutter="20">
      <el-col :span="16">
        <!-- 系统设置 -->
        <el-card shadow="hover" class="settings-card">
          <template #header>
            <span>系统设置</span>
          </template>

          <el-form label-width="120px" style="max-width: 600px">
            <el-form-item label="主题模式">
              <el-radio-group v-model="settings.theme">
                <el-radio value="light">浅色</el-radio>
                <el-radio value="dark">深色</el-radio>
                <el-radio value="auto">跟随系统</el-radio>
              </el-radio-group>
            </el-form-item>

            <el-form-item label="侧边栏">
              <el-switch
                v-model="settings.sidebarCollapsed"
                active-text="折叠"
                inactive-text="展开"
                @change="appStore.toggleSidebar"
              />
            </el-form-item>

            <el-form-item>
              <el-button type="primary" @click="handleSave">保存设置</el-button>
              <el-button @click="handleReset">重置</el-button>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>

      <el-col :span="8">
        <!-- 用户信息 -->
        <el-card shadow="hover" class="user-card">
          <template #header>
            <span>用户信息</span>
          </template>

          <div class="user-info">
            <el-avatar :size="64" :icon="'User'" />
            <div class="user-email">{{ userInfo?.email || '未登录' }}</div>
            <div class="user-role">角色: {{ userInfo?.role || 'user' }}</div>
          </div>

          <div class="user-actions">
            <el-button type="danger" :icon="'SwitchButton'" @click="handleLogout">
              退出登录
            </el-button>
          </div>
        </el-card>

        <!-- 系统信息 -->
        <el-card shadow="hover" class="info-card">
          <template #header>
            <span>关于系统</span>
          </template>

          <div class="system-info">
            <div class="info-item">
              <span class="info-label">系统名称:</span>
              <span class="info-value">Chrome DOM Diff Platform</span>
            </div>
            <div class="info-item">
              <span class="info-label">版本:</span>
              <span class="info-value">v1.0.0</span>
            </div>
            <div class="info-item">
              <span class="info-label">技术栈:</span>
              <span class="info-value">Vue 3 + TypeScript + Element Plus</span>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.settings-page {
  .settings-card,
  .user-card,
  .info-card {
    margin-bottom: 20px;
  }

  .user-info {
    text-align: center;
    padding: 20px 0;

    .user-email {
      font-size: 16px;
      font-weight: 600;
      margin-top: 12px;
      color: var(--el-text-color-primary);
    }

    .user-role {
      font-size: 14px;
      color: var(--el-text-color-secondary);
      margin-top: 4px;
    }
  }

  .user-actions {
    text-align: center;
    padding-top: 10px;
    border-top: 1px solid var(--el-border-color-lighter);
  }

  .system-info {
    .info-item {
      display: flex;
      justify-content: space-between;
      padding: 8px 0;
      border-bottom: 1px solid var(--el-border-color-lighter);

      &:last-child {
        border-bottom: none;
      }

      .info-label {
        color: var(--el-text-color-secondary);
      }

      .info-value {
        color: var(--el-text-color-primary);
        font-weight: 500;
      }
    }
  }
}
</style>
