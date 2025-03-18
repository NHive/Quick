<template>
  <div class="settings-container">
    <div class="head">
      <h3 class="settings-section-title">{{ t('settings.menu.window') }}</h3>
      <a-button class="addBtn" @click="openModal()">{{
        t('communal.add')
      }}</a-button>
    </div>
    <!-- 窗口设置 -->
    <div class="settings-section">
      <div class="settings-options">
        <div class="hotkey-tilte">
          <div class="title-item urlItem">{{ t('communal.title') }}</div>
          <div class="title-item">{{ t('communal.hotkey') }}</div>
          <div class="title-item operate"></div>
        </div>
        <div class="hotkey-box" v-for="(item, index) in urlData">
          <div class="hotkey-item urlItem">{{ item.title }}</div>
          <div class="hotkey-item">{{ formatShortcut(item) }}</div>
          <div class="hotkey-item operate">
            <EditOutlined class="operate-icon" @click="openModal(item.id)" />
            <MoreOutlined
              :class="`operate-icon ${item.isDefault ? 'disabled-icon' : ''}`"
              @click="openMenu(item.id)"
            />
          </div>
        </div>
      </div>
    </div>
  </div>
  <!-- 窗口配置弹框 -->
  <a-modal
    :open="showModal"
    :title="modalTitle"
    @cancel="closeModal"
    @ok="onSubmit"
  >
    <a-form
      style="margin-top: 20px"
      :model="formState"
      ref="formRef"
      :labelCol="{ span: 4 }"
    >
      <a-form-item
        name="url"
        :label="t('communal.url')"
        required
        :rules="[
          { required: true, message: t('tips.inputUrl') },
          {
            pattern:
              /^(https?|ftp):\/\/(?:([a-zA-Z0-9-]+\.)+[a-zA-Z0-9-]+|localhost|\d{1,3}(\.\d{1,3}){3})(:\d{1,5})?(\/[\w- .\/?%&=]*)?(#.*)?$/i,
            message: t('tips.urlFormat'),
          },
        ]"
      >
        <a-input
          v-model:value="formState.url"
          :placeholder="t('tips.inputUrl')"
        ></a-input>
      </a-form-item>
      <a-form-item :label="t('communal.title')" name="title">
        <!-- TODO 增加说明 -->
        <!-- <div
          >{{ t('communal.title') }}：
          <a-popover>
            <template #content>
              {{ t('tips.titleHelp') }}
            </template>
            <QuestionCircleOutlined /></a-popover
        >
          </div> -->
        <a-input
          v-model:value="formState.title"
          :placeholder="t('tips.inputTitle')"
        ></a-input>
      </a-form-item>
      <a-form-item name="hotkey" :label="t('communal.hotkey')">
        <a-input
          v-model:value="formState.hotkey"
          :placeholder="t('tips.inputHotkey')"
        ></a-input>
      </a-form-item>
      <a-form-item name="proxyRule" :label="t('communal.proxyRule')">
        <a-select
          v-model:value="formState.proxyRule"
          :options="proxyOptions"
        ></a-select>
      </a-form-item>
      <a-form-item name="isDefault">
        <a-checkbox v-model:checked="formState.isDefault">{{
          t('menu.setDefault')
        }}</a-checkbox>
      </a-form-item>
    </a-form>
  </a-modal>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, createVNode } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import {
  FormItem,
  message,
  theme as antdTheme,
  Modal,
  type FormInstance,
} from 'ant-design-vue';
import {
  MoreOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
  QuestionCircleOutlined,
} from '@ant-design/icons-vue';
// import { platform } from "@tauri-apps/plugin-os"
import { useI18n } from 'vue-i18n';
import { emit } from '@tauri-apps/api/event';
import { getLocalShortcut } from '@/utils/init';
import { Storage, type ShowUrlsType, type EditUrlsType } from '@/utils/storage';
import { Menu, type MenuItemOptions } from '@tauri-apps/api/menu';

const { t } = useI18n();

const initFormState: EditUrlsType = {
  id: 0,
  title: '',
  isDefault: false,
  url: '',
  icon: '',
  hotkey: '',
  proxyRule: 0,
};

const initProxyState = [{ value: 0, label: t('settings.basic.unUsedProxy') }];

const urlData = ref<ShowUrlsType[]>(); // 当前编辑的窗口链接数组
const isMacOS = ref(false); // 添加一个响应式变量来存储当前操作系统
// antd-定制主题
const { token } = antdTheme.useToken();
const showModal = ref(false);
const modalTitle = ref('');
const formRef = ref<FormInstance>();
// 表单
const formState = ref<EditUrlsType>(initFormState);
const proxyOptions = ref<Record<string, any>>(initProxyState);

// 获取数据
const initData = async () => {
  // TODO 窗口数据
  urlData.value = [
    {
      id: 1,
      title: 'ds',
      icon: '',
      url: 'https://www.deepseek.com/',
      isDefault: false,
      winHotkey: 'alt+c',
      macHotkey: 'option+c',
      proxyRule: 0,
    },
  ];
  //  await Storage.get('urls', [
  //   {
  //     id: 1,
  //     url: 'https://www.deepseek.com/',
  //     winHotkey: 'alt+c',
  //     macHotkey: 'option+c',
  //   },
  // ]);
};
const initProxyData = async () => {
  const proxyData = await Storage.get('proxyConfig', []);
  proxyOptions.value = [...initProxyState, ...proxyData];
  console.log('initS1', urlData.value);
  // TODO 代理规则选项数据
};

// 打开弹框
const openModal = (editId?: number) => {
  showModal.value = true;
  modalTitle.value = !!editId
    ? `${t('communal.editWin')}`
    : `${t('communal.addWin')}`;
  if (editId) {
    // 数据筛选&数据转换
    const data: EditUrlsType[] | undefined = urlData.value
      ?.filter((item: ShowUrlsType) => item.id === editId)
      .map((mapItem: ShowUrlsType) => ({
        id: mapItem.id,
        url: mapItem.url,
        title: mapItem.title,
        icon: mapItem.icon,
        isDefault: mapItem.isDefault,
        proxyRule: mapItem.proxyRule,
        hotkey: mapItem.macHotkey,
      }));
    if (data && Object.keys(data).length) {
      formState.value = toRaw(data[0]);
    } else {
      message.error(t('tips.dataError'));
    }
  }
};
// 关闭弹框
const closeModal = () => {
  showModal.value = false;
  formState.value = initFormState;
  formRef.value?.resetFields();
};

// TODO 提交表单
const onSubmit = () => {
  formRef.value
    ?.validate()
    .then((res) => {
      let oldUrl = urlData.value
        ? JSON.parse(JSON.stringify(urlData.value))
        : [];
      const params = { ...formState.value };
      // 新增数据-生成id
      const idArray = urlData.value?.map((item) => item.id) ?? [];
      const maxId = Math.max(...idArray);
      if (!params.id) {
        params.id = maxId + 1;
      }
      // TODO 处理默认title和icon
      const url = new URL(formState.value.url);
      if (!params.title) {
        params.title = url.hostname;
      }
      params.icon = `${url.origin}/favicon.ico`;
      // TODO 处理isDefault
      if (params.isDefault) {
        // 清除旧默认url
        oldUrl?.forEach((item: EditUrlsType) => {
          item.isDefault = false;
        });
      }
      // TODO处理不同平台快捷键、快捷键逻辑
      // TODO 更新数据
      const newUrl = oldUrl.push(params);
      Storage.set('hotkeys', newUrl);
      console.log('ass', url, formState.value);
      closeModal();
    })
    .catch((error) => {
      console.log('updateHotkeys-Error', error);
    });
};

// TODO 删除窗口链接
const deleteUrl = async (id: number) => {
  const index = urlData.value?.findIndex((item) => item.id === id);
  if (index && !urlData.value?.[index].isDefault) {
    urlData.value?.splice(index, 1);
    try {
      await Storage.set('hotkey', urlData.value);
      message.success(`${t('tips.deleteSuccess')}!`);
    } catch (error) {
      // 更改失败，提示&还原数据
      message.error(`${t('tips.deleteFail')}!`);
      initData();
    }
  }
};
// TODO 删除窗口链接二次确认
const handleDelete = (id: number) => {
  Modal.confirm({
    title: `${t('tips.confimrDelete')}?`,
    icon: createVNode(ExclamationCircleOutlined),
    okText: `${t('settings.confirm')}`,
    cancelText: `${t('settings.cancel')}`,
    onOk: () => deleteUrl(id),
  });
};

// TODO 设置默认窗口
const setDefaulrUrl = async (id: number) => {
  urlData.value?.forEach((item) => {
    if (item.id === id) {
      item.isDefault = true;
    } else {
      item.isDefault = false;
    }
  });
  try {
    await Storage.set('urls', urlData.value);
    message.success(`${t('tips.updateSuccess')}!`);
  } catch (error) {
    // 更改失败，提示&还原数据
    message.error(`${t('tips.updateFail')}!`);
    initData();
  }
};

// 打开右键菜单
const openMenu = async (id: number) => {
  const items: MenuItemOptions[] = [
    { text: t('menu.setDefault'), action: () => setDefaulrUrl(id) },
    { text: t('menu.delete'), action: () => handleDelete(id) },
  ];
  const menu = await Menu.new({ items });
  menu.popup();
};

/**
 * 快捷键设置
 */

// 格式化快捷键显示
const formatShortcut = (item: ShowUrlsType): string[] => {
  const shortcut = isMacOS.value ? item.macHotkey : item.winHotkey;
  const result = shortcut.split('+').map((key: string) => {
    const macKeyMap: Record<string, string> = {
      ctrl: '⌃',
      alt: '⌥',
      shift: '⇧',
      command: '⌘',
      enter: '↵',
      backspace: '⌫',
      space: 'Space',
      tab: '⇥',
      esc: '⎋',
      left: '←',
      right: '→',
      up: '↑',
      down: '↓',
    };

    const winKeyMap: Record<string, string> = {
      ctrl: 'Ctrl',
      alt: 'Alt',
      shift: 'Shift',
      command: 'Win',
      enter: 'Enter',
      backspace: '←',
      space: 'Space',
      tab: 'Tab',
      esc: 'Esc',
      left: '←',
      right: '→',
      up: '↑',
      down: '↓',
    };

    const keyMap = isMacOS.value ? macKeyMap : winKeyMap;
    return keyMap[key.toLowerCase()] || key.toUpperCase();
  });
  return result;
};

onMounted(async () => {
  isMacOS.value = await Storage.get('isMacOS', false);
  // 初始化数据
  initData();
  initProxyData();
});

// 在组件卸载时清理
onBeforeUnmount(() => {});
</script>

<style lang="scss" scoped>
.settings-container {
  max-width: 1200px;
  margin: 0 auto;
  padding: 16px;
  color: var(--text-color);
  overflow: auto;
}

.settings-section {
  display: flex;
  flex-direction: column;
  // background: var(--bg-color);
  background: v-bind('token.colorBgLayout');
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 18px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  margin-bottom: 24px;
  transition: all 0.3s ease;

  :root[data-theme='dark'] & {
    background: var(--bg-color);
  }
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
  margin-bottom: 0;

  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);

  // :root[data-theme='dark'] & {
  //   color: #ffffff;
  // }
}

.settings-options {
  display: flex;
  flex-direction: column;
  gap: 16px;
  color: #1d1d1f;
}

.hotkey-tilte {
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

.hotkey-box {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  padding-bottom: 18px;
  .hotkey-item {
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

.hotkey-box:last-child {
  border-bottom: none;
  padding-bottom: 0;
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

  :root[data-theme='dark'] & {
    color: #f5f5f7;
  }
}

.input-wrapper {
  position: relative;
  width: 180px;
}

.setting-control {
  width: 180px;
  height: 36px;
}

:deep(.ant-input) {
  // color: transparent !important;
  // caret-color: transparent;

  &::placeholder {
    color: #999;
    font-size: 13px;
  }

  height: 36px;
  border-radius: 8px;
  border: 1px solid rgba(0, 0, 0, 0.1);
  background: rgba(255, 255, 255, 0.8);
  backdrop-filter: blur(4px);
  transition: all 0.2s ease;
  font-size: 14px;
  padding: 0 12px;

  &:hover {
    border-color: #4784ec;
    background: rgba(255, 255, 255, 0.95);
  }

  &:focus {
    border-color: #4784ec;
    box-shadow: 0 0 0 2px rgba(71, 132, 236, 0.2);
    background: #ffffff;
  }

  :root[data-theme='dark'] & {
    background: rgba(58, 58, 60, 0.8);
    border-color: rgba(255, 255, 255, 0.1);
    color: #ffffff;

    &:hover {
      border-color: #4784ec;
      background: rgba(58, 58, 60, 0.95);
    }

    &:focus {
      background: #3a3a3c;
      border-color: #4784ec;
      box-shadow: 0 0 0 2px rgba(71, 132, 236, 0.2);
    }
  }
}

.shortcut-preview {
  position: absolute;
  left: 50%;
  top: 50%;
  transform: translate(-50%, -50%);
  display: flex;
  gap: 4px; // 按键之间的间距
  pointer-events: none;

  .key {
    font-size: 13px;
    font-weight: 500;
    color: #4784ec;
    background: rgba(71, 132, 236, 0.12);
    padding: 3px 8px;
    border-radius: 4px;
    min-width: 24px;
    height: 24px;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid rgba(71, 132, 236, 0.2);
    box-shadow: 0 1px 2px rgba(0, 0, 0, 0.05);
    text-transform: none; // 防止全大写

    // Windows 样式调整
    :global(.windows) & {
      font-family:
        system-ui,
        -apple-system,
        BlinkMacSystemFont,
        'Segoe UI',
        Roboto,
        Oxygen,
        Ubuntu,
        Cantarell,
        'Open Sans',
        'Helvetica Neue',
        sans-serif;
      font-size: 12px;
      padding: 3px 6px;
      border-radius: 3px;
      background: linear-gradient(to bottom, #ffffff, #f0f0f0);
      border: 1px solid #ccc;
      box-shadow:
        0 1px 2px rgba(0, 0, 0, 0.1),
        inset 0 1px 0 rgba(255, 255, 255, 0.8);
      color: #333;

      &:active {
        background: linear-gradient(to bottom, #f0f0f0, #e6e6e6);
        box-shadow:
          0 1px 1px rgba(0, 0, 0, 0.1),
          inset 0 1px 3px rgba(0, 0, 0, 0.1);
      }
    }

    // Windows dark theme styles
    :root[data-theme='dark'] :global(.windows) & {
      background: linear-gradient(to bottom, #3a3a3c, #2c2c2e);
      border-color: #666;
      color: #fff;
      box-shadow:
        0 1px 2px rgba(0, 0, 0, 0.2),
        inset 0 1px 0 rgba(255, 255, 255, 0.1);

      &:active {
        background: linear-gradient(to bottom, #2c2c2e, #242426);
      }
    }
  }
}

// 添加动画效果
@keyframes keyPress {
  0% {
    transform: translateY(0);
  }

  50% {
    transform: translateY(1px);
  }

  100% {
    transform: translateY(0);
  }
}

.key {
  transition:
    transform 0.1s ease,
    box-shadow 0.1s ease;

  &:hover {
    animation: keyPress 0.1s ease;
  }
}

// 在样式文件末尾添加主题变量
:root {
  --text-color: #000000;
  --bg-color: rgba(255, 255, 255, 0.98);
}

:root[data-theme='dark'] {
  --text-color: #ffffff;
  --bg-color: rgba(28, 28, 30, 0.98);
}

.addBtn {
  align-self: center;
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.1); /* 添加阴影 */
}
</style>
