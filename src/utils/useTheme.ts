import { watch, onUnmounted } from 'vue'
import type { Ref } from 'vue'

export function useTheme(themeMode: Ref<'auto' | 'light' | 'dark'>) {
  const setTheme = (mode: 'auto' | 'light' | 'dark') => {
    if (mode === 'auto') {
      const isDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      document.documentElement.setAttribute('data-theme', isDark ? 'dark' : 'light')
    } else {
      document.documentElement.setAttribute('data-theme', mode)
    }
  }

  // 监听主题变化
  // watch(themeMode, (newValue) => {
  //   setTheme(newValue)
  // })

  const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
  const handleThemeChange = (event: MediaQueryListEvent | MediaQueryList) => {
    if (themeMode.value === 'auto') {
      document.documentElement.setAttribute('data-theme', event.matches ? 'dark' : 'light')
    }
  }

  mediaQuery.addEventListener('change', handleThemeChange)

  onUnmounted(() => {
    mediaQuery.removeEventListener('change', handleThemeChange)
  })

  return { setTheme }
}
