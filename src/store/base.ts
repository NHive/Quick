import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useCounterStore = defineStore('base', () => {
  const language = ref('zh-CN')
  function setLanguage(lang: string) {
    language.value = lang
  }

  return { language, setLanguage }
})
