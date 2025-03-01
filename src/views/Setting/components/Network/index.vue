<template>
  <div class="settings-container">
    <h3 class="settings-section-title">
      {{ t('settings.menu.network') }}
    </h3>
    <div class="settings-section">
      <div class="settings-options"></div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
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
