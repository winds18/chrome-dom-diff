<!-- 艹！任务管理页面 -->
<!-- 任务列表、创建任务、操作任务 -->

<template>
  <div class="tasks-page">
    <el-card>
      <template #header>
        <div class="header">
          <span>任务管理</span>
          <el-button type="primary" @click="showCreateDialog = true">
            创建任务
          </el-button>
        </div>
      </template>

      <!-- 搜索筛选 -->
      <el-form :inline="true" class="search-form">
        <el-form-item label="状态">
          <el-select v-model="queryParams.status" placeholder="全部" clearable>
            <el-option label="等待中" value="pending" />
            <el-option label="运行中" value="running" />
            <el-option label="已完成" value="completed" />
            <el-option label="失败" value="failed" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="fetchTasks">查询</el-button>
          <el-button @click="resetQuery">重置</el-button>
        </el-form-item>
      </el-form>

      <!-- 任务列表 -->
      <el-table :data="tasks" v-loading="loading" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="serviceId" label="服务ID" width="100" />
        <el-table-column prop="action" label="操作" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)">
              {{ getStatusText(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="createdAt" label="创建时间" />
        <el-table-column label="操作" width="200">
          <template #default="{ row }">
            <el-button size="small" @click="viewTask(row)">查看</el-button>
            <el-button
              v-if="row.status === 'pending' || row.status === 'failed'"
              size="small"
              type="primary"
              @click="executeTask(row.id)"
            >
              执行
            </el-button>
            <el-button
              size="small"
              type="danger"
              @click="deleteTask(row.id)"
            >
              删除
            </el-button>
          </template>
        </el-table-column>
      </el-table>

      <!-- 分页 -->
      <el-pagination
        v-model:current-page="queryParams.page"
        v-model:page-size="queryParams.pageSize"
        :total="total"
        @current-change="fetchTasks"
        @size-change="fetchTasks"
        style="margin-top: 20px; justify-content: center"
      />
    </el-card>

    <!-- 创建任务对话框 -->
    <el-dialog v-model="showCreateDialog" title="创建任务" width="500px">
      <el-form :model="taskForm" label-width="100px">
        <el-form-item label="服务ID">
          <el-input v-model.number="taskForm.serviceId" type="number" />
        </el-form-item>
        <el-form-item label="操作">
          <el-input v-model="taskForm.action" />
        </el-form-item>
        <el-form-item label="负载参数">
          <el-input v-model="taskForm.payload" type="textarea" :rows="3" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCreateDialog = false">取消</el-button>
        <el-button type="primary" @click="handleCreateTask">确定</el-button>
      </template>
    </el-dialog>

    <!-- 查看任务对话框 -->
    <el-dialog v-model="showViewDialog" title="任务详情" width="600px">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="ID">{{ currentTask.id }}</el-descriptions-item>
        <el-descriptions-item label="服务ID">{{ currentTask.serviceId }}</el-descriptions-item>
        <el-descriptions-item label="操作">{{ currentTask.action }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="getStatusType(currentTask.status)">
            {{ getStatusText(currentTask.status) }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="结果">
          <pre>{{ JSON.stringify(currentTask.result, null, 2) }}</pre>
        </el-descriptions-item>
        <el-descriptions-item label="创建时间">{{ currentTask.createdAt }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { tasksApi } from '@/api'
import type { Task, TaskCreateRequest } from '@/types/api'

const loading = ref(false)
const tasks = ref<Task[]>([])
const total = ref(0)
const showCreateDialog = ref(false)
const showViewDialog = ref(false)
const currentTask = ref<Task>({} as Task)

const queryParams = reactive({
  page: 1,
  pageSize: 10,
  status: ''
})

const taskForm = reactive<TaskCreateRequest>({
  serviceId: 0,
  action: '',
  payload: ''
})

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

const fetchTasks = async () => {
  loading.value = true
  try {
    const response = await tasksApi.getTasks(queryParams)
    if (response.code === 200 && response.data) {
      tasks.value = response.data.items
      total.value = response.data.total
    }
  } catch (error) {
    console.error('获取任务列表失败:', error)
  } finally {
    loading.value = false
  }
}

const resetQuery = () => {
  queryParams.status = ''
  queryParams.page = 1
  fetchTasks()
}

const handleCreateTask = async () => {
  try {
    const response = await tasksApi.createTask(taskForm)
    if (response.code === 200) {
      ElMessage.success('创建任务成功')
      showCreateDialog.value = false
      fetchTasks()
    }
  } catch (error) {
    ElMessage.error('创建任务失败')
  }
}

const executeTask = async (id: number) => {
  try {
    const response = await tasksApi.executeTask(id)
    if (response.code === 200) {
      ElMessage.success('任务已开始执行')
      fetchTasks()
    }
  } catch (error) {
    ElMessage.error('执行任务失败')
  }
}

const deleteTask = async (id: number) => {
  try {
    await ElMessageBox.confirm('确定要删除这个任务吗？', '提示', {
      type: 'warning'
    })
    const response = await tasksApi.deleteTask(id)
    if (response.code === 200) {
      ElMessage.success('删除成功')
      fetchTasks()
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
    }
  }
}

const viewTask = (task: Task) => {
  currentTask.value = task
  showViewDialog.value = true
}

onMounted(() => {
  fetchTasks()
})
</script>

<style scoped>
.tasks-page {
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

pre {
  white-space: pre-wrap;
  word-wrap: break-word;
}
</style>
