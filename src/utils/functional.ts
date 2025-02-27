import { i18n } from './i18n'

const { t } = i18n.global

// html背景提取器
export const extractBackgroundColor = (html: string | undefined): string | null => {
  if (!html) return null
  const regex =
    /background-color\s*:\s*(#[0-9a-fA-F]{3,6}|rgb\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*\)|rgba\(\s*\d+\s*,\s*\d+\s*,\s*\d+\s*,\s*\d+(\.\d+)?\s*\))/i
  const match = html.match(regex)
  return match ? match[1] : null
}

// 格式化时间
export const formatTime = (time: string): string => {
  const date = new Date(time)
  const now = new Date()
  const localDate = new Date(date.getTime() - date.getTimezoneOffset() * 60000)
  const diff = now.getTime() - localDate.getTime()
  const minutes = Math.floor(diff / 60000)
  const hours = Math.floor(diff / 3600000)
  const days = Math.floor(diff / 86400000)

  if (days > 0) return t('time.daysAgo', { days })
  if (hours > 0) return t('time.hoursAgo', { hours })
  if (minutes > 0) return t('time.minutesAgo', { minutes })
  return t('time.justNow')
}

// 防抖函数
export function debounce<T extends (...args: any[]) => void>(
  func: T,
  wait: number
): (this: ThisParameterType<T>, ...args: Parameters<T>) => void {
  let timeout: ReturnType<typeof setTimeout>
  return function (this: ThisParameterType<T>, ...args: Parameters<T>) {
    clearTimeout(timeout)
    timeout = setTimeout(() => {
      func.apply(this, args)
    }, wait)
  }
}


// 在 script setup 部分添加 formatSize 函数
export const formatSize = (size: number, dataType: number): string => {
  switch (dataType) {
    case 0: // DATA_TYPE_TEXT
      return t("clipboard.content.textSize", { count: size })
    case 1: // DATA_TYPE_IMAGE
    case 2: // DATA_TYPE_FILE
      const sizeInBytes = size as number
      if (sizeInBytes >= 1_073_741_824) {
        return t("clipboard.content.sizeGB", { size: (sizeInBytes / 1_073_741_824).toFixed(2) })
      } else if (sizeInBytes >= 1_048_576) {
        return t("clipboard.content.sizeMB", { size: (sizeInBytes / 1_048_576).toFixed(2) })
      } else {
        return t("clipboard.content.sizeKB", { size: (sizeInBytes / 1024).toFixed(2) })
      }
    case 3: // DATA_TYPE_LINK
      return t("clipboard.content.fileLink")
    default:
      return t("clipboard.content.unknownType")
  }
}
