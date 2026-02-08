<!--
  App.vue - 应用根组件
  老王出品：简洁的应用入口
-->
<script setup lang="ts">
// 老王注：这里只需要router-view，布局在Layout组件里处理
</script>

<template>
  <router-view />
</template>

<style>
/* 全局样式重置 */
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

html, body, #app {
  width: 100%;
  height: 100%;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}
</style>
