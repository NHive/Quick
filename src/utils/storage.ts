import { invoke } from "@tauri-apps/api/core"

export interface StorageOptions {
  defaultValue?: any
}

export class Storage {
  static async get<T>(key: string, defaultValue: T): Promise<T> {
    try {
      const result = await invoke<T>("get_setup", { key })

      if (result === null || result === undefined) {
        await this.set(key, defaultValue)
        return defaultValue
      }

      return result
    } catch (error) {
      console.error(`Failed to get config for ${key}:`, error)
      await this.set(key, defaultValue)
      return defaultValue
    }
  }

  static async set<T>(key: string, value: T): Promise<void> {
    try {
      await invoke("set_setup", {
        key,
        value
      })
    } catch (error) {
      console.error(`Failed to set config for ${key}:`, error)
      throw error
    }
  }
}

export const SETTING_STORAGE_KEYS = {
  // 监听剪切板
  ENABLE_MONITOR: "enable_monitor",
  // 自动粘贴
  AUTO_INSERT: "auto_insert",
  // 默认设置
  DEFAULT_SET: "default_set",
  // 文件同步
  FILE_SYNC: "file_sync",
  // 最大文件大小
  MAX_FILE_SIZE: "max_file_size",
  // 关键词过滤
  KEYWORD_FILTER: "keyword_filter",
  // 主题模式
  THEME_MODE: "themeMode",
  // 语言
  LOCALE: "locale",
  // 历史记录保存时间
  HISTORY_RETENTION_DAYS: "history_save_time_days",
  // 自动启动
  AUTO_START: "auto_start",
  // 是否首次启动
  IS_FIRST_START: "is_first_start",
  // 是否将图片文件解析为图片
  IMG_FILE_CONV_TO_IMG: "img_file_conv_to_img",
  // 是否将图片有损压缩保存(统一保存为jpg)
  LOSSY_COMPRESSED_PICTURE: "lossy_compressed_picture",
  // 静默启动(启动后不显示主窗口,用快捷键唤起后出现窗口)
  SILENT_START: "silent_start",
  // 粘贴后移动记录至最前
  PASTE_AND_TOP: "paste_and_top"
} as const
