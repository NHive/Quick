<template>
  <a-config-provider
    :locale="localeLang === 'en-US' ? enUS : zhCN"
    :theme="{
      token: { colorPrimary: '#f9fbff' },
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

onMounted(async () => {
  // 初始化项目配置
  await Storage.initStorage();
  await updateTheme('theme');
  listen('setting-changed', (event: { payload: { type: string } }) => {
    updateTheme(event.payload.type);
  });
});
</script>
