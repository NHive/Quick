<template>
  <div class="container">
    <ul class="controlBox">
      <li class="controlIcon collapse">
        <div class="iconWrapper" v-html="icons.collapse"></div>
      </li>
      <li class="controlIcon moveIcon">
        <!-- 专门为拖拽创建一个div，完全覆盖整个移动图标 -->
        <div class="dragHandle" data-tauri-drag-region></div>
        <div class="iconWrapper" v-html="icons.move"></div>
      </li>
      <li class="controlIcon homeIcon" @click="openWindowByLink">
        <div class="iconWrapper" v-html="icons.home"></div>
      </li>
      <li class="controlIcon settingIcon" @click="openSettingWindow">
        <div class="iconWrapper" v-html="icons.setting"></div>
      </li>
    </ul>
  </div>
</template>
<script setup lang="ts">
import { icons } from '@/utils/svg';
import { invoke } from "@tauri-apps/api/core"

// 设置窗口和URL窗口使用完全不同的命令
const openSettingWindow = async () => {
  try {
    const response = await invoke('open_setting_window');
    console.log('Setting window opened:', response);
  } catch (error) {
    console.error('Failed to open setting window:', error);
  }
};

const openWindowByLink = async () => {
  try {
    // 确保这是一个单独的、不同的命令
    const response = await invoke('open_window_by_url', {
      url: "https://chat.deepseek.com"
    });
    console.log('Window opened with URL:', response);
  } catch (error) {
    console.error('Failed to open window with URL:', error);
    // 添加更多调试信息
    console.debug('Command details:', {
      command: 'open_window_by_url',
      params: { url: "https://chat.deepseek.com" }
    });
  }
};
</script>
<style lang="scss" scoped>
.container {
  width: 100%;
  height: 100vh;
  margin: 0;
  background-color: #f0f0f0;
  overflow: hidden;
}

.controlBox {
  list-style: none;
  padding: 15px 0;
  display: flex;
  flex-direction: column;
  justify-content: flex-start;
  align-items: center;

  .controlIcon {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    background: #fff;
    box-sizing: border-box;
    width: 40px;
    height: 40px;
    border-radius: 50%;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1);
    cursor: pointer;
    margin-bottom: 12px;
  }

  .moveIcon {
    cursor: move;
  }

  .dragHandle {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    z-index: 10;
    border-radius: 50%;
  }

  .iconWrapper {
    width: 24px;
    height: 24px;
    display: flex;
    justify-content: center;
    align-items: center;
    position: relative;
    z-index: 5;
    pointer-events: none;
    /* 防止图标本身接收点击事件 */
  }

  .collapse .iconWrapper {
    width: 20px;
    height: 20px;
  }

  .collapse {
    cursor: pointer;
  }

  /* 为每个图标添加特定的类，便于调试和维护 */
  .homeIcon,
  .settingIcon {
    z-index: 15;
    /* 确保点击区域在最上层 */
  }
}
</style>