<!--
  Dashboard.vue - 仪表盘页面
  老王出品：展示系统概览和统计信息
-->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useServiceStore } from '@/stores/service'
import { useTaskStore } from '@/stores/task'
import { useLogStore } from '@/stores/log'
import type { LogLevel } from '@/types/api'

const serviceStore = useServiceStore()
const taskStore = useTaskStore()
const logStore = useLogStore()

// 统计卡片数据
const stats = ref([
  { title: '在线服务', value: '0', icon: 'Monitor', color: '#409eff', key: 'onlineServices' },
  { title: '运行任务', value: '0', icon: 'Loading', color: '#67c23a', key: 'runningTasks' },
  { title: '今日任务', value: '0', icon: 'List', color: '#e6a23c', key: 'todayTasks' },
  { title: '错误日志', value: '0', icon: 'Warning', color: '#f56c6c', key: 'errorLogs' }
])

// 最近任务列表
const recentTasks = ref<any[]>([])

// 最近日志列表
const recentLogs = ref<any[]>([])

// 加载状态
const loading = ref(false)

/**
 * 加载仪表盘数据
 */
async function loadDashboardData() {
  loading.value = true
  try {
    // 并行加载数据
    await Promise.all([
      serviceStore.loadServices().catch(() => {}),
      taskStore.loadTasks({ page: 1, page_size: 5 }).catch(() => {}),
      logStore.queryLogs({ page: 1, page_size: 10, level: 'error' as LogLevel }).catch(() => {})
    ])

    // 更新统计数据
    const onlineServices = serviceStore.services.filter(s => s.status === 'online').length
    const runningTasks = taskStore.tasks.filter(t => t.status === 'running').length

    const stat0 = stats.value[0]
    const stat1 = stats.value[1]
    const stat2 = stats.value[2]
    const stat3 = stats.value[3]

    if (stat0) stat0.value = onlineServices.toString()
    if (stat1) stat1.value = runningTasks.toString()
    if (stat2) stat2.value = taskStore.total.toString()
    if (stat3) stat3.value = logStore.total.toString()

    // 更新最近任务
    recentTasks.value = taskStore.tasks.slice(0, 5)

    // 更新最近日志
    recentLogs.value = logStore.logs.slice(0, 5)
  } catch (error) {
    console.error('加载仪表盘数据失败:', error)
  } finally {
    loading.value = false
  }
}

/**
 * 获取任务状态对应的类型
 */
function getTaskStatusType(status: string): '' | 'success' | 'warning' | 'danger' | 'info' {
  const statusMap: Record<string, '' | 'success' | 'warning' | 'danger' | 'info'> = {
    pending: 'info',
    running: 'warning',
    completed: 'success',
    failed: 'danger',
    cancelled: 'info'
  }
  return statusMap[status] || 'info'
}

/**
 * 获取任务状态文本
 */
function getTaskStatusText(status: string): string {
  const statusMap: Record<string, string> = {
    pending: '等待中',
    running: '运行中',
    completed: '已完成',
    failed: '失败',
    cancelled: '已取消'
  }
  return statusMap[status] || status
}

/**
 * 获取日志级别对应的类型
 */
function getLogLevelType(level: string): '' | 'success' | 'warning' | 'danger' | 'info' {
  const levelMap: Record<string, '' | 'success' | 'warning' | 'danger' | 'info'> = {
    debug: 'info',
    info: 'info',
    warn: 'warning',
    error: 'danger',
    fatal: 'danger'
  }
  return levelMap[level] || 'info'
}

onMounted(() => {
  loadDashboardData()
})
</script>

<template>
  <div class="dashboard-page" v-loading="loading">
    <!-- 统计卡片 -->
    <el-row :gutter="20">
      <el-col v-for="stat in stats" :key="stat.key" :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" :style="{ backgroundColor: stat.color }">
              <el-icon :size="28">
                <component :is="stat.icon" />
              </el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stat.value }}</div>
              <div class="stat-title">{{ stat.title }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 最近记录 -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>最近任务</span>
              <el-button text type="primary" @click="$router.push('/tasks')">查看全部</el-button>
            </div>
          </template>
          <el-empty v-if="recentTasks.length === 0" description="暂无任务" />
          <div v-else class="task-list">
            <div v-for="task in recentTasks" :key="task.id" class="task-item">
              <div class="task-info">
                <div class="task-name">{{ task.name }}</div>
                <div class="task-meta">
                  <el-tag :type="getTaskStatusType(task.status)" size="small">
                    {{ getTaskStatusText(task.status) }}
                  </el-tag>
                  <span class="task-time">{{ task.created_at }}</span>
                </div>
              </div>
              <el-button text type="primary" @click="$router.push(`/tasks/${task.id}`)">查看</el-button>
            </div>
          </div>
        </el-card>
      </el-col>

      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>错误日志</span>
              <el-button text type="primary" @click="$router.push('/logs')">查看全部</el-button>
            </div>
          </template>
          <el-empty v-if="recentLogs.length === 0" description="暂无日志" />
          <div v-else class="log-list">
            <div v-for="log in recentLogs" :key="log.id" class="log-item">
              <div class="log-header">
                <el-tag :type="getLogLevelType(log.level)" size="small">{{ log.level }}</el-tag>
                <span class="log-time">{{ log.timestamp }}</span>
              </div>
              <div class="log-message">{{ log.message }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 快速操作 -->
    <el-row :gutter="20" class="mt-20">
      <el-col :span="24">
        <el-card shadow="hover">
          <template #header>
            <span>快速操作</span>
          </template>
          <div class="quick-actions">
            <el-button type="primary" :icon="'Plus'" @click="$router.push('/tasks?action=create')">
              创建任务
            </el-button>
            <el-button type="success" :icon="'Monitor'" @click="$router.push('/services?action=register')">
              注册服务
            </el-button>
            <el-button type="warning" :icon="'Document'" @click="$router.push('/logs')">
              查看日志
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.dashboard-page {
  .stat-card {
    .stat-content {
      display: flex;
      align-items: center;
      gap: 16px;

      .stat-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 56px;
        height: 56px;
        border-radius: 12px;
        color: #fff;
      }

      .stat-info {
        flex: 1;

        .stat-value {
          font-size: 28px;
          font-weight: 600;
          color: var(--el-text-color-primary);
          line-height: 1;
        }

        .stat-title {
          font-size: 14px;
          color: var(--el-text-color-secondary);
          margin-top: 8px;
        }
      }
    }
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .task-list {
    .task-item {
      display: flex;
      align-items: center;
      justify-content: space-between;
      padding: 12px 0;
      border-bottom: 1px solid var(--el-border-color-lighter);

      &:last-child {
        border-bottom: none;
      }

      .task-info {
        flex: 1;

        .task-name {
          font-size: 14px;
          font-weight: 500;
          color: var(--el-text-color-primary);
          margin-bottom: 4px;
        }

        .task-meta {
          display: flex;
          align-items: center;
          gap: 8px;

          .task-time {
            font-size: 12px;
            color: var(--el-text-color-secondary);
          }
        }
      }
    }
  }

  .log-list {
    .log-item {
      padding: 12px 0;
      border-bottom: 1px solid var(--el-border-color-lighter);

      &:last-child {
        border-bottom: none;
      }

      .log-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 4px;

        .log-time {
          font-size: 12px;
          color: var(--el-text-color-secondary);
        }
      }

      .log-message {
        font-size: 14px;
        color: var(--el-text-color-regular);
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
      }
    }
  }

  .quick-actions {
    display: flex;
    gap: 12px;
  }

  .mt-20 {
    margin-top: 20px;
  }
}
</style>
