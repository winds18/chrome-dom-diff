<!--
  ServiceDetail.vue - 服务详情页面
  老王出品：展示服务详细信息和控制
-->
<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useServiceStore } from '@/stores/service'

const route = useRoute()
const router = useRouter()
const serviceStore = useServiceStore()

// 服务详情
const service = ref<any>(null)

// 加载状态
const loading = ref(false)

// 命令表单
const commandForm = ref({
  command: '',
  params: {}
})

// 显示命令对话框
const showCommandDialog = ref(false)

/**
 * 加载服务详情
 */
async function loadService() {
  loading.value = true
  try {
    service.value = await serviceStore.loadService(route.params.id as string)
  } catch (error: any) {
    ElMessage.error(error.message || '加载服务详情失败')
  } finally {
    loading.value = false
  }
}

/**
 * 发送命令
 */
async function sendCommand() {
  loading.value = true
  try {
    await serviceStore.sendCommand(service.value.id, commandForm.value.command, commandForm.value.params)
    ElMessage.success('命令已发送')
    showCommandDialog.value = false
    await loadService()
  } catch (error: any) {
    ElMessage.error(error.message || '发送命令失败')
  } finally {
    loading.value = false
  }
}

/**
 * 返回列表
 */
function goBack() {
  router.push('/services')
}

onMounted(() => {
  loadService()
})
</script>

<template>
  <div class="service-detail-page" v-loading="loading">
    <el-page-header @back="goBack" title="返回" class="page-header">
      <template #content>
        <span class="page-title">服务详情</span>
      </template>
    </el-page-header>

    <el-card v-if="service" shadow="never" class="detail-card">
      <template #header>
        <div class="card-header">
          <span>{{ service.name }}</span>
          <el-tag :type="service.status === 'online' ? 'success' : service.status === 'error' ? 'danger' : 'info'">
            {{ service.status === 'online' ? '在线' : service.status === 'error' ? '错误' : '离线' }}
          </el-tag>
        </div>
      </template>

      <el-descriptions :column="2" border>
        <el-descriptions-item label="服务ID">{{ service.id }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="service.status === 'online' ? 'success' : service.status === 'error' ? 'danger' : 'info'" size="small">
            {{ service.status }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="描述" :span="2">{{ service.description || '-' }}</el-descriptions-item>
        <el-descriptions-item label="版本">{{ service.version || '-' }}</el-descriptions-item>
        <el-descriptions-item label="心跳时间">{{ service.last_heartbeat || '从未' }}</el-descriptions-item>
        <el-descriptions-item label="IP地址" :span="2">
          {{ service.ip_address || '-' }}:{{ service.port || '-' }}
        </el-descriptions-item>
        <el-descriptions-item label="创建时间" :span="2">{{ service.created_at }}</el-descriptions-item>
      </el-descriptions>

      <div class="actions" v-if="service.status === 'online'">
        <el-button type="primary" :icon="'Setting'" @click="showCommandDialog = true">
          发送命令
        </el-button>
      </div>
    </el-card>
  </div>

  <!-- 命令对话框 -->
  <el-dialog v-model="showCommandDialog" title="发送命令" width="500px">
    <el-form :model="commandForm" label-width="100px">
      <el-form-item label="命令类型">
        <el-select v-model="commandForm.command" placeholder="请选择命令">
          <el-option label="获取状态" value="status" />
          <el-option label="重启" value="restart" />
          <el-option label="停止" value="stop" />
          <el-option label="配置" value="config" />
        </el-select>
      </el-form-item>
    </el-form>
    <template #footer>
      <el-button @click="showCommandDialog = false">取消</el-button>
      <el-button type="primary" @click="sendCommand">确定</el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.service-detail-page {
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
