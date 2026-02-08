<!--
  NotFound.vue - 404页面
  老王出品：页面不存在时显示
-->
<script setup lang="ts">
import { useRouter } from 'vue-router'

const router = useRouter()

// 返回首页
const goHome = () => {
  router.push('/')
}
</script>

<template>
  <div class="not-found-page">
    <div class="content">
      <div class="error-code">404</div>
      <div class="error-message">页面不存在</div>
      <div class="error-description">
        抱歉，您访问的页面不存在或已被移除
      </div>
      <el-button type="primary" @click="goHome">返回首页</el-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.not-found-page {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  min-height: 400px;

  .content {
    text-align: center;

    .error-code {
      font-size: 120px;
      font-weight: bold;
      color: var(--color-primary);
      line-height: 1;
    }

    .error-message {
      font-size: 24px;
      color: var(--color-text-primary);
      margin: 20px 0 10px;
    }

    .error-description {
      font-size: 14px;
      color: var(--color-text-secondary);
      margin-bottom: 30px;
    }
  }
}
</style>
