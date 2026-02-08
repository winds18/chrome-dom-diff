<!-- 艹！服务管理页面 -->
<!-- 服务列表、查看服务详情、发送命令 -->

<template>
  <div class="services-page">
    <el-card>
      <template #header>
        <span>服务管理</span>
      </template>

      <!-- 服务列表 -->
      <el-table :data="services" v-loading="loading" style="width: 100%">
        <el-table-column prop="id" label="ID" width="80" />
        <el-table-column prop="pluginId" label="插件ID" width="200" />
        <el-table-column prop="url" label="URL" />
        <el-table-column prop="title" label="页面标题" />
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="row.status === 'online' ? 'success' : 'danger'">
              {{ row.status === 'online' ? '在线' : '离线' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="lastSeen" label="最后活跃" />
        <el-table-column label="操作" width="200">
          <template #default="{ row }">
            <el-button size="small" @click="viewService(row)">查看</el-button>
            <el-button
              size="small"
              type="primary"
              @click="showCommandDialog(row)"
            >
              发送命令
            </el-button>
            <el-button
              size="small"
              type="danger"
              @click="deleteService(row.id)"
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
        @current-change="fetchServices"
        @size-change="fetchServices"
        style="margin-top: 20px; justify-content: center"
      />
    </el-card>

    <!-- 查看服务详情对话框 -->
    <el-dialog v-model="showViewDialog" title="服务详情" width="600px">
      <el-descriptions :column="1" border>
        <el-descriptions-item label="ID">{{ currentService.id }}</el-descriptions-item>
        <el-descriptions-item label="插件ID">{{ currentService.pluginId }}</el-descriptions-item>
        <el-descriptions-item label="Tab ID">{{ currentService.tabId }}</el-descriptions-item>
        <el-descriptions-item label="URL">{{ currentService.url }}</el-descriptions-item>
        <el-descriptions-item label="标题">{{ currentService.title }}</el-descriptions-item>
        <el-descriptions-item label="状态">
          <el-tag :type="currentService.status === 'online' ? 'success' : 'danger'">
            {{ currentService.status === 'online' ? '在线' : '离线' }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="能力列表">
          <el-tag v-for="cap in currentService.capabilities" :key="cap" style="margin: 2px">
            {{ cap }}
          </el-tag>
        </el-descriptions-item>
        <el-descriptions-item label="最后活跃">{{ currentService.lastSeen }}</el-descriptions-item>
        <el-descriptions-item label="注册时间">{{ currentService.createdAt }}</el-descriptions-item>
      </el-descriptions>
    </el-dialog>

    <!-- 发送命令对话框 -->
    <el-dialog v-model="showCommandDialogFlag" title="发送命令" width="500px">
      <el-form :model="commandForm" label-width="100px">
        <el-form-item label="操作">
          <el-input v-model="commandForm.action" placeholder="例如: dom_capture" />
        </el-form-item>
        <el-form-item label="参数">
          <el-input v-model="commandForm.payload" type="textarea" :rows="3" placeholder='JSON格式，例如: {"xpath": "//div"}' />
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="showCommandDialogFlag = false">取消</el-button>
        <el-button type="primary" @click="handleSendCommand">发送</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, reactive } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { servicesApi } from '@/api'
import type { Service, ServiceCommand } from '@/types/api'

const loading = ref(false)
const services = ref<Service[]>([])
const total = ref(0)
const showViewDialog = ref(false)
const showCommandDialogFlag = ref(false)
const currentService = ref<Service>({} as Service)

const queryParams = reactive({
  page: 1,
  pageSize: 10
})

const commandForm = reactive<ServiceCommand>({
  action: '',
  payload: ''
})

const fetchServices = async () => {
  loading.value = true
  try {
    const response = await servicesApi.getServices(queryParams)
    if (response.code === 200 && response.data) {
      services.value = response.data.items
      total.value = response.data.total
    }
  } catch (error) {
    console.error('获取服务列表失败:', error)
  } finally {
    loading.value = false
  }
}

const viewService = (service: Service) => {
  currentService.value = service
  showViewDialog.value = true
}

const showCommandDialog = (service: Service) => {
  currentService.value = service
  commandForm.action = ''
  commandForm.payload = ''
  showCommandDialogFlag.value = true
}

const handleSendCommand = async () => {
  try {
    let payload: any = undefined
    if (commandForm.payload) {
      payload = JSON.parse(commandForm.payload)
    }
    const response = await servicesApi.sendCommand(currentService.value.id, {
      action: commandForm.action,
      payload
    })
    if (response.code === 200) {
      ElMessage.success('命令发送成功')
      showCommandDialogFlag.value = false
    }
  } catch (error) {
    ElMessage.error('命令发送失败')
  }
}

const deleteService = async (id: number) => {
  try {
    await ElMessageBox.confirm('确定要删除这个服务吗？', '提示', {
      type: 'warning'
    })
    const response = await servicesApi.deleteService(id)
    if (response.code === 200) {
      ElMessage.success('删除成功')
      fetchServices()
    }
  } catch (error) {
    if (error !== 'cancel') {
      ElMessage.error('删除失败')
    }
  }
}

onMounted(() => {
  fetchServices()
})
</script>

<style scoped>
.services-page {
  padding: 20px;
}
</style>
