<template>
  <div class="settings-container">
    <!-- Show simplified login form when not logged in -->
    <template v-if="!userInfo.is_login">
      <a-card class="settings-card">
        <div class="account-status">
          <div class="status-icon">
            <UserOutlined style="font-size: 24px" />
          </div>
          <div class="status-text">
            <div class="status-title">
              <span class="mainTitle">{{ t("settings.account.notLoggedIn") }}</span>
              <span class="secondTitle">（{{ t("settings.account.loginTip") }}）</span>
            </div>
            <a-button class="status-desc" type="link"
              >{{ t("settings.account.memberTips") }} ></a-button
            >
          </div>
        </div>
        <a-button size="large" type="primary" class="login-button" @click="onFinish">
          {{ t("settings.account.login") }}
        </a-button>
      </a-card>

      <a-card class="settings-card">
        <div class="settings-section-title">{{ t("settings.account.versions") }}</div>
        <div class="version-list">
          <div
            :class="`version-item ${item.key === 'premium' ? 'premium' : ''}`"
            v-for="item in featuresData"
            :key="item.key"
          >
            <div class="version-header">
              <div class="version-name">{{ item.title }}</div>
            </div>
            <div class="version-features">
              <div class="feature-item" v-for="child in item.data" :key="child.key">
                <span class="featureIcon">
                  <SnippetsOutlined v-if="child.iconType === 'copy'" />
                  <HistoryOutlined v-if="child.iconType === 'history'" />
                  <FormOutlined v-if="child.iconType === 'hotkey'" />
                  <UserAddOutlined v-if="child.iconType === 'member'" />
                  <SendOutlined v-if="child.iconType === 'sync'" />
                  <DeliveredProcedureOutlined v-if="child.iconType === 'data'" />
                  <ProfileOutlined v-if="child.iconType === 'label'" />
                </span>
                <span>{{ child.value }}</span>
              </div>
            </div>
          </div>
        </div>
      </a-card>
    </template>
    <!-- Show account settings when logged in -->
    <template v-else>
      <!-- User Profile Section -->
      <div v-if="userInfo.is_login" class="profile-section">
        <div class="avatar-section">
          <a-upload
            v-model:file-list="fileList"
            class="avatar-upload"
            :show-upload-list="false"
            :accept="acceptImgType"
            :before-upload="beforeUpload"
            :custom-request="handleUpload"
            @preview="handlePreview"
          >
            <div class="avatar-wrapper">
              <template v-if="loading">
                <div class="avatar-loading">
                  <a-spin />
                </div>
              </template>
              <template v-else>
                <img
                  v-if="userInfo.avatar"
                  :src="userInfo.avatar"
                  class="avatar-image"
                  alt="用户头像"
                />
                <img
                  v-else
                  class="defaultImage"
                  src="@/assets/image/icon.png"
                  alt="defaultAvatar"
                />

                <!-- <div v-else class="avatar-placeholder">
                  {{ getAvatarText(userInfo.nickname) }}
                </div> -->
                <div class="avatar-overlay">
                  <camera-outlined />
                  <span>{{ t("settings.account.updateAvatar") }}</span>
                </div>
              </template>
            </div>
          </a-upload>
          <!-- VIP标识 -->
          <div v-if="isVip" class="vip-badge" v-html="icons.vip"></div>
          <!-- Avatar Preview Modal -->
          <a-modal
            v-model:open="previewVisible"
            :title="t('settings.account.preview')"
            :footer="null"
            @cancel="handlePreviewCancel"
          >
            <img :src="previewImage" style="width: 100%" alt="" />
          </a-modal>
        </div>

        <div class="user-details">
          <!-- Nickname -->
          <div class="detail-item">
            <div class="detail-header">
              <span class="detail-label">{{ t("settings.account.nickname") }}</span>
              <a-button type="link" class="edit-button" @click="startEditing('nickname')">
                <edit-outlined />
              </a-button>
            </div>
            <div class="detail-content">
              <template v-if="editingField === 'nickname'">
                <a-input
                  v-model:value="editingValue"
                  @pressEnter="handleSave"
                  @blur="handleSave"
                  :maxLength="20"
                  ref="inputRef"
                />
              </template>
              <template v-else>
                <span>{{ userInfo.nickname }}</span>
              </template>
            </div>
          </div>

          <!-- Email -->
          <div class="detail-item">
            <div class="detail-label">{{ t("settings.account.email") }}</div>
            <div class="detail-content">
              <span>{{ userInfo.email }}</span>
              <a-tag v-if="userInfo.email" color="success">{{
                t("settings.account.verified")
              }}</a-tag>
            </div>
          </div>

          <!-- Phone -->
          <div class="detail-item">
            <div class="detail-label">{{ t("settings.account.phone") }}</div>
            <div class="detail-content">
              <span>{{ formatPhone(userInfo.phone) }}</span>
            </div>
          </div>

          <!-- Device Name -->
          <div class="detail-item">
            <div class="detail-header">
              <span class="detail-label">{{ t("settings.account.deviceName") }}</span>
              <a-button type="link" class="edit-button" @click="startEditing('device_name')">
                <edit-outlined />
              </a-button>
            </div>
            <div class="detail-content">
              <template v-if="editingField === 'deviceName'">
                <a-input
                  v-model:value="editingValue"
                  @pressEnter="handleSave"
                  @blur="handleSave"
                  :maxLength="30"
                  ref="inputRef"
                />
              </template>
              <template v-else>
                <span>{{ userInfo.device_name }}</span>
              </template>
            </div>
          </div>

          <!-- Add logout button at the end of user-details -->
          <div class="detail-item">
            <a-button danger class="logout-button" @click="handleLogout">
              {{ t("settings.account.logout") }}
            </a-button>
          </div>
        </div>
      </div>
    </template>
    <!-- </div> -->
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, nextTick, onBeforeUnmount } from "vue"
import { invoke } from "@tauri-apps/api/core"
import { message } from "ant-design-vue"
import { useI18n } from "vue-i18n"
import {
  UserOutlined,
  EditOutlined,
  CameraOutlined,
  SnippetsOutlined,
  HistoryOutlined,
  FormOutlined,
  UserAddOutlined,
  SendOutlined,
  DeliveredProcedureOutlined,
  ProfileOutlined
} from "@ant-design/icons-vue"
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow"
import { icons } from "@/utils/svg"

export interface UserInfo {
  nickname: string
  email: string
  avatar: string
  phone: string
  device_name: string
  is_login: boolean
  device_id: string
}

export interface UploadResponse {
  url: string
  status: "success" | "error"
  message?: string
}

const { t } = useI18n()

// State
const loading = ref(false)
const userInfo = ref<UserInfo>({
  nickname: "",
  email: "",
  avatar: "",
  phone: "",
  device_name: "",
  is_login: false,
  device_id: ""
})
// TODO
const isVip = ref<boolean>(false)

const appWebview = getCurrentWebviewWindow()
const unListener = ref<any>(null)
// Upload related
const fileList = ref([])
const acceptImgType = ".jpg,.jpeg,.png,.gif" as string
const previewVisible = ref(false)
const previewImage = ref("")

// Edit related
const editingField = ref<string | null>(null)
const editingValue = ref("")
const inputRef = ref<HTMLInputElement | null>(null)

// const formState = reactive({
//   username: "",
//   password: "",
//   phone: "",
//   captcha: ""
// })
// const disabled = ref(false)
// const captchaDisabled = ref(false)
const featuresData = [
  {
    title: t("settings.account.offlineVersion"),
    key: "offline",
    data: [
      { iconType: "copy", value: t("feature.clipboard"), key: "clipboard" },
      { iconType: "hotkey", value: t("feature.hotkey"), key: "hotkey" },
      { iconType: "history", value: t("feature.history"), key: "history" }
    ]
  },
  {
    title: t("settings.account.freeVersion"),
    key: "free",
    data: [
      { iconType: "copy", value: t("feature.clipboard"), key: "clipboard" },
      { iconType: "hotkey", value: t("feature.hotkey"), key: "hotkey" },
      { iconType: "history", value: t("feature.history"), key: "history" },
      { iconType: "member", value: t("feature.membership"), key: "membership" }
    ]
  },
  {
    title: t("settings.account.premiumVersion"),
    key: "premium",
    data: [
      { iconType: "copy", value: t("feature.clipboard"), key: "clipboard" },
      { iconType: "hotkey", value: t("feature.hotkey"), key: "hotkey" },
      { iconType: "history", value: t("feature.history"), key: "history" },
      { iconType: "sync", value: t("feature.sync"), key: "sync" },
      { iconType: "data", value: t("feature.dataBackup"), key: "data" },
      { iconType: "label", value: t("feature.label"), key: "label" }
    ]
  }
]

// Methods
const getUserInfo = async () => {
  try {
    loading.value = true
    const data = await invoke<UserInfo>("get_hide_config_user_info")
    console.log("getUserInfo", data, data.is_login)
    userInfo.value = data
    console.log(userInfo.value)
  } catch (error) {
    console.error("Failed to fetch user info:", error)
    message.error(t("settings.account.fetchError"))
    userInfo.value.is_login = false
  } finally {
    loading.value = false
  }
}

// const getAvatarText = (name: string): string => (name ? name.charAt(0).toUpperCase() : "U")

const formatPhone = (phone: string): string =>
  phone ? phone.replace(/(\d{3})\d{4}(\d{4})/, "$1****$2") : "-"

const beforeUpload = (file: File) => {
  const isValidType = acceptImgType.includes(file.type)
  const isLt2M = file.size / 1024 / 1024 < 2

  if (!isValidType) {
    message.error(t("settings.account.invalidImageType"))
  }
  if (!isLt2M) {
    message.error(t("settings.account.imageTooLarge"))
  }

  return isValidType && isLt2M
}

const handleUpload = async ({ file, onSuccess, onError }: any) => {
  try {
    loading.value = true
    // 这里添加实际的上传逻辑
    const formData = new FormData()
    formData.append("file", file)

    // 模拟上传
    await new Promise((resolve) => {
      setTimeout(resolve, 1000)
    })

    onSuccess()
    await getUserInfo() // 刷新用户信息
    message.success(t("settings.account.avatarUpdateSuccess"))
  } catch (error) {
    onError()
    message.error(t("settings.account.avatarUpdateError"))
  } finally {
    loading.value = false
  }
}

const startEditing = (field: "nickname" | "device_name") => {
  editingField.value = field
  editingValue.value = userInfo.value[field] || ""
  nextTick(() => {
    inputRef.value?.focus()
  })
}

const handleSave = async () => {
  if (!editingField.value) return

  try {
    // 这里添实际的保存逻辑
    await invoke("update_user_info", {
      field: editingField.value,
      value: editingValue.value
    })

    userInfo.value = {
      ...userInfo.value,
      [editingField.value]: editingValue.value
    }

    message.success(t("settings.account.updateSuccess"))
  } catch (error) {
    message.error(t("settings.account.updateError"))
  } finally {
    editingField.value = null
  }
}
// TODO
// const onFinish = async () => {
//   disabled.value = true
//   const { username, password, phone, captcha } = formState

//   try {
//     if (activeTab.value === 'captcha') {
//       await invoke('login_from_sms', { phone, code: captcha })
//     } else {
//       await invoke('login', { username, password })
//     }
//     await getUserInfo() // Refresh user info after successful login
//   } catch (e) {
//     if (e instanceof Error) {
//       message.error(e.message)
//     } else {
//       message.error('登录失败')
//     }
//   } finally {
//     disabled.value = false
//   }
// }

// 打开置窗口
const onFinish = async () => {
  try {
    await invoke("open_login_window")
  } catch (error) {
    console.error("open window error:", error)
  }
}

// const sendCaptcha = async () => {
//   captchaDisabled.value = true
//   const { phone } = formState

//   try {
//     await invoke("send_sms", { countryCode: "86", phone })
//     message.success("验证码已发送")
//     setTimeout(() => {
//       captchaDisabled.value = false
//     }, 30000)
//   } catch (e) {
//     captchaDisabled.value = false
//     if (e instanceof Error) {
//       message.error(e.message)
//     } else {
//       message.error("发送验证码失败")
//     }
//   }
// }

// Add these methods for handling avatar preview
const handlePreview = (file: any) => {
  previewImage.value = file.url || file.thumbUrl
  previewVisible.value = true
}

const handlePreviewCancel = () => {
  previewVisible.value = false
}

const handleLogout = async () => {
  try {
    await invoke("logout_device", { deviceId: String(userInfo.value.device_id) })
    await getUserInfo()
    message.success(t("settings.account.logoutSuccess"))
  } catch (error) {
    console.error(error)
    message.error(t("settings.account.logoutError"))
  }
}

// Lifecycle
onMounted(async () => {
  getUserInfo()
  unListener.value = await appWebview.listen("updateUserInfo", getUserInfo)
})

onBeforeUnmount(() => {
  unListener.value()
})
</script>

<style lang="scss" scoped>
.settings-container {
  width: 100%;
  margin: 0 auto;
  color: var(--text-color);
}

.settings-section {
  background: var(--bg-color);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 24px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  transition: all 0.3s ease;

  .settings-header {
    margin-bottom: 24px;
  }

  .settings-section-title {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--text-color);
    opacity: 1;

    .section-icon {
      font-size: 24px;
      color: var(--text-color);
    }
  }

  :deep(.ant-tabs) {
    .ant-tabs-nav {
      margin-bottom: 32px;

      &::before {
        display: none;
      }

      .ant-tabs-tab {
        padding: 12px 24px;
        margin: 0 8px;
        transition: all 0.3s ease;

        &:hover {
          color: #0290fe;
        }

        &.ant-tabs-tab-active .ant-tabs-tab-btn {
          color: #0290fe;
          font-weight: 500;
        }
      }

      .ant-tabs-ink-bar {
        height: 3px;
        border-radius: 3px;
        background: #0290fe;
      }
    }
  }

  :deep(.ant-form) {
    .ant-input-affix-wrapper,
    .ant-input-password {
      height: 40px;
      border-radius: 6px;
      border: 1px solid rgba(0, 0, 0, 0.1);
      transition: all 0.3s ease;
      background: rgba(245, 245, 247, 0.05);

      &:hover {
        transform: translateY(-1px);
        border-color: #0290fe;
      }

      &:focus,
      &-focused {
        border-color: #0290fe;
        box-shadow: 0 0 0 2px rgba(2, 144, 254, 0.2);
      }

      .anticon {
        color: #0290fe;
        font-size: 16px;
      }

      input {
        font-size: 14px;
        background: transparent;

        &::placeholder {
          color: rgba(0, 0, 0, 0.45);
        }
      }
    }

    // 统一按钮样式
    .submit-button {
      width: 100%;
      height: 40px;
      background: #0290fe;
      border: none;
      color: #f5f5f5;
      font-weight: 500;
      border-radius: 6px;
      margin-top: 24px;
      transition: all 0.3s ease;

      &:hover {
        background: lighten(#0290fe, 5%);
        transform: translateY(-1px);
      }

      &:active {
        transform: translateY(0);
        background: darken(#0290fe, 5%);
      }

      &.ant-btn-loading {
        opacity: 0.8;
      }
    }

    // 验证码输入框和按钮的布局
    .captcha-row {
      display: flex;
      gap: 12px;
      align-items: center; // 确保垂直居中对齐

      .ant-input-affix-wrapper {
        flex: 1; // 输入框占据剩余空间
      }

      .captcha-btn {
        min-width: 120px;
        height: 40px; // 与输入框同高
        border-color: #0290fe;
        color: #0290fe;
        white-space: nowrap; // 防止文字换行
        padding: 0 16px;
        border-radius: 6px;
        transition: all 0.3s ease;

        &:hover {
          color: darken(#0290fe, 5%);
          border-color: darken(#0290fe, 5%);
          transform: translateY(-1px);
        }

        &:active {
          transform: translateY(0);
        }

        &:disabled {
          background: #f5f5f5;
          border-color: #d9d9d9;
          color: rgba(0, 0, 0, 0.25);
          transform: none;
        }
      }
    }

    // 密码登录表单特定样式
    [key="password"] {
      .ant-input-affix-wrapper,
      .ant-input-password {
        height: 40px;
        border-radius: 6px;
        border: 1px solid rgba(0, 0, 0, 0.1);
        transition: all 0.3s ease;
        background: rgba(245, 245, 247, 0.05);

        &:hover {
          transform: translateY(-1px);
          border-color: #0290fe;
        }

        &:focus,
        &-focused {
          border-color: #0290fe;
          box-shadow: 0 0 0 2px rgba(2, 144, 254, 0.2);
        }

        .anticon {
          color: #0290fe;
          font-size: 16px;
        }

        input {
          font-size: 14px;
          background: transparent;

          &::placeholder {
            color: rgba(0, 0, 0, 0.45);
          }
        }
      }

      // 密码登录的提交按钮
      .submit-button {
        width: 100%;
        height: 40px;
        background: #0290fe;
        border: none;
        color: #000000;
        font-weight: 500;
        border-radius: 6px;
        margin-top: 24px;
        transition: all 0.3s ease;

        &:hover {
          background: lighten(#0290fe, 5%);
          transform: translateY(-1px);
        }

        &:active {
          transform: translateY(0);
          background: darken(#0290fe, 5%);
        }

        &.ant-btn-loading {
          opacity: 0.8;
        }
      }
    }
  }
}

// 深色模式适配
:root[data-theme="dark"] {
  .settings-section {
    background: #2c2c2e;

    :deep(.ant-form) {
      .ant-input-affix-wrapper,
      .ant-input-password {
        background: rgba(255, 255, 255, 0.04);
        border-color: rgba(255, 255, 255, 0.1);

        &:hover,
        &:focus,
        &-focused {
          border-color: #0290fe;
          background: rgba(255, 255, 255, 0.08);
        }

        input {
          color: rgba(255, 255, 255, 0.85);

          &::placeholder {
            color: rgba(255, 255, 255, 0.3);
          }
        }
      }

      .submit-button {
        &:hover {
          background: lighten(#0290fe, 5%);
        }
      }

      .captcha-btn:disabled {
        background: rgba(255, 255, 255, 0.04);
        border-color: rgba(255, 255, 255, 0.1);
        color: rgba(255, 255, 255, 0.3);
      }

      // 深色模式下密码录的适配
      [key="password"] {
        .ant-input-affix-wrapper,
        .ant-input-password {
          background: rgba(255, 255, 255, 0.04);
          border-color: rgba(255, 255, 255, 0.1);

          &:hover,
          &:focus,
          &-focused {
            border-color: #0290fe;
            background: rgba(255, 255, 255, 0.08);
          }

          input {
            color: rgba(255, 255, 255, 0.85);

            &::placeholder {
              color: rgba(255, 255, 255, 0.3);
            }
          }
        }

        .submit-button {
          &:hover {
            background: lighten(#0290fe, 5%);
          }
        }
      }
    }
  }
}

.profile-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 24px;

  .avatar-section {
    position: relative;
    .avatar-wrapper {
      width: 100px;
      height: 100px;
      border-radius: 50%;
      overflow: hidden;
      position: relative;
      cursor: pointer;
      background: #f5f5f5;
      display: flex;
      align-items: center;
      justify-content: center;
      border: 2px solid #e8e8e8;
      transition: all 0.3s ease;

      &:hover {
        border-color: #0290fe;

        .avatar-overlay {
          opacity: 1;
        }
      }

      .avatar-image {
        width: 100%;
        height: 100%;
        object-fit: cover;
      }

      .defaultImage {
        width: 90%;
        height: 90%;
      }

      .avatar-placeholder {
        font-size: 36px;
        color: #999;
      }

      .avatar-overlay {
        position: absolute;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        flex-direction: column;
        align-items: center;
        justify-content: center;
        opacity: 0;
        transition: opacity 0.3s;
        color: white;
        border-radius: 50%;

        .anticon {
          font-size: 20px;
          margin-bottom: 4px;
        }

        span {
          font-size: 12px;
        }
      }
    }
    .vip-badge {
      position: absolute;
      bottom: 0px;
      right: 0px;
      width: 28px;
      height: 28px;
      display: flex;
      align-items: center;
      justify-content: center;
      z-index: 2;
    }
  }

  .user-details {
    width: 100%;
    max-width: 400px;

    .detail-item {
      margin-bottom: 16px;
      padding-bottom: 12px;
      border-bottom: 1px solid rgba(0, 0, 0, 0.06);

      &:last-child {
        border-bottom: none;
        margin-bottom: 0;
      }

      .detail-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-bottom: 8px;
      }

      .detail-label {
        font-size: 13px;
        color: rgba(0, 0, 0, 0.45);
        margin-bottom: 8px;
      }

      .detail-content {
        font-size: 14px;
        display: flex;
        align-items: center;
        gap: 8px;

        .ant-tag {
          margin: 0;
        }

        .ant-input {
          margin: -4px 0;
        }
      }

      .edit-button {
        padding: 4px 8px;
        color: #0290fe;

        &:hover {
          color: darken(#0290fe, 10%);
          background: rgba(2, 144, 254, 0.1);
        }
      }
    }
  }
}

// 深色模式适配
:root[data-theme="dark"] {
  .profile-section {
    .avatar-wrapper {
      background: #1f1f1f;
      border-color: #434343;
    }

    .user-details {
      .detail-item {
        border-bottom-color: rgba(255, 255, 255, 0.08);

        .detail-label {
          color: rgba(255, 255, 255, 0.45);
        }

        .detail-content {
          color: rgba(255, 255, 255, 0.85);
        }
      }
    }
  }
}

.logout-button {
  width: 100%;
  height: 40px;
  border-radius: 6px;
  font-weight: 500;
  transition: all 0.3s ease;
  margin-top: 16px;

  &:hover {
    transform: translateY(-1px);
  }

  &:active {
    transform: translateY(0);
  }
}

// Add theme variables
:root {
  --text-color: #000000;
  --text-secondary: #666666;
  --bg-color: rgba(255, 255, 255, 0.98);
  --card-bg: rgba(255, 255, 255, 0.9);
  --item-bg: rgba(255, 255, 255, 0.8);
  --item-hover-bg: rgba(255, 255, 255, 0.95);
  --border-color: rgba(0, 0, 0, 0.06);
}

:root[data-theme="dark"] {
  --text-color: #ffffff;
  --text-secondary: #b0b0b0;
  --bg-color: rgba(28, 28, 30, 0.98);
  --card-bg: rgba(44, 44, 46, 0.98);
  --item-bg: rgba(44, 44, 46, 0.95);
  --item-hover-bg: rgba(58, 58, 60, 0.98);
  --border-color: rgba(255, 255, 255, 0.12);
}

// Add text shadow effects
.settings-section-title,
.detail-label,
.detail-content {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

:root[data-theme="dark"] {
  .settings-section-title {
    color: #ffffff;

    .section-icon {
      color: #ffffff;
    }
  }

  .detail-label {
    color: #b0b0b0;
  }

  .detail-content {
    color: #f0f0f0;
  }
}

.settings-card {
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(10px);
  border-radius: 20px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  margin-bottom: 18px;
  transition: all 0.3s ease;

  :root[data-theme="dark"] & {
    background: var(--bg-color);
  }
}

:deep(.ant-card-body) {
  padding: 18px;
}

.settings-section-title {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 14px;
  color: #1d1d1f;

  :root[data-theme="dark"] & {
    color: #ffffff;
  }
}

.account-status {
  display: flex;
  align-items: center;
  margin-bottom: 12px;

  .status-icon {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background: #f5f5f7;
    display: flex;
    align-items: center;
    justify-content: center;
    margin-right: 16px;
    color: #86868b;
  }

  .status-text {
    .status-title {
      font-size: 16px;
      font-weight: 500;
      margin-bottom: 4px;
      color: #1d1d1f;

      .secondTitle {
        font-weight: normal;
        font-size: 14px;
        color: #86868b;
      }
    }

    .status-desc {
      margin-top: 8px;
      font-size: 14px;
      color: #f8da52;
      padding: 0;
      // color: #86868b;
    }
  }
}

.login-button {
  width: 100%;
  color: #232222;
}

.version-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.version-item {
  position: relative;
  padding: 22px 10px 10px 10px;
  border-radius: 12px;
  background: #f5f5f7;
  transition: all 0.3s ease;

  &.premium {
    background: #f5f0ff;
  }

  .version-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 12px;
    position: absolute;
    top: 0;
    left: 0;
    border-radius: 12px 0 12px 0;
    background-color: #f8da52;
    padding: 4px 8px;

    .version-name {
      font-weight: 500;
      font-size: 14px;
    }

    .version-tag {
      font-size: 12px;
      padding: 2px 8px;
      border-radius: 4px;
      background: #4784ec;
      color: white;
    }
  }

  .version-features {
    width: 100%;
    display: flex;
    justify-content: flex-start;
    align-items: center;
    flex-wrap: wrap;
    gap: 8px;

    .feature-item {
      display: flex;
      flex-direction: column;
      justify-content: flex-start;
      align-items: center;
      gap: 2px;
      font-size: 12px;
      color: #1d1d1f;
      flex: 0 1 32%;

      :deep(svg) {
        font-size: 24px;
        color: #f8da52;
      }
    }
  }
}
</style>
