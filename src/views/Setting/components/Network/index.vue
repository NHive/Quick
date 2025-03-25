<template>
  <div class="settings-container">
    <div class="head">
      <div class="settings-section-title">
        {{ t('settings.menu.proxy') }}
      </div>
      <a-button class="addBtn" @click="openModal()">{{
        t('communal.add')
      }}</a-button>
    </div>
    <!-- 窗口设置 -->
    <div class="settings-section">
      <div class="settings-options" v-if="proxyData.length">
        <div class="proxy-title">
          <div class="title-item">{{ t('settings.basic.proxyType') }}</div>
          <div class="title-item urlItem">
            {{ t('settings.basic.proxyAddress') }}
          </div>
          <div class="title-item">{{ t('settings.basic.port') }}</div>
          <div class="title-item operate"></div>
        </div>
        <div class="proxy-box" v-for="(item, index) in proxyData">
          <div class="proxy-item urlItem">{{ item.proxyType }}</div>
          <div class="proxy-item">{{ item.host }}</div>
          <div class="proxy-item">{{ item.port }}</div>
          <div class="proxy-item operate">
            <EditOutlined class="operate-icon" @click="openModal(item.id)" />
            <DeleteOutlined @click="deleteConfirm(item.id)" />
          </div>
        </div>
      </div>
      <div v-else class="empty">{{ t('tips.empty') }}</div>
    </div>
    <a-modal
      :open="showProxyModal"
      :title="proxyModalTitle"
      @ok="onSubmit"
      @cancel="closeModal"
    >
      <a-form
        ref="proxyFormRef"
        :model="proxyFormState"
        :labelCol="{ span: 5 }"
      >
        <a-form-item name="proxyType" :label="t('settings.basic.proxyType')">
          <a-select
            v-model:value="proxyFormState.proxyType"
            :options="proxyTypeOptions"
          ></a-select>
        </a-form-item>
        <a-form-item
          name="host"
          :label="t('settings.basic.proxyAddress')"
          required
          :rules="[
            { required: true, message: t('placeholder.proxyAddress') },
            {
              pattern:
                /(?:([a-zA-Z0-9-]+\.)+[a-zA-Z0-9-]+|localhost|\d{1,3}(\.\d{1,3}){3})/,
              message: t('tips.hostWrongFormat'),
            },
          ]"
        >
          <a-input
            v-model:value="proxyFormState.host"
            :placeholder="t('placeholder.proxyAddress')"
          ></a-input>
        </a-form-item>
        <a-form-item
          name="port"
          :label="t('settings.basic.port')"
          required
          :rules="[
            { required: true, message: t('placeholder.port') },
            {
              pattern: /^\d{1,5}$/,
              message: t('tips.portWrongFormat'),
            },
            {
              max: 65535,
              message: t('tips.portMax', { max: 65535 }),
            },
          ]"
        >
          <a-input
            v-model:value="proxyFormState.port"
            :placeholder="t('placeholder.port')"
          ></a-input>
        </a-form-item>
        <a-form-item name="username" :label="t('settings.basic.userName')">
          <a-input
            v-model:value="proxyFormState.username"
            :placeholder="t('settings.login.enterName')"
          ></a-input>
        </a-form-item>
        <a-form-item name="password" :label="t('settings.basic.pwd')">
          <a-input
            type="password"
            v-model:value="proxyFormState.password"
            :placeholder="t('settings.login.passwordRequired')"
          ></a-input>
        </a-form-item>
      </a-form>
    </a-modal>
  </div>
</template>

<script setup lang="ts">
import type { ProxyConfigType } from '@/utils/storage';
import { useI18n } from 'vue-i18n';
import {
  DeleteOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons-vue';
import { invoke } from '@tauri-apps/api/core';
import {
  message,
  Modal,
  type SelectProps,
  theme as antdTheme,
  type FormInstance,
} from 'ant-design-vue';
import { createVNode, h } from 'vue';
const { t } = useI18n();
const { token } = antdTheme.useToken();

const proxyData = ref<Array<ProxyConfigType>>([]);
const initData = async () => {
  proxyData.value = await invoke('cmd_get_setting_proxy_configs');
};

/**
 * 弹框
 */
const defaultProxy = {
  id: 0,
  proxyType: 'http',
  host: '',
  port: '',
  username: '',
  password: '',
};
const proxyTypeOptions = ref<SelectProps['options']>([
  { value: 'http', label: 'http' },
  { value: 'socket', label: 'socket' },
]);
const showProxyModal = ref<boolean>(false);
const proxyModalTitle = ref<String>('');
const proxyFormRef = ref<FormInstance>();
const proxyFormState = ref<ProxyConfigType>(defaultProxy);

const openModal = (id?: number) => {
  try {
    showProxyModal.value = true;
    proxyModalTitle.value = !!id
      ? t('communal.editProxy')
      : t('communal.addProxy');
    console.log(
      'addd0',
      proxyData.value,
      proxyFormState.value,
      showProxyModal.value,
      proxyModalTitle.value
    );
    if (id) {
      const index = proxyData.value.findIndex((item) => item.id === id);
      proxyFormState.value = proxyData.value[index];
    }
  } catch (error) {
    console.log('add-err', error);
  }
};

const closeModal = () => {
  showProxyModal.value = false;
  proxyFormRef.value?.resetFields();
  initData();
};

const onSubmit = () => {
  proxyFormRef.value?.validate().then(() => {
    const params = {
      ...proxyFormState.value,
      port: Number(proxyFormState.value.port),
    };
    console.log('sub', proxyFormRef.value, params);
    if (params.id) {
      invoke('cmd_update_proxy', { ...params })
        .then(() => {
          updateSuccess();
        })
        .catch((error) => {
          console.log('update-proxy-Error', error);
          updateFail();
        });
    } else {
      invoke('cmd_create_proxy', { ...params })
        .then(() => {
          updateSuccess();
        })
        .catch((error) => {
          console.log('add-proxy-Error', error);
          updateFail();
        });
    }
  });
};

const updateSuccess = () => {
  message.success(t('tips.operateSuccess'));
  closeModal();
};

const updateFail = () => {
  message.error(t('tips.operateFail'));
  // TODO 报错提示精细化
};

const deleteConfirm = (id: number) => {
  Modal.confirm({
    title: t('clipboard.confirm.delete.title'),
    icon: createVNode(h(ExclamationCircleOutlined)),
    content: t('clipboard.confirm.delete.content'),
    okText: t('clipboard.confirm.delete.ok'),
    cancelText: t('clipboard.confirm.delete.cancel'),
    onOk: () => {
      deleteProxy(id);
    },
  });
};

const deleteProxy = (id: number) => {
  invoke('cmd_delete_proxy', { id })
    .then(() => {
      message.success(t('tips.deleteSuccess'));
      initData();
    })
    .catch((error) => {
      console.log('deleteProxy-Error', error);
      message.error(t('tips.deleteFail'));
    });
};

onMounted(() => {
  initData();
});
</script>

<style scoped lang="scss">
.settings-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px;
  color: var(--text-color);
}

.settings-section {
  background: var(--bg-color);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 24px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  margin-bottom: 24px;
  transition: all 0.3s ease;
}

.head {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.settings-section-title {
  font-size: 20px;
  font-weight: 600;
  color: var(--text-color);
}

.settings-options {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.setting-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0;
}

.setting-label {
  font-size: 14px;
  font-weight: 500;
  color: #1d1d1f;
}

.setting-control {
  width: 200px;
  height: 36px;
}

.proxy-title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 17px;
  font-weight: 500;
  padding-bottom: 16px;
  border-bottom: 1px solid #e0e0e0;
  .title-item {
    flex: 2;
    display: flex;
    justify-content: flex-start;
  }
}

.proxy-box {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  padding-bottom: 18px;
  .proxy-item {
    flex: 2;
    display: flex;
    justify-content: flex-start;
    color: #333;
    text-decoration: none;
    width: 100%;
    text-overflow: ellipsis;
  }
}

.operate {
  flex: 1 !important;
  .operate-icon {
    cursor: grab;
    font-size: 20px;
    font-weight: 600;
    padding-right: 15px;
  }
  .disabled-icon {
    color: v-bind('token.colorTextDisabled');
    cursor: not-allowed;
  }
}

.proxy-box:last-child {
  border-bottom: none;
  padding-bottom: 0;
}
.empty {
  display: flex;
  justify-content: center;
  font-size: 16px;
}

:deep(.ant-select-selector) {
  height: 36px !important;
  border-radius: 8px !important;
  border: 1px solid rgba(0, 0, 0, 0.1) !important;
  background: rgba(255, 255, 255, 0.8) !important;
  backdrop-filter: blur(4px);
  transition: all 0.2s ease;
}

:deep(.ant-select:hover .ant-select-selector) {
  border-color: #4784ec !important;
  background: rgba(255, 255, 255, 0.95) !important;
}

:deep(.ant-select-focused .ant-select-selector) {
  border-color: #4784ec !important;
  box-shadow: 0 0 0 2px rgba(71, 132, 236, 0.2) !important;
  background: #ffffff !important;
}

:deep(.ant-switch) {
  background-color: rgba(120, 120, 128, 0.16);
  min-width: 36px;
  height: 20px;
  border-radius: 10px;
}

:deep(.ant-switch-checked) {
  background-color: #0290fe !important;
}

:deep(.ant-switch-handle) {
  width: 16px;
  height: 16px;
  top: 2px;
  left: 2px;
}

:deep(.ant-switch-checked .ant-switch-handle) {
  left: calc(100% - 18px);
}

/* 添加CSS变量 */
:root {
  --text-color: #000000;
  --text-secondary: #666666;
  --bg-color: rgba(255, 255, 255, 0.98);
  --card-bg: rgba(255, 255, 255, 0.9);
  --item-bg: rgba(255, 255, 255, 0.8);
  --item-hover-bg: rgba(255, 255, 255, 0.95);
  --border-color: rgba(0, 0, 0, 0.06);
}

:root[data-theme='dark'] {
  --text-color: #ffffff;
  --text-secondary: #b0b0b0;
  --bg-color: rgba(28, 28, 30, 0.98);
  --card-bg: rgba(44, 44, 46, 0.98);
  --item-bg: rgba(44, 44, 46, 0.95);
  --item-hover-bg: rgba(58, 58, 60, 0.98);
  --border-color: rgba(255, 255, 255, 0.12);
}

/* 替换原来的深色模式适配 */
:root[data-theme='dark'] .settings-section {
  background: var(--bg-color);
}

:root[data-theme='dark'] .settings-section-title,
:root[data-theme='dark'] .setting-label {
  color: #f5f5f7;
}

/* 添加文字阴影效果 */
.settings-section-title,
.setting-label {
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}
</style>
