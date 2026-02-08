<!--
  TaskDetail.vue - 任务详情页面
  老王出品：展示任务详细信息和执行历史
-->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useTaskStore } from '@/stores/task'

const route = useRoute()
const router = useRouter()
const taskStore = useTaskStore()

// 任务详情
const task = ref<any>(null)

// 加载状态
const loading = ref(false)

/**
 * 加载任务详情
 */
async function loadTask() {
  loading.value = true
  try {
    task.value = await taskStore.loadTask(route.params.id as string)
  } catch (error: any) {
    ElMessage.error(error.message || '加载任务详情失败')
  } finally {
    loading.value = false
  }
}

/**
 * 执行任务
 */
async function executeTask() {
  loading.value = true
  try {
    await taskStore.executeTask(task.value.id)
    ElMessage.success('任务已开始执行')
    await loadTask()
  } catch (error: any) {
    ElMessage.error(error.message || '执行任务失败')
  } finally {
    loading.value = false
  }
}

/**
 * 返回列表
 */
function goBack() {
  router.push('/tasks')
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): '' | 'success' | 'warning' | 'danger' | 'info' {
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
 * 获取状态文本
 */
function getStatusText(status: string): string {
  const statusMap: Record<string, string> = {
    pending: '等待中',
    running: '运行中',
    completed: '已完成',
    failed: '失败',
    cancelled: '已取消'
  }
  return statusMap[status] || status
}

onMounted(() => {
  loadTask()
})
</script>

<template>
  <div class="task-detail-page" v-loading="loading">
    <el-page-header @back="goBack" title="返回" class="page-header">
      <template #content>
        <span class="page-title">任务详情</span>
      </template>
    </el-page-header>

    <el-card v-if="task" shadow="never" class="detail-card">
      <template #header>
        <div class="card-header">
          <span>{{ task.name }}</span>
          <el-tag :type="getStatusType(task.status)">
            {{ getStatusText(task.status) }}
          </el-tag>
        </div>
      </template>

      <el-descriptions :column="2" border>
        <el-descriptions-item label="任务ID">{{ task.id }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="getStatusType(task.status)" size="small">
            {{ getStatusText(task.status) }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="任务类型" :span="2">{{ task.task_type }}</el-descriptions-item>
        <el-descriptions-item label="调度类型" :span="2">{{ task.schedule_type }}</el-descriptions-item>
        <el-descriptions-item label="描述" :span="2">{{ task.description || '-' }}</el-descriptions-item>
        <el-descriptions-item label="目标服务" :span="2">{{ task.target_service }}</el-descriptions-item>
        <el-descriptions-item label="创建时间" :span="2">{{ task.created_at }}</el-descriptions-item>
        <el-descriptions-item label="更新时间" :span="2">{{ task.updated_at }}</el-descriptions-item>
      </el-descriptions>

      <div class="actions">
        <el-button
          type="primary"
          :icon="'VideoPlay'"
          @click="executeTask"
          :disabled="task.status === 'running'"
        >
          执行任务
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.task-detail-page {
  .page-header {
    margin-bottom: 20px;
  }

  .detail-card {
    .card-header {
      display: flex;
      align-items: center;
      justify-content: space-between;
      font-size: 18px;
      font-weight: 600;
    }

    .actions {
      margin-top: 20px;
      padding-top: 20px;
      border-top: 1px solid var(--el-border-color-lighter);
    }
  }
}
</style>
