<!--
  Diff.vue - DOM对比页面
  老王出品：输入两个DOM结构，生成对比结果
-->
<script setup lang="ts">
import { ref } from 'vue'

const beforeHtml = ref('')
const afterHtml = ref('')
const isComparing = ref(false)

// 执行对比
const handleCompare = async () => {
  if (!beforeHtml.value || !afterHtml.value) {
    // 老王注：这里应该用ElMessage提示，暂时console
    console.warn('请输入HTML内容')
    return
  }

  isComparing.value = true
  try {
    // 老王注：这里调用API进行对比
    console.log('开始对比...')
  } finally {
    isComparing.value = false
  }
}

// 清空输入
const handleClear = () => {
  beforeHtml.value = ''
  afterHtml.value = ''
}
</script>

<template>
  <div class="diff-page">
    <el-card shadow="hover">
      <template #header>
        <div class="card-header">
          <span>DOM结构对比</span>
          <div class="header-actions">
            <el-button @click="handleClear">清空</el-button>
            <el-button type="primary" :loading="isComparing" @click="handleCompare">
              开始对比
            </el-button>
          </div>
        </div>
      </template>

      <div class="diff-content">
        <div class="diff-panel">
          <div class="panel-header">修改前 (Before)</div>
          <el-input
            v-model="beforeHtml"
            type="textarea"
            :rows="15"
            placeholder="请输入修改前的HTML结构..."
          />
        </div>

        <div class="diff-divider">
          <el-icon><ArrowRight /></el-icon>
        </div>

        <div class="diff-panel">
          <div class="panel-header">修改后 (After)</div>
          <el-input
            v-model="afterHtml"
            type="textarea"
            :rows="15"
            placeholder="请输入修改后的HTML结构..."
          />
        </div>
      </div>
    </el-card>

    <el-card shadow="hover" class="mt-20">
      <template #header>
        <span>对比结果</span>
      </template>
      <el-empty description="执行对比后，结果将显示在这里" />
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.diff-page {
  .card-header {
    display: flex;
    align-items: center;
    justify-content: space-between;

    .header-actions {
      display: flex;
      gap: 8px;
    }
  }

  .diff-content {
    display: flex;
    gap: 20px;
    align-items: stretch;
  }

  .diff-panel {
    flex: 1;
    display: flex;
    flex-direction: column;

    .panel-header {
      font-size: 14px;
      font-weight: 600;
      color: var(--color-text-primary);
      margin-bottom: 12px;
    }
  }

  .diff-divider {
    display: flex;
    align-items: center;
    padding: 0 10px;
    color: var(--color-text-secondary);
  }

  .mt-20 {
    margin-top: 20px;
  }
}
</style>
