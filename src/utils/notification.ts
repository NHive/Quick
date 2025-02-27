import {
  isPermissionGranted,
  requestPermission,
  sendNotification
} from '@tauri-apps/plugin-notification'

/**
 * 检查并请求通知权限
 * @returns {Promise<boolean>} 是否授予了通知权限
 */
async function checkAndRequestPermission(): Promise<boolean> {
  let permissionGranted = await isPermissionGranted()
  if (!permissionGranted) {
    const permission = await requestPermission()
    permissionGranted = permission === 'granted'
  }
  return permissionGranted
}

/**
 * 发送通知
 * @param {string} title 通知标题
 * @param {string} body 通知内容
 */
export async function notify(title: string, body: string): Promise<void> {
  try {
    const permissionGranted = await checkAndRequestPermission()
    if (permissionGranted) {
      sendNotification({ title, body })
    } else {
      console.warn('Notification permission not granted')
    }
  } catch (error) {
    console.error('Failed to send notification:', error)
  }
}
