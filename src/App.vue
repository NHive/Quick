<template>
  <a-config-provider
    :locale="localeLang === 'en-US' ? enUS : zhCN"
    :theme="{
      token: {
        colorPrimary: '#f8da52'
      }
    }"
  >
    <RouterView />
  </a-config-provider>
</template>

<script setup lang="ts">
import { ref, watchEffect, onMounted } from "vue"
import { useI18n } from "vue-i18n"
import enUS from "ant-design-vue/es/locale/en_US"
import zhCN from "ant-design-vue/es/locale/zh_CN"
import { useTheme } from "@/utils/useTheme"

const { locale } = useI18n({ useScope: "global" })

// 创建对语言变量的引用
const localeLang = ref(locale)

// 监听语言变化并更新
watchEffect(() => {
  localeLang.value = locale.value
})

// 从本地存储或状态管理中获取主题模式
const themeMode = ref<"auto" | "light" | "dark">(
  (localStorage.getItem("theme") as "auto" | "light" | "dark") || "auto"
)

// 使用主题钩子
useTheme(themeMode)
// 保存主题设置
watchEffect(() => {
  localStorage.setItem("theme", themeMode.value)
})

onMounted(() => {})
</script>
