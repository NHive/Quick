<template>
  <div class="settings-container" v-if="isLoaded">
    <h3 class="settings-section-title">{{ t('settings.basic.title') }}</h3>
    <div class="settings-section">
      <!-- <div class="settings-options">
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.basic.theme") }}</span>
          <a-select v-model:value="themeMode" :options="themeOptions" class="setting-control" />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.basic.language") }}</span>
          <a-select
            v-model:value="currentLocale"
            :options="languageOptions"
            class="setting-control"
          />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.basic.clipboardMonitor") }}</span>
          <a-switch v-model:checked="enableMonitor" />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.basic.autoInsert") }}</span>
          <a-switch v-model:checked="autoInsert" />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.basic.defaultClipboard") }}</span>
          <a-switch v-model:checked="defaultSet" />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.basic.autoStart") }}</span>
          <a-switch v-model:checked="autostartEnabled" />
        </div>
        <div class='setting-item'>
          <span class='setting-label'>{{ t('settings.basic.imgFileConvToImg') }}</span>
          <a-switch v-model:checked='imgFileConvToImg' />
        </div>
        <div class='setting-item'>
          <span class='setting-label'>{{ t('settings.basic.lossyCompressedPicture') }}</span>
          <a-switch v-model:checked='lossyCompressedPicture' />
        </div>
        <div class='setting-item'>
          <span class='setting-label'>{{ t('settings.basic.silentStart') }}</span>
          <a-switch v-model:checked='silentStart' />
        </div>
        <div class='setting-item'>
          <span class='setting-label'>{{ t('settings.basic.pasteAndTop') }}</span>
          <a-switch v-model:checked='pasteAndTop' />
        </div>
      </div> -->
    </div>

    <!-- <div class="settings-section">
      <h3 class="settings-section-title">{{ t("settings.fileSync.title") }}</h3>
      <div class="settings-options">
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.fileSync.enable") }}</span>
          <a-switch v-model:checked="fileSync" />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.fileSync.sizeLimit") }}</span>
          <a-select
            v-model:value="capacityValue"
            :options="capacityOptions"
            class="setting-control"
          />
        </div>
        <div class="setting-item">
          <span class="setting-label">{{ t("settings.fileSync.keywordFilter") }}</span>
          <a-select
            v-model:value="selectValue"
            mode="tags"
            class="setting-control"
            :token-separators="[',']"
            :placeholder="t('settings.fileSync.keywordPlaceholder')"
            :options="keywordOptions"
            @change="updateSetting('keywordFilter', selectValue)"
          />
        </div>
      </div>
    </div> -->
  </div>
  <div v-else class="flex-row-center mt-50">
    <a-spin :indicator="indicator" />
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref, computed, watch, h } from 'vue';
import { type SelectProps } from 'ant-design-vue';
import { enable, isEnabled, disable } from '@tauri-apps/plugin-autostart';
import { LoadingOutlined } from '@ant-design/icons-vue';
import { useI18n } from 'vue-i18n';
import { emit } from '@tauri-apps/api/event';
import { Storage, SETTING_STORAGE_KEYS } from '@/utils/storage';
import { useTheme } from '@/utils/useTheme';
import { useLanguage } from '@/utils/i18n';

const indicator = h(LoadingOutlined, {
  style: { fontSize: '40px' },
  spin: true,
});

const { t } = useI18n();
const { currentLocale, updateLanguage, getLanguageOptions } = useLanguage();

const themeMode = ref('auto' as 'auto' | 'light' | 'dark');
const enableMonitor = ref(false);
const autoInsert = ref(false);
const defaultSet = ref(false);
const autostartEnabled = ref(false);
const fileSync = ref(false);
const capacityValue = ref<number>(10485760);
const selectValue = ref<string[]>([]);
const keywordOptions = ref<SelectProps['options']>([]);
const imgFileConvToImg = ref(false);
const lossyCompressedPicture = ref(false);
const silentStart = ref(false);
const pasteAndTop = ref(false);
const isLoaded = ref(false);

const { setTheme } = useTheme(themeMode);

// 语言选项
const languageOptions = getLanguageOptions();

// 监听语言变化
watch(currentLocale, async (newLocale) => {
  await Storage.set(SETTING_STORAGE_KEYS.LOCALE, newLocale);
  await updateLanguage(newLocale);
  await emit('setting-changed', { type: 'language', value: newLocale });
});

const themeOptions = computed(() => [
  { label: t('settings.basic.themeAuto'), value: 'auto' },
  { label: t('settings.basic.themeLight'), value: 'light' },
  { label: t('settings.basic.themeDark'), value: 'dark' },
]);

const capacityOptions: SelectProps['options'] = [
  { label: '10MB', value: 10 * 1024 * 1024 },
  { label: '20MB', value: 20 * 1024 * 1024 },
  { label: '50MB', value: 50 * 1024 * 1024 },
];

const updateSetting = async (key: string, value: any) => {
  try {
    const storageKey =
      SETTING_STORAGE_KEYS[
        key.toUpperCase() as keyof typeof SETTING_STORAGE_KEYS
      ];
    if (!storageKey) {
      throw new Error(`Invalid setting key: ${key}`);
    }
    await Storage.set(storageKey, value);
  } catch (error) {
    console.error(`Failed to update ${key}:`, error);
  }
};

const initializeSettings = async () => {
  try {
    // 1. 先初始化语言
    const savedLocale = await Storage.get(SETTING_STORAGE_KEYS.LOCALE, 'zh-CN');
    currentLocale.value = savedLocale;
    await updateLanguage(savedLocale); // 确保立即更新语言

    // 2. 初始化主题
    const savedTheme = (await Storage.get(
      SETTING_STORAGE_KEYS.THEME_MODE,
      'auto'
    )) as 'auto' | 'light' | 'dark';
    themeMode.value = savedTheme;
    setTheme(savedTheme); // 确保立即应用主题
    // 3. 初始化其他设置
    enableMonitor.value = await Storage.get(
      SETTING_STORAGE_KEYS.ENABLE_MONITOR,
      true
    );
    autoInsert.value = await Storage.get(
      SETTING_STORAGE_KEYS.AUTO_INSERT,
      true
    );
    defaultSet.value = await Storage.get(
      SETTING_STORAGE_KEYS.DEFAULT_SET,
      true
    );
    fileSync.value = await Storage.get(SETTING_STORAGE_KEYS.FILE_SYNC, true);
    capacityValue.value = await Storage.get(
      SETTING_STORAGE_KEYS.MAX_FILE_SIZE,
      10485760
    );
    imgFileConvToImg.value = await Storage.get(
      SETTING_STORAGE_KEYS.IMG_FILE_CONV_TO_IMG,
      true
    );
    lossyCompressedPicture.value = await Storage.get(
      SETTING_STORAGE_KEYS.LOSSY_COMPRESSED_PICTURE,
      true
    );
    silentStart.value = await Storage.get(
      SETTING_STORAGE_KEYS.SILENT_START,
      false
    );
    pasteAndTop.value = await Storage.get(
      SETTING_STORAGE_KEYS.PASTE_AND_TOP,
      true
    );

    // 修改自启动初始化逻辑
    const savedAutoStart = await Storage.get(
      SETTING_STORAGE_KEYS.AUTO_START,
      false
    );
    const currentAutoStart = await isEnabled();

    // 只更新状态，不执行启用/禁用操作
    autostartEnabled.value = currentAutoStart;

    // 如果存储的状态与实际状态不一致，更新存储
    if (savedAutoStart !== currentAutoStart) {
      await Storage.set(SETTING_STORAGE_KEYS.AUTO_START, currentAutoStart);
    }
  } catch (error) {
    console.error('Failed to initialize settings:', error);
  } finally {
    // 数据加载完毕
    isLoaded.value = true;
  }
};

const settingsToWatch = [
  { ref: currentLocale, key: 'LOCALE' },
  { ref: themeMode, key: 'THEME_MODE' },
  { ref: enableMonitor, key: 'ENABLE_MONITOR' },
  { ref: autoInsert, key: 'AUTO_INSERT' },
  { ref: defaultSet, key: 'DEFAULT_SET' },
  { ref: fileSync, key: 'FILE_SYNC' },
  { ref: capacityValue, key: 'MAX_FILE_SIZE' },
  { ref: imgFileConvToImg, key: 'IMG_FILE_CONV_TO_IMG' },
  { ref: lossyCompressedPicture, key: 'LOSSY_COMPRESSED_PICTURE' },
  { ref: silentStart, key: 'SILENT_START' },
  { ref: pasteAndTop, key: 'PASTE_AND_TOP' },
];

settingsToWatch.forEach(({ ref: setting, key }) => {
  watch(setting, async (newValue) => {
    await Storage.set(
      SETTING_STORAGE_KEYS[key as keyof typeof SETTING_STORAGE_KEYS],
      newValue
    );
  });
});

// 添加主题监听
watch(themeMode, async (newTheme) => {
  await Storage.set(SETTING_STORAGE_KEYS.THEME_MODE, newTheme);
  setTheme(newTheme);
  await emit('setting-changed', { type: 'theme', value: newTheme });
});

// 添加对自动启动开关的监听
watch(autostartEnabled, async (newValue) => {
  try {
    if (newValue) {
      await enable();
    } else {
      await disable();
    }
    // 可选：保存设置到本地存储
    await Storage.set(SETTING_STORAGE_KEYS.AUTO_START, newValue);
  } catch (error) {
    console.error('Failed to update autostart:', error);
    // 如果设置失败，回滚开关状态
    autostartEnabled.value = !newValue;
  }
});

onMounted(async () => {
  await initializeSettings();
});

defineOptions({ name: 'CustomSetting' });
</script>

<style scoped>
.settings-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px;
  color: var(--text-color);
}

.settings-section {
  background: var(--bg-color);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 24px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  margin-bottom: 24px;
  transition: all 0.3s ease;
}

.settings-section-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 24px;
}

.settings-options {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: #1d1d1f;
}

.setting-control {
  width: 200px;
  height: 36px;
}

:deep(.ant-select-selector) {
  height: 36px !important;
  border-radius: 8px !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
  background: rgba(255, 255, 255, 0.8) !important;
  backdrop-filter: blur(4px);
  transition: all 0.2s ease;
}

:deep(.ant-select:hover .ant-select-selector) {
  border-color: #4784ec !important;
  background: rgba(255, 255, 255, 0.95) !important;
}

:deep(.ant-select-focused .ant-select-selector) {
  border-color: #4784ec !important;
  box-shadow: 0 0 0 2px rgba(71, 132, 236, 0.2) !important;
  background: #ffffff !important;
}

:deep(.ant-switch) {
  background-color: rgba(120, 120, 128, 0.16);
  min-width: 36px;
  height: 20px;
  border-radius: 10px;
}

:deep(.ant-switch-checked) {
  background-color: #0290fe !important;
}

:deep(.ant-switch-handle) {
  width: 16px;
  height: 16px;
  top: 2px;
  left: 2px;
}

:deep(.ant-switch-checked .ant-switch-handle) {
  left: calc(100% - 18px);
}

/* 添加CSS变量 */
:root {
  --text-color: #000000;
  --text-secondary: #666666;
  --bg-color: rgba(255, 255, 255, 0.98);
  --card-bg: rgba(255, 255, 255, 0.9);
  --item-bg: rgba(255, 255, 255, 0.8);
  --item-hover-bg: rgba(255, 255, 255, 0.95);
  --border-color: rgba(0, 0, 0, 0.06);
}

:root[data-theme='dark'] {
  --text-color: #ffffff;
  --text-secondary: #b0b0b0;
  --bg-color: rgba(28, 28, 30, 0.98);
  --card-bg: rgba(44, 44, 46, 0.98);
  --item-bg: rgba(44, 44, 46, 0.95);
  --item-hover-bg: rgba(58, 58, 60, 0.98);
  --border-color: rgba(255, 255, 255, 0.12);
}

/* 替换原来的深色模式适配 */
:root[data-theme='dark'] .settings-section {
  background: var(--bg-color);
}

:root[data-theme='dark'] .settings-section-title,
:root[data-theme='dark'] .setting-label {
  color: #f5f5f7;
}

/* 添加文字阴影效果 */
.settings-section-title,
.setting-label {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}
</style>
