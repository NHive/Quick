<template>
  <div class="container">
    <div class="controlPanel">
      <!-- Left side: Function buttons -->
      <ul class="functionButtons">
        <li class="controlIcon collapse" @click="hideControlWindow">
          <div class="iconWrapper" v-html="icons.collapse"></div>
        </li>
        <li class="controlIcon moveIcon">
          <div class="dragHandle" data-tauri-drag-region></div>
          <div class="iconWrapper" v-html="icons.move"></div>
        </li>
        <li class="controlIcon homeIcon" @click="openWindowByLink">
          <div class="iconWrapper" v-html="icons.home"></div>
        </li>
        <li class="controlIcon settingIcon" @click="openSettingWindow">
          <div class="iconWrapper" v-html="icons.setting"></div>
        </li>
        <li class="controlIcon pinIcon" @click="togglePinWindow">
          <div class="iconWrapper" v-html="isPinned ? icons.pinFilled : icons.pinOutline"></div>
        </li>
      </ul>

      <!-- Right side: Window tabs -->
      <div class="windowTabs">
        <div v-for="window in windows" :key="window.label" :class="['windowTab', { active: window.status === 'Front' }]"
          @click="switchToWindow(window.label)">
          <span class="tabTitle">{{ window.title }}</span>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { icons } from '@/utils/svg';
import { invoke } from "@tauri-apps/api/core";
import { onMounted, onUnmounted, ref } from 'vue';
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

// Window configurations
const windowConfigs = [
  { title: "deepseek", url: "https://chat.deepseek.com" },
  { title: "doubao", url: "https://www.doubao.com/chat/" },
  { title: "kimi", url: "https://kimi.moonshot.cn/" }
];

// Define the structure of window information
interface WindowInfo {
  label: string;
  title: string;
  url: string;
  status: 'Front' | 'Back';
  loaded: boolean;
  position?: {
    x: number;
    y: number;
    width: number;
    height: number;
  };
}

// Reactive windows list
const windows = ref<WindowInfo[]>([]);
// Track window pin state
const isPinned = ref(false);

// Configure windows
const configureWindows = async () => {
  try {
    await invoke('cmd_configure_windows', { configs: windowConfigs });
    console.log('Windows configured successfully');
    await refreshWindowsList();
  } catch (error) {
    console.error('Failed to configure windows:', error);
  }
};

// Refresh the windows list
const refreshWindowsList = async () => {
  try {
    const allWindows = await invoke<WindowInfo[]>('cmd_get_all_windows');
    windows.value = allWindows;
    console.log('All windows:', windows.value);

    // Also update active window
    await getActiveWindow();
  } catch (error) {
    console.error('Failed to get all windows:', error);
  }
};

// Get active window
const getActiveWindow = async () => {
  try {
    const activeWindow = await invoke<WindowInfo>('cmd_get_active_window');
    console.log('Active window:', activeWindow);

    // Update active status in windows list
    if (activeWindow) {
      windows.value = windows.value.map(win => ({
        ...win,
        status: win.label === activeWindow.label ? 'Front' : 'Back'
      }));
    }
  } catch (error) {
    console.error('Failed to get active window:', error);
  }
};

// Open settings window
const openSettingWindow = async () => {
  try {
    await invoke('open_setting_window');
  } catch (error) {
    console.error('Failed to open setting window:', error);
  }
};

// Hide control window
const hideControlWindow = async () => {
  try {
    await invoke('cmd_hide_control_window');
  } catch (error) {
    console.error('Failed to hide control window:', error);
  }
};

// Open window by link
const openWindowByLink = async () => {
  try {
    await invoke('cmd_create_window', {
      url: "https://chat.deepseek.com",
      title: "deepseek"
    });

    // Refresh the windows list after opening a new window
    setTimeout(refreshWindowsList, 500);
  } catch (error) {
    console.error('Failed to open window with URL:', error);
  }
};

// Switch to a specific window
const switchToWindow = async (label: string) => {
  try {
    await invoke('cmd_switch_to_window', { label });
    await refreshWindowsList(); // Update tab statuses
  } catch (error) {
    console.error(`Failed to switch to window ${label}:`, error);
  }
};

// Toggle window pin state
const togglePinWindow = async () => {
  try {
    isPinned.value = !isPinned.value;
    await invoke('set_window_pin', { pin: isPinned.value });
    console.log(`Window pin state set to: ${isPinned.value}`);
  } catch (error) {
    console.error('Failed to toggle window pin state:', error);
  }
};

// Get current pin state
const getWindowPinState = async () => {
  try {
    isPinned.value = await invoke('get_window_pin');
    console.log(`Window pin state loaded: ${isPinned.value}`);
  } catch (error) {
    console.error('Failed to get window pin state:', error);
  }
};

// Set up event listeners when component is mounted
onMounted(async () => {
  try {
    const window = getCurrentWebviewWindow();

    // Listen for focus changes to update tabs
    const unlisten = await window.onFocusChanged(async (focused) => {
      if (focused) {
        await refreshWindowsList();
      }
    });

    // Clean up on unmount
    onUnmounted(() => {
      unlisten();
    });

    // Initialize windows configuration
    await configureWindows();

    // Get initial pin state
    await getWindowPinState();

    // Set up a periodic refresh to keep tabs in sync
    const refreshInterval = setInterval(refreshWindowsList, 3000);
    onUnmounted(() => {
      clearInterval(refreshInterval);
    });

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
  background-color: #f9fbff;
  overflow: hidden;
}

.controlPanel {
  width: 100%;
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 5px 10px;
  background-color: #f0f2f5;
  border-bottom: 1px solid #e0e0e0;
}

.functionButtons {
  list-style: none;
  display: flex;
  flex-direction: row;
  justify-content: flex-start;
  align-items: center;
  margin: 0;
  padding: 0;

  .controlIcon {
    position: relative;
    display: flex;
    justify-content: center;
    align-items: center;
    box-sizing: border-box;
    width: 30px;
    height: 30px;
    cursor: pointer;
    margin-right: 12px;
    color: #888;

    &:hover {
      color: #333;
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
    opacity: 0.8;
  }

  .collapse .iconWrapper {
    width: 20px;
    height: 20px;
  }

  .homeIcon,
  .settingIcon,
  .pinIcon {
    z-index: 15;
  }
}

.windowTabs {
  display: flex;
  flex-direction: row;
  align-items: center;
  overflow-x: auto;
  max-width: 70%;
  scrollbar-width: thin;

  &::-webkit-scrollbar {
    height: 3px;
  }

  &::-webkit-scrollbar-thumb {
    background: #ccc;
    border-radius: 3px;
  }
}

.windowTab {
  padding: 5px 10px;
  margin-left: 2px;
  min-width: 100px;
  max-width: 160px;
  height: 28px;
  display: flex;
  align-items: center;
  justify-content: center;
  background-color: #e4e6e8;
  border-radius: 4px 4px 0 0;
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;

  &:hover {
    background-color: #d8dade;
  }

  &.active {
    background-color: #fff;
    border: 1px solid #ddd;
    border-bottom: none;
    font-weight: 500;
  }

  .tabTitle {
    font-size: 13px;
    color: #333;
  }
}
</style>