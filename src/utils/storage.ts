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

  // 初始化
  static async initStorage() {
    try {
      const result = await invoke("init_setup", { defaults: defaultConfig })
      console.log("initS", result)
    } catch (error) {
      console.log("initStorage Fail", error)
    }
  }
}

export const SETTING_STORAGE_KEYS = {
  // 主题模式
  THEME_MODE: "themeMode",
  // 语言
  LOCALE: "locale",
  // 自动启动
  AUTO_START: "auto_start",
  // 是否首次启动
  IS_FIRST_START: "is_first_start",
  // 静默启动(启动后不显示主窗口,用快捷键唤起后出现窗口)
  SILENT_START: "silent_start",
  // 主窗口位置（跟随鼠标位置or跟随上次位置）
  WIN_LOCATION: "win_location"
} as const

export enum THEMEMODE_ENUM {
  AUTO = 'auto',
  LIGHT = 'light',
  DARK = 'dark'
}
export type ProxyType = 'unUsed' | 'auto' | 'customize'
// 代理配置类型定义
export interface ProxyConfigType {
  id: number,
  proxyType: string,
  host: String,
  port: number | string,
  username?: String,
  password?: String,
}

// 快捷键类型定义
export interface UrlsType {
  id: number,
  url: string,
  title: string,
  icon?: string,
  isDefault: boolean,
  proxyId?: number,
  shortcut: string
  sortOrder?: number
}

// 默认配置类型定义
export interface DefaultConfigType {
  default: { isMacOS: boolean, isWindows: boolean },
  customConfig: {
    theme: string, // 主题色
    themeMode: 'auto' | 'light' | 'dark', // 主题模式：亮|暗
    locale: 'zh-CN' | 'en-US', // 国际化
    bootUp: boolean, // 是否开机启动
    silentStart: boolean, // 是否静默启动
    winLocation: 'mouseLocation' | 'lastLocation', // 主窗口位置，鼠标位置｜上次位置
  },
  winConfig: {
    globalHotkey: string, // 打开窗口的默认全局快捷键
    openWinRule: 'lastTime' | 'defaultUrl' // 打开窗口的默认行为：打开上次窗口/打开默认窗口url
  }

}

// 默认配置值
export const defaultConfig: DefaultConfigType = {
  default: { isMacOS: false, isWindows: true },
  // 通用配置
  customConfig: {
    theme: '', // 主题色
    themeMode: 'light', // 主题模式：亮|暗
    locale: 'zh-CN', // 国际化
    bootUp: false, // 是否开机启动
    silentStart: false, // 是否静默启动
    winLocation: 'mouseLocation', // 主窗口位置，鼠标位置｜上次位置    
  },
  // 窗口配置
  winConfig: {
    globalHotkey: 'ctrl+C',
    openWinRule: 'lastTime'
  }
}
