<!--
  Tasks.vue - 任务管理页面
  老王出品：管理所有任务
-->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useTaskStore } from '@/stores/task'
import type { Task, TaskStatus, TaskType } from '@/types/api'

const router = useRouter()
const taskStore = useTaskStore()

// 任务列表
const tasks = ref<Task[]>([])

// 筛选条件
const filters = ref({
  status: '',
  type: '',
  keyword: ''
})

// 分页
const pagination = ref({
  page: 1,
  pageSize: 20
})

// 加载状态
const loading = ref(false)

// 对话框状态
const showCreateDialog = ref(false)
const showExecuteDialog = ref(false)

// 创建表单
const createForm = ref({
  name: '',
  description: '',
  task_type: 'dom_capture' as TaskType,
  schedule_type: 'manual' as any,
  target_service_id: '',
  config: {} as Record<string, any>
})

// 表单引用
const createFormRef = ref()

// 表单验证规则
const rules = {
  name: [
    { required: true, message: '请输入任务名称', trigger: 'blur' }
  ],
  task_type: [
    { required: true, message: '请选择任务类型', trigger: 'change' }
  ],
  target_service_id: [
    { required: true, message: '请选择目标服务', trigger: 'change' }
  ]
}

// 过滤后的任务列表
const filteredTasks = computed(() => {
  let result = tasks.value

  if (filters.value.status) {
    result = result.filter(t => t.status === filters.value.status)
  }

  if (filters.value.type) {
    result = result.filter(t => t.task_type === filters.value.type)
  }

  if (filters.value.keyword) {
    const keyword = filters.value.keyword.toLowerCase()
    result = result.filter(t =>
      t.name.toLowerCase().includes(keyword) ||
      t.description?.toLowerCase().includes(keyword)
    )
  }

  return result
})

/**
 * 加载任务列表
 */
async function loadTasks() {
  loading.value = true
  try {
    await taskStore.loadTasks({
      page: pagination.value.page,
      page_size: pagination.value.pageSize
    })
    tasks.value = taskStore.tasks
  } catch (error: any) {
    ElMessage.error(error.message || '加载任务列表失败')
  } finally {
    loading.value = false
  }
}

/**
 * 打开创建对话框
 */
function openCreateDialog() {
  showCreateDialog.value = true
}

/**
 * 关闭对话框
 */
function closeDialog() {
  showCreateDialog.value = false
  createFormRef.value?.resetFields()
}

/**
 * 创建任务
 */
async function handleCreate() {
  const valid = await createFormRef.value?.validate().catch(() => false)
  if (!valid) return

  loading.value = true
  try {
    await taskStore.createTask(createForm.value)
    ElMessage.success('任务创建成功')
    closeDialog()
    await loadTasks()
  } catch (error: any) {
    ElMessage.error(error.message || '创建任务失败')
  } finally {
    loading.value = false
  }
}

/**
 * 执行任务
 */
async function handleExecute(task: Task) {
  try {
    await ElMessageBox.confirm(
      `确定要立即执行任务"${task.name}"吗？`,
      '提示',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    loading.value = true
    await taskStore.executeTask(task.id)
    ElMessage.success('任务已开始执行')
    await loadTasks()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '执行任务失败')
    }
  } finally {
    loading.value = false
  }
}

/**
 * 删除任务
 */
async function handleDelete(task: Task) {
  try {
    await ElMessageBox.confirm(
      `确定要删除任务"${task.name}"吗？`,
      '提示',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    loading.value = true
    await taskStore.deleteTask(task.id)
    ElMessage.success('任务已删除')
    await loadTasks()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除任务失败')
    }
  } finally {
    loading.value = false
  }
}

/**
 * 查看任务详情
 */
function viewDetail(task: Task) {
  router.push(`/tasks/${task.id}`)
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

/**
 * 获取任务类型文本
 */
function getTaskTypeText(type: string): string {
  const typeMap: Record<string, string> = {
    dom_capture: 'DOM捕获',
    dom_diff: 'DOM对比',
    data_extraction: '数据提取',
    custom: '自定义'
  }
  return typeMap[type] || type
}

onMounted(() => {
  loadTasks()
})
</script>

<template>
  <div class="tasks-page" v-loading="loading">
    <!-- 工具栏 -->
    <el-card shadow="never" class="toolbar-card">
      <div class="toolbar">
        <div class="filters">
          <el-input
            v-model="filters.keyword"
            placeholder="搜索任务名称或描述"
            :prefix-icon="'Search'"
            style="width: 280px"
            clearable
          />
          <el-select v-model="filters.status" placeholder="筛选状态" style="width: 120px" clearable>
            <el-option label="等待中" value="pending" />
            <el-option label="运行中" value="running" />
            <el-option label="已完成" value="completed" />
            <el-option label="失败" value="failed" />
          </el-select>
          <el-select v-model="filters.type" placeholder="筛选类型" style="width: 140px" clearable>
            <el-option label="DOM捕获" value="dom_capture" />
            <el-option label="DOM对比" value="dom_diff" />
            <el-option label="数据提取" value="data_extraction" />
          </el-select>
        </div>
        <el-button type="primary" :icon="'Plus'" @click="openCreateDialog">
          创建任务
        </el-button>
      </div>
    </el-card>

    <!-- 任务表格 -->
    <el-card shadow="never">
      <el-table :data="filteredTasks" style="width: 100%">
        <el-table-column prop="name" label="任务名称" min-width="180" />
        <el-table-column prop="description" label="描述" min-width="200" show-overflow-tooltip />
        <el-table-column label="类型" width="110">
          <template #default="{ row }">
            <el-tag size="small">{{ getTaskTypeText(row.task_type) }}</el-tag>
          </template>
        </el-table-column>
        <el-table-column label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)" size="small">
              {{ getStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="180" />
        <el-table-column label="操作" width="240" fixed="right">
          <template #default="{ row }">
            <el-button text type="primary" size="small" @click="viewDetail(row)">
              详情
            </el-button>
            <el-button
              text
              type="success"
              size="small"
              @click="handleExecute(row)"
              :disabled="row.status === 'running'"
            >
              执行
            </el-button>
            <el-button text type="danger" size="small" @click="handleDelete(row)">
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <div class="pagination">
        <el-pagination
          v-model:current-page="pagination.page"
          v-model:page-size="pagination.pageSize"
          :total="taskStore.total"
          :page-sizes="[10, 20, 50, 100]"
          layout="total, sizes, prev, pager, next, jumper"
          @size-change="loadTasks"
          @current-change="loadTasks"
        />
      </div>
    </el-card>

    <!-- 创建对话框 -->
    <el-dialog
      v-model="showCreateDialog"
      title="创建任务"
      width="600px"
      @close="closeDialog"
    >
      <el-form
        ref="createFormRef"
        :model="createForm"
        :rules="rules"
        label-width="100px"
      >
        <el-form-item label="任务名称" prop="name">
          <el-input v-model="createForm.name" placeholder="请输入任务名称" />
        </el-form-item>
        <el-form-item label="任务描述">
          <el-input v-model="createForm.description" type="textarea" placeholder="请输入任务描述" />
        </el-form-item>
        <el-form-item label="任务类型" prop="task_type">
          <el-select v-model="createForm.task_type" style="width: 100%">
            <el-option label="DOM捕获" value="dom_capture" />
            <el-option label="DOM对比" value="dom_diff" />
            <el-option label="数据提取" value="data_extraction" />
            <el-option label="自定义" value="custom" />
          </el-select>
        </el-form-item>
        <el-form-item label="调度类型">
          <el-select v-model="createForm.schedule_type" style="width: 100%">
            <el-option label="手动执行" value="manual" />
            <el-option label="定时执行" value="cron" />
            <el-option label="间隔执行" value="interval" />
          </el-select>
        </el-form-item>
        <el-form-item label="目标服务" prop="target_service_id">
          <el-select v-model="createForm.target_service_id" placeholder="请选择目标服务" style="width: 100%">
            <!-- 老王注：服务列表需要从serviceStore获取 -->
            <el-option
              v-for="service in ([] as any[])"
              :key="service.id"
              :label="service.name"
              :value="service.id"
            />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="closeDialog">取消</el-button>
        <el-button type="primary" @click="handleCreate">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.tasks-page {
  .toolbar-card {
    margin-bottom: 20px;
  }

  .toolbar {
    display: flex;
    align-items: center;
    justify-content: space-between;

    .filters {
      display: flex;
      gap: 12px;
    }
  }

  .pagination {
    display: flex;
    justify-content: flex-end;
    margin-top: 20px;
  }
}
</style>
