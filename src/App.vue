<template>
  <a-config-provider
    :locale="localeLang === 'en-US' ? enUS : zhCN"
    :theme="{
      algorithm:
        themeModule === 'dark'
          ? antdTheme.darkAlgorithm
          : antdTheme.defaultAlgorithm,
    }"
  >
    <RouterView />
  </a-config-provider>
</template>

<script setup lang="ts">
import { ref, watchEffect, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { theme as antdTheme } from 'ant-design-vue';
import enUS from 'ant-design-vue/es/locale/en_US';
import zhCN from 'ant-design-vue/es/locale/zh_CN';
import { SETTING_STORAGE_KEYS, Storage } from './utils/storage';
import { listen } from '@tauri-apps/api/event';
import { invoke } from '@tauri-apps/api/core';

const { locale } = useI18n({ useScope: 'global' });
// 创建对语言变量的引用
const localeLang = ref(locale);
const themeModule = ref('light');
const { token } = antdTheme.useToken();
// 监听语言变化并更新
watchEffect(() => {
  localeLang.value = locale.value;
});

const updateTheme = async (type: string) => {
  if (type === 'theme') {
    themeModule.value = await Storage.get(
      SETTING_STORAGE_KEYS.THEME_MODE,
      'light'
    );
  }
};

// 添加默认窗口链接
const initDefaultWin = async () => {
  let data: Array<any> = await invoke('cmd_get_setting_window_configs');
  // 默认值处理；若窗口配置无值，则添加默认值
  if (!data || !Array.from(data).length) {
    await invoke('cmd_add_window', {
      title: 'deepseek',
      url: 'https://www.deepseek.com/',
    });
  }
};

onMounted(async () => {
  // 初始化项目配置
  await Storage.initStorage();
  await updateTheme('theme');
  initDefaultWin()
  listen('setting-changed', (event: { payload: { type: string } }) => {
    updateTheme(event.payload.type);
  });
});
</script>
