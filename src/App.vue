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
const { locale } = useI18n({ useScope: "global" })

// 创建对语言变量的引用
const localeLang = ref(locale)

// 监听语言变化并更新
watchEffect(() => {
  localeLang.value = locale.value
})

onMounted(() => {})
</script>
