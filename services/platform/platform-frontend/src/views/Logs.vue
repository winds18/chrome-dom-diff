<!-- 艹！日志查看页面 -->
<!-- 日志列表、实时日志流 -->

<template>
  <div class="logs-page">
    <el-card>
      <template #header>
        <div class="header">
          <span>日志查看</span>
          <el-button type="primary" @click="toggleStream">
            {{ streaming ? '停止实时日志' : '开始实时日志' }}
          </el-button>
        </div>
      </template>

      <!-- 筛选器 -->
      <el-form :inline="true" class="search-form">
        <el-form-item label="日志级别">
          <el-select v-model="queryParams.level" placeholder="全部" clearable>
            <el-option label="调试" value="debug" />
            <el-option label="信息" value="info" />
            <el-option label="警告" value="warn" />
            <el-option label="错误" value="error" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchLogs">查询</el-button>
          <el-button @click="resetQuery">重置</el-button>
        </el-form-item>
      </el-form>

      <!-- 日志列表 -->
      <el-timeline v-loading="loading">
        <el-timeline-item
          v-for="log in logs"
          :key="log.id"
          :timestamp="log.createdAt"
          placement="top"
        >
          <el-card>
            <div class="log-header">
              <el-tag :type="getLevelType(log.level)" size="small">
                {{ log.level.toUpperCase() }}
              </el-tag>
              <span class="log-message">{{ log.message }}</span>
            </div>
            <div v-if="log.metadata" class="log-metadata">
              <pre>{{ JSON.stringify(log.metadata, null, 2) }}</pre>
            </div>
          </el-card>
        </el-timeline-item>
      </el-timeline>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, reactive } from 'vue'
import { ElMessage } from 'element-plus'
import { logsApi } from '@/api'
import type { Log } from '@/types/api'

const loading = ref(false)
const logs = ref<Log[]>([])
const streaming = ref(false)
let streamInterval: number | null = null

const queryParams = reactive({
  level: '' as 'debug' | 'info' | 'warn' | 'error' | '',
  limit: 50
})

const getLevelType = (level: string) => {
  const map: Record<string, any> = {
    debug: 'info',
    info: '',
    warn: 'warning',
    error: 'danger'
  }
  return map[level] || 'info'
}

const fetchLogs = async () => {
  loading.value = true
  try {
    const params: any = {
      limit: queryParams.limit
    }
    if (queryParams.level) {
      params.level = queryParams.level
    }
    const response = await logsApi.getLogs(params)
    if (response.code === 200 && response.data) {
      logs.value = response.data
    }
  } catch (error) {
    console.error('获取日志失败:', error)
  } finally {
    loading.value = false
  }
}

const resetQuery = () => {
  queryParams.level = ''
  fetchLogs()
}

const toggleStream = () => {
  streaming.value = !streaming.value
  if (streaming.value) {
    ElMessage.success('实时日志已启动')
    streamInterval = window.setInterval(async () => {
      try {
        const response = await logsApi.getLogStream()
        if (response.code === 200 && response.data) {
          const newLogs = response.data
          if (newLogs.length > 0) {
            logs.value = [...newLogs, ...logs.value].slice(0, queryParams.limit)
          }
        }
      } catch (error) {
        console.error('获取实时日志失败:', error)
      }
    }, 3000)
  } else {
    ElMessage.info('实时日志已停止')
    if (streamInterval) {
      clearInterval(streamInterval)
      streamInterval = null
    }
  }
}

onMounted(() => {
  fetchLogs()
})

onUnmounted(() => {
  if (streamInterval) {
    clearInterval(streamInterval)
  }
})
</script>

<style scoped>
.logs-page {
  padding: 20px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.search-form {
  margin-bottom: 20px;
}

.log-header {
  display: flex;
  align-items: center;
  gap: 10px;
}

.log-message {
  font-weight: bold;
}

.log-metadata {
  margin-top: 10px;
  padding: 10px;
  background-color: #f5f5f5;
  border-radius: 4px;
}

pre {
  white-space: pre-wrap;
  word-wrap: break-word;
  margin: 0;
}
</style>
