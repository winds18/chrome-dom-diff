<!--
  Services.vue - 服务管理页面
  老王出品：管理所有注册的服务
-->
<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useServiceStore } from '@/stores/service'
import type { Service } from '@/types/api'

const router = useRouter()
const serviceStore = useServiceStore()

// 服务列表
const services = ref<Service[]>([])

// 筛选条件
const filters = ref({
  status: '',
  keyword: ''
})

// 加载状态
const loading = ref(false)

// 对话框状态
const showRegisterDialog = ref(false)

// 注册表单
const registerForm = ref({
  name: '',
  description: '',
  type: 'forwarder',
  version: '1.0.0',
  ip_address: '',
  port: 8080,
  tags: [] as string[]
})

// 表单引用
const registerFormRef = ref()

// 表单验证规则
const rules = {
  name: [
    { required: true, message: '请输入服务名称', trigger: 'blur' }
  ],
  type: [
    { required: true, message: '请选择服务类型', trigger: 'change' }
  ]
}

// 过滤后的服务列表
const filteredServices = computed(() => {
  let result = services.value

  if (filters.value.status) {
    result = result.filter(s => s.status === filters.value.status)
  }

  if (filters.value.keyword) {
    const keyword = filters.value.keyword.toLowerCase()
    result = result.filter(s =>
      s.name.toLowerCase().includes(keyword) ||
      s.description?.toLowerCase().includes(keyword)
    )
  }

  return result
})

/**
 * 加载服务列表
 */
async function loadServices() {
  loading.value = true
  try {
    await serviceStore.loadServices()
    services.value = serviceStore.services
  } catch (error: any) {
    ElMessage.error(error.message || '加载服务列表失败')
  } finally {
    loading.value = false
  }
}

/**
 * 打开注册对话框
 */
function openRegisterDialog() {
  showRegisterDialog.value = true
}

/**
 * 关闭对话框
 */
function closeDialog() {
  showRegisterDialog.value = false
  registerFormRef.value?.resetFields()
}

/**
 * 注册服务
 */
async function handleRegister() {
  const valid = await registerFormRef.value?.validate().catch(() => false)
  if (!valid) return

  loading.value = true
  try {
    await serviceStore.registerService(registerForm.value)
    ElMessage.success('服务注册成功')
    closeDialog()
    await loadServices()
  } catch (error: any) {
    ElMessage.error(error.message || '服务注册失败')
  } finally {
    loading.value = false
  }
}

/**
 * 删除服务
 */
async function handleDelete(service: Service) {
  try {
    await ElMessageBox.confirm(
      `确定要删除服务"${service.name}"吗？`,
      '提示',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    )

    loading.value = true
    await serviceStore.deleteService(service.id)
    ElMessage.success('服务已删除')
    await loadServices()
  } catch (error: any) {
    if (error !== 'cancel') {
      ElMessage.error(error.message || '删除服务失败')
    }
  } finally {
    loading.value = false
  }
}

/**
 * 查看服务详情
 */
function viewDetail(service: Service) {
  router.push(`/services/${service.id}`)
}

/**
 * 获取状态类型
 */
function getStatusType(status: string): '' | 'success' | 'warning' | 'danger' | 'info' {
  const statusMap: Record<string, '' | 'success' | 'warning' | 'danger' | 'info'> = {
    online: 'success',
    offline: 'info',
    error: 'danger'
  }
  return statusMap[status] || 'info'
}

/**
 * 获取状态文本
 */
function getStatusText(status: string): string {
  const statusMap: Record<string, string> = {
    online: '在线',
    offline: '离线',
    error: '错误'
  }
  return statusMap[status] || status
}

onMounted(() => {
  loadServices()
})
</script>

<template>
  <div class="services-page" v-loading="loading">
    <!-- 工具栏 -->
    <el-card shadow="never" class="toolbar-card">
      <div class="toolbar">
        <div class="filters">
          <el-input
            v-model="filters.keyword"
            placeholder="搜索服务名称或描述"
            :prefix-icon="'Search'"
            style="width: 280px"
            clearable
          />
          <el-select v-model="filters.status" placeholder="筛选状态" style="width: 140px" clearable>
            <el-option label="在线" value="online" />
            <el-option label="离线" value="offline" />
            <el-option label="错误" value="error" />
          </el-select>
        </div>
        <el-button type="primary" :icon="'Plus'" @click="openRegisterDialog">
          注册服务
        </el-button>
      </div>
    </el-card>

    <!-- 服务列表 -->
    <el-row :gutter="20" class="service-list">
      <el-col v-for="service in filteredServices" :key="service.id" :span="8">
        <el-card shadow="hover" class="service-card">
          <div class="service-header">
            <div class="service-name">{{ service.name }}</div>
            <el-tag :type="getStatusType(service.status)" size="small">
              {{ getStatusText(service.status) }}
            </el-tag>
          </div>
          <div class="service-description" v-if="service.description">
            {{ service.description }}
          </div>
          <div class="service-meta">
            <div class="meta-item" v-if="service.version">
              <span class="meta-label">版本:</span>
              <span class="meta-value">{{ service.version }}</span>
            </div>
            <div class="meta-item" v-if="service.ip_address">
              <span class="meta-label">地址:</span>
              <span class="meta-value">{{ service.ip_address }}:{{ service.port }}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">心跳:</span>
              <span class="meta-value">{{ service.last_heartbeat || '从未' }}</span>
            </div>
          </div>
          <div class="service-actions">
            <el-button text type="primary" size="small" @click="viewDetail(service)">
              查看详情
            </el-button>
            <el-button text type="danger" size="small" @click="handleDelete(service)">
              删除
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 空状态 -->
    <el-empty v-if="filteredServices.length === 0" description="暂无服务" />

    <!-- 注册对话框 -->
    <el-dialog
      v-model="showRegisterDialog"
      title="注册服务"
      width="500px"
      @close="closeDialog"
    >
      <el-form
        ref="registerFormRef"
        :model="registerForm"
        :rules="rules"
        label-width="100px"
      >
        <el-form-item label="服务名称" prop="name">
          <el-input v-model="registerForm.name" placeholder="请输入服务名称" />
        </el-form-item>
        <el-form-item label="服务描述">
          <el-input v-model="registerForm.description" type="textarea" placeholder="请输入服务描述" />
        </el-form-item>
        <el-form-item label="服务类型" prop="type">
          <el-select v-model="registerForm.type" style="width: 100%">
            <el-option label="转发服务" value="forwarder" />
            <el-option label="采集服务" value="collector" />
            <el-option label="分析服务" value="analyzer" />
          </el-select>
        </el-form-item>
        <el-form-item label="版本号">
          <el-input v-model="registerForm.version" placeholder="1.0.0" />
        </el-form-item>
        <el-form-item label="IP地址">
          <el-input v-model="registerForm.ip_address" placeholder="127.0.0.1" />
        </el-form-item>
        <el-form-item label="端口">
          <el-input-number v-model="registerForm.port" :min="1" :max="65535" style="width: 100%" />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="closeDialog">取消</el-button>
        <el-button type="primary" @click="handleRegister">确定</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped lang="scss">
.services-page {
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

  .service-list {
    .service-card {
      margin-bottom: 20px;

      .service-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        margin-bottom: 12px;

        .service-name {
          font-size: 16px;
          font-weight: 600;
          color: var(--el-text-color-primary);
        }
      }

      .service-description {
        font-size: 14px;
        color: var(--el-text-color-secondary);
        margin-bottom: 12px;
        min-height: 40px;
      }

      .service-meta {
        margin-bottom: 12px;

        .meta-item {
          display: flex;
          font-size: 13px;
          margin-bottom: 4px;

          .meta-label {
            color: var(--el-text-color-secondary);
            width: 60px;
          }

          .meta-value {
            color: var(--el-text-color-regular);
          }
        }
      }

      .service-actions {
        display: flex;
        gap: 8px;
        padding-top: 12px;
        border-top: 1px solid var(--el-border-color-lighter);
      }
    }
  }
}
</style>
