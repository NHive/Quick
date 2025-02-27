import { defineStore } from 'pinia'
import { type } from '@tauri-apps/plugin-os'

interface ShortcutState {
  shortcutKey: string
}

export const useShortcutStore = defineStore('shortcut', {
  state: (): ShortcutState => ({ shortcutKey: '' }),
  actions: {
    setShortcutKey(newShortcutKey: string) {
      this.shortcutKey = newShortcutKey
    },
    async detectAndSetDefaultShortcut() {
      const osType = await type()

      if (osType === 'macos') {
        this.shortcutKey = 'option+v' // macOS 默认快捷键
      } else {
        this.shortcutKey = 'alt+v' // 其他系统默认快捷键
      }
    }
  }
})
