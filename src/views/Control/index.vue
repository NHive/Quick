<template>
  <div class="container">
    <ul class="controlBox">
      <li class="controlIcon collapse" v-html="icons.collapse"></li>
      <li class="controlIcon" v-html="icons.move"></li>
      <li class="controlIcon" v-html="icons.home"></li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import { icons } from '@/utils/svg';
</script>
<style lang="scss" scoped>
.container {
  width: 80px;
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
    box-sizing: border-box;
    width: 40px;
    height: 40px;
    padding: 8px;
    border-radius: 50%;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* 添加阴影 */
  }
  .controlIcon:not(:last-child) {
    margin-bottom: 12px;
  }
  .collapse{
    padding: 10px;
  }
}
</style>
