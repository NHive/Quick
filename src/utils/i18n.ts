import { createI18n } from "vue-i18n";
import { ref } from "vue";
import { Storage, SETTING_STORAGE_KEYS } from "./storage";
import zhCN from "@/lang/zh-CN.json";
import enUS from "@/lang/en-US.json";
import { invoke } from "@tauri-apps/api/core";

type MessageSchema = typeof zhCN;

// 语言映射表
const LANG_MAP: { [key: string]: string } = {
  "zh-CN": "zh-CN",
  zh: "zh-CN",
  "en-US": "en-US",
  en: "en-US",
};

// 获取系统语言
const getSystemLanguage = (): string => {
  const browserLang = navigator.language;
  return LANG_MAP[browserLang] || "en-US";
};

// 创建 i18n 实例
export const i18n = createI18n<[MessageSchema], "zh-CN" | "en-US">({
  legacy: false,
  locale: "zh-CN",
  fallbackLocale: "en-US",
}) as any;

// 导出语言相关的工具函数
export const useLanguage = () => {
  const currentLocale = ref(i18n.global.locale.value);

  const initLanguage = async () => {
    try {
      const savedLocale = await Storage.get(
        SETTING_STORAGE_KEYS.LOCALE,
        "auto"
      );
      const locale = savedLocale === "auto" ? getSystemLanguage() : savedLocale;
      i18n.global.locale.value = locale;
      currentLocale.value = savedLocale; // 保持用户的选择（auto 或具体语言）
    } catch (error) {
      console.error("Failed to initialize language:", error);
    }
  };

  const updateLanguage = async (value: string) => {
    try {
      const locale = value === "auto" ? getSystemLanguage() : value;

      await invoke("set_locale", { locale: locale });

      i18n.global.locale.value = locale;
      currentLocale.value = value; // 保存用户的选择
      await Storage.set(SETTING_STORAGE_KEYS.LOCALE, value);
    } catch (error) {
      console.error("Failed to update language:", error);
    }
  };

  // 获取可用的语言选项
  const getLanguageOptions = () => [
    { label: i18n.global.t("settings.basic.languageAuto"), value: "auto" },
    { label: "简体中文", value: "zh-CN" },
    { label: "English", value: "en-US" },
  ];

  // 如果需要监听系统语言变化
  const setupSystemLanguageWatch = () => {
    window.matchMedia("(language: *)").addEventListener("change", async () => {
      const savedLocale = await Storage.get(
        SETTING_STORAGE_KEYS.LOCALE,
        "auto"
      );
      if (savedLocale === "auto") {
        const newLocale = getSystemLanguage();
        i18n.global.locale.value = newLocale;
      }
    });
  };

  return {
    currentLocale,
    initLanguage,
    updateLanguage,
    getLanguageOptions,
    setupSystemLanguageWatch,
  };
};
