import { invoke } from "@tauri-apps/api/core";
import { platform, type } from "@tauri-apps/plugin-os";
import { listen } from "@tauri-apps/api/event";
import {
  register,
  isRegistered,
  unregister,
  unregisterAll,
  type ShortcutEvent,
} from "@tauri-apps/plugin-global-shortcut";
import { useLanguage } from "../utils/i18n";

// 获取持久化数据
export const getLocalShortcut = async (
  key: string,
  winDefaultVal: string,
  macDefaultVal: string
) => {
  let defaultVal = "";
  const osType = await type();
  if (osType === "macos") {
    defaultVal = macDefaultVal; // macOS 默认值
  } else {
    defaultVal = winDefaultVal; // 其他系统默认值
  }
  try {
    const response = await invoke<string>("get_setup", { key });
    const data = response || defaultVal;
    return data;
  } catch (error) {
    console.error(`Failed to get config for ${key}:`, error);
    return defaultVal;
  }
};
// 主窗口显示—快捷键注册事件
export const mainWinRegisterFun = (event: ShortcutEvent) => {
  // 按下快捷键时触发
  // Pressed为按下，Released为松开
  if (event.state === "Pressed") {
    invoke("toggle_clipboard_window_visibility");
  }
};

// 注册全局快捷键
const registerShortcut = async (shortcut?: string) => {
  try {
    // 获取当前配置的快捷键
    const shortcutKey =
      shortcut ||
      (await getLocalShortcut("showMainWinHotKey", "option+v", "alt+v"));
    // 注册快捷键
    const hkIsRegistered = await isRegistered(shortcutKey);
    if (!hkIsRegistered) {
      await register(shortcutKey, mainWinRegisterFun);
    }
  } catch (error) {
    console.error("Failed to register shortcut:", error);
  }
};
// 解绑全局快捷键
const unRegisterShortcut = async (shortcut?: string) => {
  try {
    // 获取当前配置的快捷键
    const shortcutKey =
      shortcut ||
      (await getLocalShortcut("showMainWinHotKey", "option+v", "alt+v"));
    // 注册快捷键
    const hkIsRegistered = await isRegistered(shortcutKey);
    if (hkIsRegistered) {
      await unregister(shortcutKey);
    }
  } catch (error) {
    console.error("Failed to unregister shortcut:", error);
  }
};

// 解绑所有快捷键
const unRegisterAllShortcut = async () => {
  await unregisterAll();
};

// 监听事件，监听快捷键变更事件、解绑快捷键事件
const shortcutListener = async () => {
  try {
    // 监听快捷键变更事件
    await listen("register-shortcut", (payload: Record<string, any>) => {
      console.log(payload);
      registerShortcut(payload.shortcut ?? "");
    });
    await listen("unRegister-shortcut", (payload: Record<string, any>) => {
      console.log(payload);
      unRegisterShortcut(payload.shortcut ?? "");
    });
    await listen("unRegisterAll-shortcut", () => {
      unRegisterAllShortcut();
    });
  } catch (error) {
    console.error("Failed to setup shortcut listeners:", error);
  }
};

const initializeApp = async () => {
  try {
    // 初始化语言
    const { updateLanguage } = useLanguage();
    // 检测操作系统
    const currentPlatform = await platform();
    const isMacOS = currentPlatform === "macos";
    const isWindows = currentPlatform === "windows";
  } catch (error) {
    console.error("Failed to initialize app settings:", error);
  }
};

export const initialization = async () => {
  try {
    // 先解绑所有快捷键，确保没有残留的注册
    await unregisterAll();
    // 重新注册快捷键
    await registerShortcut();
    // 初始化应用
    await initializeApp();
    // 监听快捷键变更事件
    await shortcutListener();
  } catch (error) {
    console.error("Failed to initialize app:", error);
  }
};

export const initHome = async () => {
  await invoke("show_window");
};
