<!-- 艹！仪表盘页面 -->
<!-- 统计卡片 + 最近任务列表 -->

<template>
  <div class="dashboard">
    <el-row :gutter="20" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <el-statistic title="服务总数" :value="stats.totalServices">
            <template #prefix>
              <el-icon><Connection /></el-icon>
            </template>
          </el-statistic>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <el-statistic title="任务总数" :value="stats.totalTasks">
            <template #prefix>
              <el-icon><List /></el-icon>
            </template>
          </el-statistic>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <el-statistic title="运行中任务" :value="stats.runningTasks">
            <template #prefix>
              <el-icon><Loading /></el-icon>
            </template>
          </el-statistic>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <el-statistic title="在线服务" :value="stats.onlineServices">
            <template #prefix>
              <el-icon><CircleCheck /></el-icon>
            </template>
          </el-statistic>
        </el-card>
      </el-col>
    </el-row>

    <el-card class="recent-tasks">
      <template #header>
        <span>最近任务</span>
      </template>
      <el-table :data="recentTasks" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="action" label="操作" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ getStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="createdAt" label="创建时间" />
      </el-table>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { servicesApi, tasksApi } from '@/api'
import { Connection, List, Loading, CircleCheck } from '@element-plus/icons-vue'
import type { Task } from '@/types/api'

const stats = ref({
  totalServices: 0,
  totalTasks: 0,
  runningTasks: 0,
  onlineServices: 0
})

const recentTasks = ref<Task[]>([])

const getStatusType = (status: string) => {
  const map: Record<string, any> = {
    pending: 'info',
    running: 'warning',
    completed: 'success',
    failed: 'danger'
  }
  return map[status] || 'info'
}

const getStatusText = (status: string) => {
  const map: Record<string, string> = {
    pending: '等待中',
    running: '运行中',
    completed: '已完成',
    failed: '失败'
  }
  return map[status] || status
}

const fetchStats = async () => {
  try {
    const [servicesRes, tasksRes] = await Promise.all([
      servicesApi.getServices(),
      tasksApi.getTasks({ page: 1, pageSize: 10 })
    ])

    if (servicesRes.code === 200 && servicesRes.data) {
      stats.value.totalServices = servicesRes.data.total
      stats.value.onlineServices = servicesRes.data.items.filter(s => s.status === 'online').length
    }

    if (tasksRes.code === 200 && tasksRes.data) {
      stats.value.totalTasks = tasksRes.data.total
      stats.value.runningTasks = tasksRes.data.items.filter(t => t.status === 'running').length
      recentTasks.value = tasksRes.data.items.slice(0, 5)
    }
  } catch (error) {
    console.error('获取统计数据失败:', error)
  }
}

onMounted(() => {
  fetchStats()
})
</script>

<style scoped>
.dashboard {
  padding: 20px;
}

.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  text-align: center;
}

.recent-tasks {
  margin-top: 20px;
}
</style>
