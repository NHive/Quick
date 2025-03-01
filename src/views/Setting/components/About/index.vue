<template>
  <div class="about-container">
    <div class="about-content">
      <div class="app-icon">
        <img src="../../../../assets/image/icon.png" alt="NewbeePaste" />
      </div>

      <h1 class="app-name">Paste Newbee</h1>
      <p class="version">{{ t("settings.about.version") }} {{ appVersion }}</p>
      <a-button
        type="primary"
        :loading="checkUpdateLoading"
        class="update-btn"
        @click="handleUpdate"
      >
        {{ t("settings.about.checkUpdate") }}
      </a-button>

      <div class="divider"></div>

      <div class="app-info">
        <p class="description">{{ t("settings.about.description") }}</p>
        <p class="copyright">{{ t("settings.about.copyright") }}</p>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, createVNode } from "vue"
import { getVersion } from "@tauri-apps/api/app"
import { useI18n } from "vue-i18n"
import { invoke } from "@tauri-apps/api/core"
import { message, Modal } from "ant-design-vue"
import { ExclamationCircleOutlined } from "@ant-design/icons-vue"
import { Storage } from "@/utils/storage"

const { t } = useI18n()
const appVersion = ref(t("settings.about.loading"))
const checkUpdateLoading = ref<boolean>(false)
const isMacOS = ref<boolean>(false)

const fetchAppVersion = async () => {
  try {
    const version = await getVersion()
    appVersion.value = version
  } catch (error) {
    console.error("Failed to get version:", error)
    appVersion.value = t("settings.about.loadFailed")
  }
}

const handleUpdate = async () => {
  try {
    checkUpdateLoading.value = true
    const isNeedUpdate = await invoke("update_version")
    if (!isNeedUpdate) {
      message.warning(`${t("tips.notUpdated")}!`)
    } else if (isNeedUpdate && !isMacOS.value) {
      message.success(`${t("tips.updateNow")}!`)
    } else {
      Modal.confirm({
        title: t("tips.newVersion"),
        icon: createVNode(ExclamationCircleOutlined),
        content: createVNode("div", {},`${t('tips.confirmUpdateNow')}?`),
        onOk() {
          // 重启逻辑
          invoke('restart_app')
        }
      })
    }
    console.log("update", isNeedUpdate)
  } catch (error) {
    console.error("Failed to check update:", error)
  } finally {
    setTimeout(() => {
      checkUpdateLoading.value = false
    }, 200)
  }
}

onMounted(async () => {
  await fetchAppVersion()
  // 检测操作系统
  isMacOS.value = await Storage.get("isMacOS", false)
})
</script>

<style lang="scss" scoped>
.about-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px;
  color: var(--text-color);
}

.about-content {
  background: var(--bg-color);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 24px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.15);
  transition: all 0.3s ease;
  text-align: center;
}

.app-icon {
  margin-bottom: 1.5rem;

  img {
    width: 128px;
    height: 128px;
    border-radius: 24px;
  }
}

.app-name {
  font-size: 24px;
  font-weight: 700;
  color: var(--text-color) !important;
  margin: 0;
}

.version {
  font-size: 14px;
  color: var(--text-secondary) !important;
  margin: 0.5rem 0;
  font-weight: 500;
}

.divider {
  height: 1px;
  background: var(--text-secondary);
  margin: 1rem 0;
}

.app-info {
  .description {
    font-size: 14px;
    color: var(--text-color) !important;
    margin-bottom: 1rem;
    font-weight: 500;
  }

  .copyright {
    font-size: 12px;
    color: var(--text-secondary) !important;
    font-weight: 500;
  }
}

// 优化后的主题变量
:root {
  --text-color: #000000;
  --text-secondary: #666666;
  --bg-color: rgba(255, 255, 255, 0.98);
  --card-bg: rgba(255, 255, 255, 0.9);
}

:root[data-theme="dark"] {
  --text-color: #ffffff;
  --text-secondary: #b0b0b0;
  --bg-color: rgba(28, 28, 30, 0.98);
  --card-bg: rgba(44, 44, 46, 0.98);
}

// 优化文字阴影效果
.app-name,
.version,
.app-info .description,
.app-info .copyright {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

:root[data-theme="dark"] {
  .app-name,
  .version,
  .app-info .description,
  .app-info .copyright {
    text-shadow: 0 1px 3px rgba(255, 255, 255, 0.1);
  }

  .app-name {
    color: #ffffff !important;
  }

  .version,
  .app-info .copyright {
    color: #b0b0b0 !important;
  }

  .app-info .description {
    color: #f0f0f0 !important;
  }
}

.update-btn {
  margin: 0px auto;
  padding: 6px 12px;
  border-radius: 4px;
  border: 1px solid var(--text-secondary);
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  display: inline-block;
  min-width: 80px;
  text-align: center;

  &:hover {
    color: var(--bg-color);
  }

  &:active {
    transform: translateY(1px);
  }

  outline: none;
  -webkit-appearance: none;
  -moz-appearance: none;
  appearance: none;
}

// 确保深色模式下的样式正确应用
:root[data-theme="dark"] {
  .update-btn {
    border: 1px solid var(--text-secondary);
    color: var(--text-secondary);

    &:hover {
      color: var(--bg-color);
    }
  }
}
</style>
