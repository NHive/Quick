<template>
  <div class="sider-menu">
    <div
      v-for="item in siderItems"
      :key="item.value"
      :class="['menu-item', { active: activedKey === item.value }]"
      @click="clickSider(item)"
    >
      <span v-if="item.icon" v-html="item.icon"></span>
      <span>{{ item.label }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import { listen } from "@tauri-apps/api/event"
import { useRoute, useRouter } from "vue-router"
import { icons } from "@/utils/svg"

interface MenuItem {
  value: string
  label: string
  icon?: string
}

const { t } = useI18n()
const router = useRouter()
const route = useRoute()
const siderItems = computed<MenuItem[]>(() => [
  { value: "customSetting", label: t("settings.menu.generalSettings"), icon: icons.setting },
  { value: "win", label: t("settings.menu.window"), icon: icons.shortcuts },
  { value: "network", label: t("settings.menu.network"), icon: icons.network },
  // { value: "historicalRecord", label: t("settings.menu.history"), icon: icons.history },
  // { value: "about", label: t("settings.menu.about"), icon: icons.about }
])

const activedKey = ref<string>("user")

const clickSider = (item: MenuItem) => {
  if (activedKey.value === item.value) return
  activedKey.value = item.value
  router.replace(`/setting/${item.value}`)
}

onMounted(() => {
  activedKey.value = route.name ? String(route.name) : "user"
  listen("changeSider", (data: Record<string, any>) => {
    clickSider({ value: data.payload.currentSider ?? "user", label: "" })
  })
})
</script>

<style lang="scss" scoped>
.sider-menu {
  background: transparent;
  border-right: none;
  min-width: 200px;

  .menu-item {
    display: flex;
    align-items: center;
    gap: 8px;
    margin: 2px 8px;
    border-radius: 6px;
    padding: 8px 12px;
    color: #1d1d1f;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    transition: all 0.2s cubic-bezier(0.4, 0, 0.2, 1);

    :deep(svg) {
      width: 16px;
      height: 16px;

      path {
        fill: currentColor;
        transition: fill 0.2s;
      }
    }

    &:hover {
      background: rgba(0, 0, 0, 0.04);
    }

    &.active {
      background: #0071e3;
      color: #ffffff;

      &:hover {
        background: #0077ed;
      }
    }
  }
}

:root[data-theme="dark"] {
  .sider-menu {
    .menu-item {
      color: #e5e5e5;

      &:hover {
        background: rgba(255, 255, 255, 0.06);
      }

      &.active {
        background: #0077ed;
        color: #ffffff;

        &:hover {
          background: #0084ff;
        }
      }
    }
  }
}
</style>
