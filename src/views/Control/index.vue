<template>
  <a-config-provider
    :theme="{
      token: { colorPrimary: '#f9fbff' },
    }"
  >
    <div class="container">
      <ul class="controlBox">
        <li class="controlIcon collapse" @click="openSettingWindow">
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
      </ul></div
  ></a-config-provider>
</template>

<script setup lang="ts">
import { theme as antdTheme } from 'ant-design-vue';

import { icons } from '@/utils/svg';
import { invoke } from '@tauri-apps/api/core';
import { onMounted, onUnmounted } from 'vue';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';

const data = antdTheme.useToken();
console.log('#theme', data, data.token.value);

// 定义窗口配置
const windowConfigs = [
  { title: 'deepseek', url: 'https://chat.deepseek.com' },
  { title: 'doubao', url: 'https://www.doubao.com/chat/' },
];

// 调用 cmd_configure_windows 命令
const configureWindows = async () => {
  try {
    await invoke('cmd_configure_windows', { configs: windowConfigs });
    console.log('Windows configured successfully');
    await getAllWindows();
    await getActiveWindow();
  } catch (error) {
    console.error('Failed to configure windows:', error);
  }
};

// 获取所有窗口信息
const getAllWindows = async () => {
  try {
    const windows = await invoke('cmd_get_all_windows');
    console.log('All windows:', windows);
  } catch (error) {
    console.error('Failed to get all windows:', error);
  }
};

// 获取活动窗口信息
const getActiveWindow = async () => {
  try {
    const activeWindow = await invoke('cmd_get_active_window');
    console.log('Active window:', activeWindow);
  } catch (error) {
    console.error('Failed to get active window:', error);
  }
};

const openSettingWindow = async () => {
  try {
    await invoke('open_setting_window');
  } catch (error) {
    console.error('Failed to open setting window:', error);
  }
};

const openWindowByLink = async () => {
  try {
    await invoke('cmd_create_window', {
      url: 'https://chat.deepseek.com',
      title: 'deepseek',
    });
  } catch (error) {
    console.error('Failed to open window with URL:', error);
  }
};

// 添加窗口显示监听
onMounted(async () => {
  try {
    const window = getCurrentWebviewWindow();

    // 监听窗口显示事件
    const unlisten = await window.onFocusChanged(() => {
      console.log('Window is shown, opening link window');
      openWindowByLink();
    });

    // 清理函数
    onUnmounted(() => {
      unlisten();
    });

    // // 如果窗口已经是可见状态，也调用一次
    // const isVisible = await window.isVisible();
    // if (isVisible) {
    //   console.log('Window is already visible, opening link window');
    //   openWindowByLink();
    // }

    // 配置窗口列表并获取窗口信息
    await configureWindows();
  } catch (error) {
    console.error('Error setting up window event listener:', error);
  }
});
</script>

<style lang="scss" scoped>
.container {
  width: 100%;
  height: 100vh;
  margin: 0;
  background-color: v-bind('data.token.value.colorBgLayout');
  overflow: hidden;
  transition: all 0.3s;
}

.controlBox {
  list-style: none;
  padding: 10px 15px;
  display: flex;
  flex-direction: row; /* 由纵向改为横向 */
  justify-content: flex-start;
  align-items: center;

  .controlIcon {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    box-sizing: border-box;
    width: 30px;
    height: 30px;
    cursor: pointer;
    margin-right: 20px; /* 由下边距改为右边距 */
    margin-bottom: 0; /* 移除下边距 */
  }
  :deep(svg) {
    path {
      fill: v-bind('data.token.value.colorTextBase');
      transition: fill 0.2s;
    }
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
    opacity: 0.8;
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
