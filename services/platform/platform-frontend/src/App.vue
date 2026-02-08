<!-- 艹！老王的前端根组件 -->
<!-- 使用router-view渲染路由组件 -->

<template>
  <router-view />
</template>

<script setup lang="ts">
// 艹！根组件就这么简单
// 所有的页面内容都由router-view渲染
</script>

<style>
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 'Helvetica Neue', Arial, sans-serif;
}

#app {
  width: 100%;
  height: 100vh;
}
</style>
