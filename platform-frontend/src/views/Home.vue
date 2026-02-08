<!--
  Home.vue - 首页仪表盘
  老王出品：展示系统概览和统计信息
-->
<script setup lang="ts">
import { ref, onMounted } from 'vue'

// 统计卡片数据
const stats = ref([
  { title: '在线服务', value: 0, icon: 'Server', color: '#409eff' },
  { title: '运行任务', value: 0, icon: 'Loading', color: '#67c23a' },
  { title: '今日对比', value: 0, icon: 'DocumentCopy', color: '#e6a23c' },
  { title: '错误日志', value: 0, icon: 'Warning', color: '#f56c6c' }
])

onMounted(() => {
  // 老王注：这里后面要从API获取真实数据
  console.log('Home页面加载完成')
})
</script>

<template>
  <div class="home-page">
    <el-row :gutter="20">
      <el-col v-for="stat in stats" :key="stat.title" :span="6">
        <el-card shadow="hover" class="stat-card">
          <div class="stat-content">
            <div class="stat-icon" :style="{ backgroundColor: stat.color }">
              <el-icon :size="32">
                <component :is="stat.icon" />
              </el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stat.value }}</div>
              <div class="stat-title">{{ stat.title }}</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <el-row :gutter="20" class="mt-20">
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>最近对比记录</span>
              <el-button text>查看全部</el-button>
            </div>
          </template>
          <el-empty description="暂无数据" />
        </el-card>
      </el-col>
      <el-col :span="12">
        <el-card shadow="hover">
          <template #header>
            <div class="card-header">
              <span>系统日志</span>
              <el-button text>查看全部</el-button>
            </div>
          </template>
          <el-empty description="暂无数据" />
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<style scoped lang="scss">
.home-page {
  .stat-card {
    .stat-content {
      display: flex;
      align-items: center;
      gap: 16px;

      .stat-icon {
        display: flex;
        align-items: center;
        justify-content: center;
        width: 64px;
        height: 64px;
        border-radius: 12px;
        color: #fff;
      }

      .stat-info {
        flex: 1;

        .stat-value {
          font-size: 28px;
          font-weight: 600;
          color: var(--color-text-primary);
        }

        .stat-title {
          font-size: 14px;
          color: var(--color-text-secondary);
          margin-top: 4px;
        }
      }
    }
  }

  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .mt-20 {
    margin-top: 20px;
  }
}
</style>
