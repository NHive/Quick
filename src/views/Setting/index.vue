<template>
  <div class="setting-container">
    <div v-if="isWindows" class="window-controls" data-tauri-drag-region>
      <div class="control-button minimize" @click="minimizeWindow">
        <span v-html="icons.minimize"></span>
      </div>
      <div class="control-button close" @click="closeWindow">
        <span v-html="icons.close"></span>
      </div>
    </div>
    <div class="setting-box">
      <div class="setting-sider" data-tauri-drag-region>
        <div class="drag-header" data-tauri-drag-region></div>
        <Sider />
      </div>
      <div class="setting-content" data-tauri-drag-region>
        <router-view v-slot="{ Component }">
          <keep-alive>
            <component :is="Component" />
          </keep-alive>
        </router-view>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
// eslint-disable-next-line @typescript-eslint/no-unused-vars
import { onMounted, ref } from 'vue';
import { useRoute, RouterView } from 'vue-router';
import { getCurrentWebviewWindow } from '@tauri-apps/api/webviewWindow';
import { Storage, SETTING_STORAGE_KEYS } from '@/utils/storage';
import { useTheme } from '@/utils/useTheme';
import { useLanguage } from '@/utils/i18n';
import { icons } from '@/utils/svg';
import { Sider } from './components';
import { theme as antdTheme } from 'ant-design-vue';

const appWindow = getCurrentWebviewWindow();
const { token } = antdTheme.useToken();

const route = useRoute();
const isVertical = ref<boolean>(false);
const id = ref<string | null>(null);
const themeMode = ref<'auto' | 'light' | 'dark'>('auto');
const isWindows = ref(false);

const minimizeWindow = () => {
  appWindow.minimize();
};

const closeWindow = () => {
  appWindow.close();
};

const initializeApp = async () => {
  console.log('cus00');
  try {
    // 初始化语言
    const { updateLanguage } = useLanguage();
    const savedLocale = await Storage.get(SETTING_STORAGE_KEYS.LOCALE, 'zh-CN');
    await updateLanguage(savedLocale);

    // 初始化主题
    const savedTheme = (await Storage.get(
      SETTING_STORAGE_KEYS.THEME_MODE,
      'auto'
    )) as 'auto' | 'light' | 'dark';
    themeMode.value = savedTheme;
    const { setTheme } = useTheme(themeMode);
    setTheme(savedTheme);
    isWindows.value = await Storage.get('isWindows', false);
  } catch (error) {
    console.error('Failed to initialize app settings:', error);
  }
};

onMounted(async () => {
  await initializeApp();
  isVertical.value = Boolean(route.query.verticalModule);
  id.value = route.query.id ? String(route.query.id) : null;
});
</script>

<style lang="scss" scoped>
.setting-container {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  // background: rgba(245, 245, 247, 0.95);
  border-radius: 8px;
  overflow: hidden;
  background:v-bind("token.colorBgContainer");


  // :root[data-theme='dark'] & {
  //   background: rgba(28, 28, 30, 0.95);
  // }
}

.setting-box {
  flex: 1;
  display: flex;
  height: 100%;
}

.setting-sider {
  width: 200px;
  // background: #e9e8e7;
  background:v-bind("token.colorBgLayout");
  flex-shrink: 0;
  padding: 0 0 12px;
  border-right: 1px solid rgba(0, 0, 0, 0.1);

  // :root[data-theme='dark'] & {
  //   background: #2c2c2e;
  //   border-right: 1px solid rgba(255, 255, 255, 0.1);
  // }

  :deep(.menu-item) {
    padding: 8px 16px;
    margin: 0 8px;
    border-radius: 6px;
    color: #1d1d1f;
    cursor: pointer;
    transition: all 0.2s;

    // :root[data-theme='dark'] & {
    //   color: #ffffff;
    // }

    // &:hover {
    //   background: rgba(0, 0, 0, 0.06);

    //   :root[data-theme='dark'] & {
    //     background: rgba(255, 255, 255, 0.1);
    //   }
    // }

    // &.active {
    //   background: #4784ec;
    //   color: #ffffff;
    // }
  }
}

.setting-content {
  flex: 1;
  // background: #f9f9f9;
  background:v-bind("token.colorBgLayout");
  padding: 20px;
  overflow-y: auto;
  height: 100vh;

  &::-webkit-scrollbar {
    display: none;
  }

  -ms-overflow-style: none;
  /* IE and Edge */
  scrollbar-width: none;
  /* Firefox */

  // :root[data-theme='dark'] & {
  //   background: #2c2c2e;
  // }
}

.drag-header {
  height: 40px;
  width: 100%;
  -webkit-user-select: none;
  position: relative;
}

.window-controls {
  position: fixed;
  top: 0;
  right: 0;
  display: flex;
  height: 40px;
  align-items: center;
  z-index: 9999;

  .control-button {
    width: 52px;
    height: 40px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;

    :deep(svg) {
      width: 16px;
      height: 16px;

      path {
        fill: currentColor;
      }
    }

    // &:hover {
    //   background-color: rgba(0, 0, 0, 0.06);

    //   :root[data-theme='dark'] & {
    //     background-color: rgba(255, 255, 255, 0.08);
    //   }
    // }

    &.minimize {
      // &:hover {
      //   background-color: rgba(0, 0, 0, 0.06);

      //   :root[data-theme='dark'] & {
      //     background-color: rgba(255, 255, 255, 0.08);
      //   }
      // }
    }

    &.close {
      &:hover {
        background-color: #c42b1c;
        color: white;
        border-top-right-radius: 8px;
      }
    }
  }
}
</style>
