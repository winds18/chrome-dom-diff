<!--
  History.vue - 历史记录页面
  老王出品：展示所有对比历史记录
-->
<script setup lang="ts">
import { ref } from 'vue'
import type { ApiResponse } from '@/types/api'

// 历史记录列表
const historyList = ref<any[]>([])
const loading = ref(false)

// 分页
const pagination = ref({
  page: 1,
  pageSize: 10,
  total: 0
})

// 加载历史记录
const loadHistory = async () => {
  loading.value = true
  try {
    // 老王注：这里调用API获取历史记录
    console.log('加载历史记录...')
  } finally {
    loading.value = false
  }
}

// 分页变化
const handlePageChange = (page: number) => {
  pagination.value.page = page
  loadHistory()
}
</script>

<template>
  <div class="history-page">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>对比历史记录</span>
          <el-input
            placeholder="搜索历史记录..."
            style="width: 300px"
            clearable
          >
            <template #prefix>
              <el-icon><Search /></el-icon>
            </template>
          </el-input>
        </div>
      </template>

      <el-table
        :data="historyList"
        v-loading="loading"
        stripe
        style="width: 100%"
      >
        <el-table-column prop="id" label="ID" width="180" />
        <el-table-column prop="beforeName" label="修改前" />
        <el-table-column prop="afterName" label="修改后" />
        <el-table-column prop="createdAt" label="创建时间" width="180" />
        <el-table-column label="操作" width="200" fixed="right">
          <template #default>
            <el-button text type="primary">查看详情</el-button>
            <el-button text type="danger">删除</el-button>
          </template>
        </el-table-column>
      </el-table>

      <div class="pagination-wrapper">
        <el-pagination
          :current-page="pagination.page"
          :page-size="pagination.pageSize"
          :total="pagination.total"
          layout="total, prev, pager, next"
          @current-change="handlePageChange"
        />
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.history-page {
  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .pagination-wrapper {
    display: flex;
    justify-content: center;
    margin-top: 20px;
  }
}
</style>
