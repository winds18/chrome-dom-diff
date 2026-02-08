<!--
  Logs.vue - 日志管理页面
  老王出品：查询和查看系统日志
-->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useLogStore } from '@/stores/log'
import type { LogLevel, LogQuery } from '@/types/api'

const logStore = useLogStore()

// 日志列表
const logs = ref<any[]>([])

// 筛选条件
const filters = ref({
  level: '',
  source: '',
  message: '',
  service_id: '',
  task_id: '',
  start_time: '',
  end_time: ''
})

// 分页
const pagination = ref({
  page: 1,
  pageSize: 50
})

// 加载状态
const loading = ref(false)

// 日志级别选项
const levelOptions = [
  { label: '调试', value: 'debug' },
  { label: '信息', value: 'info' },
  { label: '警告', value: 'warn' },
  { label: '错误', value: 'error' },
  { label: '致命', value: 'fatal' }
]

// 显示详情对话框
const showDetailDialog = ref(false)
const selectedLog = ref<any>(null)

/**
 * 查询日志
 */
async function queryLogs() {
  loading.value = true
  try {
    const query: LogQuery = {
      page: pagination.value.page,
      page_size: pagination.value.pageSize
    }
    if (filters.value.level) query.level = filters.value.level as LogLevel
    if (filters.value.source) query.source = filters.value.source
    if (filters.value.message) query.message = filters.value.message
    if (filters.value.service_id) query.service_id = filters.value.service_id
    if (filters.value.task_id) query.task_id = filters.value.task_id

    await logStore.queryLogs(query)
    logs.value = logStore.logs
  } catch (error: any) {
    ElMessage.error(error.message || '查询日志失败')
  } finally {
    loading.value = false
  }
}

/**
 * 刷新日志
 */
async function refreshLogs() {
  await queryLogs()
}

/**
 * 清空筛选条件
 */
function clearFilters() {
  filters.value = {
    level: '',
    source: '',
    message: '',
    service_id: '',
    task_id: '',
    start_time: '',
    end_time: ''
  }
  pagination.value.page = 1
  queryLogs()
}

/**
 * 显示日志详情
 */
function showLogDetail(log: any) {
  selectedLog.value = log
  showDetailDialog.value = true
}

/**
 * 获取日志级别类型
 */
function getLevelType(level: string): '' | 'success' | 'warning' | 'danger' | 'info' {
  const levelMap: Record<string, '' | 'success' | 'warning' | 'danger' | 'info'> = {
    debug: 'info',
    info: 'info',
    warn: 'warning',
    error: 'danger',
    fatal: 'danger'
  }
  return levelMap[level] || 'info'
}

/**
 * 格式化时间戳
 */
function formatTime(timestamp: string): string {
  if (!timestamp) return ''
  const date = new Date(timestamp)
  return date.toLocaleString('zh-CN')
}

onMounted(() => {
  queryLogs()
})
</script>

<template>
  <div class="logs-page" v-loading="loading">
    <!-- 筛选工具栏 -->
    <el-card shadow="never" class="filter-card">
      <el-form :model="filters" inline>
        <el-form-item label="日志级别">
          <el-select v-model="filters.level" placeholder="全部" clearable style="width: 120px">
            <el-option
              v-for="option in levelOptions"
              :key="option.value"
              :label="option.label"
              :value="option.value"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="来源">
          <el-input v-model="filters.source" placeholder="输入来源" clearable style="width: 160px" />
        </el-form-item>
        <el-form-item label="关键词">
          <el-input v-model="filters.message" placeholder="搜索关键词" clearable style="width: 200px" />
        </el-form-item>
        <el-form-item label="服务ID">
          <el-input v-model="filters.service_id" placeholder="服务ID" clearable style="width: 180px" />
        </el-form-item>
        <el-form-item>
          <el-button type="primary" :icon="'Search'" @click="queryLogs">查询</el-button>
          <el-button :icon="'Refresh'" @click="refreshLogs">刷新</el-button>
          <el-button :icon="'Delete'" @click="clearFilters">清空</el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 日志列表 -->
    <el-card shadow="never" class="log-list-card">
      <div v-if="logs.length === 0" class="empty-state">
        <el-empty description="暂无日志" />
      </div>
      <div v-else class="log-list">
        <div
          v-for="log in logs"
          :key="log.id"
          class="log-item"
          @click="showLogDetail(log)"
        >
          <div class="log-header">
            <el-tag :type="getLevelType(log.level)" size="small">
              {{ log.level.toUpperCase() }}
            </el-tag>
            <span class="log-source" v-if="log.source">{{ log.source }}</span>
            <span class="log-time">{{ formatTime(log.timestamp) }}</span>
          </div>
          <div class="log-message">{{ log.message }}</div>
          <div class="log-meta" v-if="log.service_id || log.task_id">
            <span v-if="log.service_id" class="meta-item">服务: {{ log.service_id.slice(0, 8) }}...</span>
            <span v-if="log.task_id" class="meta-item">任务: {{ log.task_id.slice(0, 8) }}...</span>
          </div>
        </div>
      </div>

      <!-- 分页 -->
      <div class="pagination" v-if="logs.length > 0">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :total="logStore.total"
          :page-sizes="[20, 50, 100, 200]"
          layout="total, sizes, prev, pager, next"
          @size-change="queryLogs"
          @current-change="queryLogs"
        />
      </div>
    </el-card>

    <!-- 详情对话框 -->
    <el-dialog
      v-model="showDetailDialog"
      title="日志详情"
      width="600px"
    >
      <div v-if="selectedLog" class="log-detail">
        <el-descriptions :column="1" border>
          <el-descriptions-item label="时间">
            {{ formatTime(selectedLog.timestamp) }}
          </el-descriptions-item>
          <el-descriptions-item label="级别">
            <el-tag :type="getLevelType(selectedLog.level)" size="small">
              {{ selectedLog.level.toUpperCase() }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="来源" v-if="selectedLog.source">
            {{ selectedLog.source }}
          </el-descriptions-item>
          <el-descriptions-item label="服务ID" v-if="selectedLog.service_id">
            {{ selectedLog.service_id }}
          </el-descriptions-item>
          <el-descriptions-item label="任务ID" v-if="selectedLog.task_id">
            {{ selectedLog.task_id }}
          </el-descriptions-item>
          <el-descriptions-item label="消息">
            <pre class="log-message-detail">{{ selectedLog.message }}</pre>
          </el-descriptions-item>
          <el-descriptions-item label="元数据" v-if="selectedLog.metadata">
            <pre class="log-metadata-detail">{{ JSON.stringify(selectedLog.metadata, null, 2) }}</pre>
          </el-descriptions-item>
        </el-descriptions>
      </div>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.logs-page {
  .filter-card {
    margin-bottom: 20px;
  }

  .log-list-card {
    min-height: 400px;
  }

  .log-list {
    .log-item {
      padding: 12px;
      border-bottom: 1px solid var(--el-border-color-lighter);
      cursor: pointer;
      transition: background-color 0.2s;

      &:hover {
        background-color: var(--el-fill-color-light);
      }

      &:last-child {
        border-bottom: none;
      }

      .log-header {
        display: flex;
        align-items: center;
        gap: 12px;
        margin-bottom: 8px;

        .log-source {
          font-size: 13px;
          color: var(--el-text-color-secondary);
        }

        .log-time {
          margin-left: auto;
          font-size: 12px;
          color: var(--el-text-color-secondary);
        }
      }

      .log-message {
        font-size: 14px;
        color: var(--el-text-color-primary);
        margin-bottom: 8px;
        font-family: 'Courier New', monospace;
      }

      .log-meta {
        display: flex;
        gap: 16px;

        .meta-item {
          font-size: 12px;
          color: var(--el-text-color-secondary);
        }
      }
    }
  }

  .pagination {
    display: flex;
    justify-content: flex-end;
    margin-top: 20px;
  }

  .log-detail {
    .log-message-detail,
    .log-metadata-detail {
      margin: 0;
      white-space: pre-wrap;
      word-break: break-all;
      font-family: 'Courier New', monospace;
      font-size: 13px;
      background-color: var(--el-fill-color-light);
      padding: 8px;
      border-radius: 4px;
    }
  }
}
</style>
