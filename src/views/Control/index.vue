<template>
  <div class="container">
    <ul class="controlBox">
      <li class="controlIcon collapse" v-html="icons.collapse"></li>
      <li class="controlIcon" v-html="icons.fullScreen"></li>
      <li class="controlIcon" v-html="icons.home"></li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { icons } from '@/utils/svg';
</script>
<style lang="scss" scoped>
.container {
  height: 100vh;
  margin: 0;
  background-color: #f0f0f0; /* 设置背景颜色 */
}


.controlBox {
  list-style: none;
  padding: 15px 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;
  .controlIcon {
    display: flex;
    justify-content: center;
    align-items: center;
    background: #fff;
    width: 40px;
    height: 40px;
    padding: 4px 4px 6px 4px;
    border-radius: 50%;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* 添加阴影 */
  }
  .controlIcon:not(:last-child) {
    margin-bottom: 12px;
  }
  .collapse{
    transform: rotate(45deg);
  }
}
</style>
