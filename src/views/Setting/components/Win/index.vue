<template>
  <div class="settings-container">
    <div class="head">
      <h3 class="settings-section-title">
        {{ t('settings.menu.defaultWindow') }}
      </h3>
    </div>
    <!-- 默认窗口行为设置 -->
    <div class="settings-section">
      <div class="settings-options">
        <div class="setting-item">
          <div class="setting-label">{{ t('communal.hotkey') }}</div>
          <div class="hotkey-control">
            <a-input
              v-model:value="globalHotkey"
              @focus="handleFocusInput('global', '')"
              @keydown.prevent="captureShortcut"
            ></a-input>
            <div v-if="globalHotkey" class="shortcut-preview">
              <span
                class="key"
                v-for="item in formatShortcut(globalHotkey)"
                :key="item"
                >{{ item }}</span
              >
            </div>
          </div>
        </div>
        <div class="setting-item">
          <div class="setting-label">{{ t('communal.behavior') }}</div>
          <a-select
            v-model:value="openWinRule"
            :options="openWinOptions"
            @change="setOpenWinRule"
          ></a-select>
        </div>
      </div>
    </div>
    <div class="head">
      <h3 class="settings-section-title">
        {{ t('settings.menu.customWindow') }}
      </h3>
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
          <div class="hotkey-item">{{ item.title }}</div>
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
    style="top: 30px"
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
        <div class="title-tips">{{ t('tips.titleHelp') }}</div>
      </a-form-item>
      <a-form-item
        name="hotkey"
        :label="t('communal.hotkey')"
        class="hotkey-control"
      >
        <a-input
          v-model:value="formState.shortcut"
          :placeholder="t('tips.inputHotkey')"
          @focus="handleFocusInput('win', '')"
          @keydown.prevent="captureShortcut"
        ></a-input>
        <div v-if="formState.shortcut" class="shortcut-preview">
          <span
            class="key"
            v-for="item in formatShortcut(formState.shortcut)"
            :key="item"
            >{{ item }}</span
          >
        </div>
      </a-form-item>
      <a-form-item name="proxyId" :label="t('communal.proxyRule')">
        <a-select
          v-model:value="formState.proxyId"
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
  message,
  theme as antdTheme,
  Modal,
  type FormInstance,
  type SelectProps,
} from 'ant-design-vue';
import {
  MoreOutlined,
  EditOutlined,
  ExclamationCircleOutlined,
} from '@ant-design/icons-vue';
import { useI18n } from 'vue-i18n';
import { Storage, type UrlsType } from '@/utils/storage';
import { Menu, type MenuItemOptions } from '@tauri-apps/api/menu';

const { t } = useI18n();

const initFormState: UrlsType = {
  id: 0,
  title: '',
  isDefault: false,
  url: '',
  icon: '',
  shortcut: '',
  proxyId: 0,
};
const initProxyState = [{ value: 0, label: t('settings.basic.unUsedProxy') }];
const openWinOptions = ref<SelectProps['options']>([
  { value: 'lastTime', label: t('settings.lastTimeWin') },
  { value: 'defaultUrl', label: t('settings.defaultUrl') },
]);
// antd-定制主题
const { token } = antdTheme.useToken();

const globalHotkey = ref(''); // 全局快捷键——打开窗口
const openWinRule = ref('lastTime'); // 打开窗口的默认规则：打开上次窗口｜打开默认窗口
const defaultUrl = ref(''); // 当前默认窗口
const urlData = ref<UrlsType[]>(); // 当前编辑的窗口链接数组
const isMacOS = ref(false); // 添加一个响应式变量来存储当前操作系统
const showModal = ref(false); // 打开弹框
const modalTitle = ref(''); // 弹框标题
const formRef = ref<FormInstance>(); // 表单Ref
const formState = ref<UrlsType>(initFormState); // 表单State
const proxyOptions = ref<Record<string, any>>(initProxyState); // 代理规则选项

// TODO loading
// 获取数据
const initData = async () => {
  getWinConfig();
  getUrlData();
  getProxyData();
};
// 获取代理规则选项数据
const getProxyData = async () => {
  const proxyData: Array<any> = await invoke('cmd_get_setting_proxy_configs');
  console.log('proxy1', proxyData);
  proxyOptions.value = [...initProxyState, ...proxyData];
  // TODO 代理规则选项数据格式化
};
// 获取窗口链接数据
const getUrlData = async () => {
  // TODO 窗口数据
  // urlData.value = [
  //   {
  //     id: 1,
  //     title: 'ds',
  //     icon: '',
  //     url: 'https://www.deepseek.com/',
  //     isDefault: false,
  //     winHotkey: 'alt+c',
  //     macHotkey: 'option+c',
  //     proxyId: 0,
  //   },
  // ];
  // 获取所有窗口配置
  let data: Array<any> = await invoke('cmd_get_setting_window_configs');
  // 默认值处理；若窗口配置无值，则添加默认值
  if (!data || !Array.from(data).length) {
    await invoke('cmd_add_window', {
      title: 'deepseek',
      url: 'https://www.deepseek.com/',
    });
    // 更新数据
    data = await invoke('cmd_get_setting_window_configs');
  }
  urlData.value = data;
};
// 获取窗口默认配置
const getWinConfig = async () => {
  // TODO 更新接口
  globalHotkey.value = await Storage.get('globalHotkey', 'ControlLeft+C');
  openWinRule.value = await Storage.get('openWinRule', 'lastTime');
};

// 打开弹框
const openModal = (editId?: number) => {
  showModal.value = true;
  modalTitle.value = !!editId
    ? `${t('communal.editWin')}`
    : `${t('communal.addWin')}`;
  if (editId) {
    // TODO 数据筛选&数据转换
    const data: UrlsType[] | undefined = urlData.value
      ?.filter((item: UrlsType) => item.id === editId)
      .map((mapItem: UrlsType) => ({
        ...mapItem,
        isDefault: false,
      }));
    if (data && Object.keys(data).length) {
      formState.value = toRaw(data[0]);
      // editShortcut.value = formState.value.shortcut;
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
      const params = { ...formState.value };
      // 处理默认title
      const url = new URL(formState.value.url);
      if (!params.title) {
        params.title = url.hostname;
      }
      if (!params.proxyId) {
        delete params.proxyId;
      }
      // TODO 更新数据
      if (!params.id) {
        invoke('cmd_add_window', params).then(() => {
          updateUrlSuccess();
        });
      } else {
        invoke('cmd_update_window', params).then(() => {
          updateUrlSuccess();
        });
      }

      console.log('ass', url, formState.value);
    })
    .catch((error) => {
      console.log('updateHotkeys-Error', error);
    });
};

const updateUrlSuccess = () => {
  message.success(`${t('tips.operateSuccess')}!`);
  closeModal();
  getUrlData();
  // TODO 更新setup的默认链接值
};

const updateUrlFail = () => {
  // TODO 错误提示
};

// 删除窗口链接
const deleteUrl = async (id: number) => {
  try {
    await invoke('cmd_delete_window', { id });
    message.success(`${t('tips.deleteSuccess')}!`);
  } catch (error) {
    // 更改失败，提示&还原数据
    message.error(`${t('tips.deleteFail')}!`);
    getUrlData();
  }
};
// 删除窗口链接二次确认
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
  const index = urlData.value?.findIndex((item) => item.id === id);
  if (index !== undefined) {
    invoke('', { ...urlData.value?.[index], isDefault: true })
      .then(() => {
        message.success(`${t('tips.updateSuccess')}!`);
      })
      .catch(() => {
        // 更改失败，提示&还原数据
        message.error(`${t('tips.updateFail')}!`);
        getUrlData();
      });
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

// 更改默认窗口打开链接行为
const setOpenWinRule = (value: string) => {
  Storage.set('openWinRule', value);
};

/**
 * 快捷键设置
 * 设置全局快捷键逻辑：设置完成后——>主动调接口更新配置
 * url快捷键设置流程：设置完成后，点击表单提交——>调接口更新数据
 */
const isFirstFocus = ref(false);
const lastShortcut = ref('');
const keyPressTimeout = ref();
const isFetching = ref(false); // 设置全局快捷键时，正在请求接口中
//  TODO 1、快捷键设置
// 重置lastShortcut，更新快捷键input显示
const resetShortcut = (type: 'global' | 'win', newVal: string) => {
  if (type === 'global') {
    lastShortcut.value = globalHotkey.value;
    globalHotkey.value = newVal;
  } else {
    lastShortcut.value = formState.value.shortcut;
    formState.value.shortcut = newVal;
  }
};
const handleFocusInput = (type: 'global' | 'win', newVal: string) => {
  try {
    if (isFirstFocus.value) {
      resetShortcut(type, newVal);
    }
    isFirstFocus.value = true;
  } catch (error) {}
};
// 监听键盘输入，设置快捷键
const captureShortcut = (
  event: KeyboardEvent,
  type: 'global' | 'win'
): void => {
  // 阻止默认行为和事件冒泡
  event.preventDefault();
  event.stopPropagation();

  // 如果是重复事件，直接返回
  if (event.repeat) {
    return;
  }

  // 清除之前的超时
  if (keyPressTimeout.value) {
    clearTimeout(keyPressTimeout.value);
  }

  // 在输入时禁用输入法
  const input = event.target as HTMLInputElement;
  input.blur();
  input.focus();

  const keys: string[] = [];
  if (event.ctrlKey) keys.push('ctrl');
  if (event.altKey) keys.push('alt');
  if (event.shiftKey) keys.push('shift');
  if (event.metaKey) keys.push('command');
  console.log('keyboard', keys, event, event.code);

  const keyCode = event.code;
  const isModifierKey = [
    'ControlLeft',
    'AltLeft',
    'ShiftLeft',
    'MetaLeft',
    'ControlRight',
    'AltRight',
    'ShiftRight',
    'MetaRight',
  ].includes(keyCode);

  // 如果是修饰键，只更新显示，不触发完成事件
  if (isModifierKey) {
    if (type === 'global') {
      globalHotkey.value = keys.join('+');
    } else {
      formState.value.shortcut = keys.join('+');
    }
    return;
  }
  // 使用一个映射来转换键码
  const keyMap: { [key: string]: string } = {
    KeyA: 'a',
    KeyB: 'b',
    KeyC: 'c',
    Digit1: '1',
    Digit2: '2',
    Minus: '-',
    Equal: '=',
    BracketLeft: '[',
    BracketRight: ']',
    Semicolon: ';',
    Quote: "'",
    Backslash: '\\',
    Comma: ',',
    Period: '.',
    Slash: '/',
    Space: 'space',
    Enter: 'enter',
    Backspace: 'backspace',
    Tab: 'tab',
    Escape: 'esc',
    ArrowUp: '↑',
    ArrowDown: '↓',
    ArrowLeft: '←',
    ArrowRight: '→',
  };

  if (keyCode.startsWith('Key')) {
    keys.push(keyCode.slice(3).toLowerCase());
  } else if (keyCode.startsWith('Digit')) {
    keys.push(keyCode.slice(5));
  } else {
    keys.push(keyMap[keyCode] || keyCode.toLowerCase());
  }
  const data = keys.join('+');
  if (type === 'global') {
    globalHotkey.value = data;
  } else {
    formState.value.shortcut = data;
  }

  // 延长完成事件的等待时间
  keyPressTimeout.value = setTimeout(() => {
    handleShortcutComplete(type, data);
  }, 350);
};

const handleShortcutComplete = async (
  type: 'global' | 'win',
  shortcut: string
): Promise<void> => {
  try {
    if (type === 'global') {
      // TODO 请求接口更新全局快捷键，增加loading效果
      // 更新快捷键成功，显示最新值
      await Storage.set('globalHotkey', shortcut);
      // getWinConfig()
    }
  } catch (error) {
    console.log('handleShortcutComplete-Error', error);
    message.error(t('settings.shortcuts.saveError'));
  } finally {
    inputBlur();
    isFirstFocus.value = true;
  }
};

// 触发输入框失焦
const inputBlur = () => {
  const { activeElement } = document;
  if (activeElement instanceof HTMLInputElement) {
    activeElement.blur();
  }
};

// 格式化快捷键显示
const formatShortcut = (shortcut: string): string[] => {
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

// TODO 2、默认窗口行为

onMounted(async () => {
  isMacOS.value = await Storage.get('isMacOS', false);
  // 初始化数据
  initData();
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
}

.settings-options {
  display: flex;
  flex-direction: column;
  gap: 20px;
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
}

.input-wrapper {
  position: relative;
  width: 180px;
}

.setting-control {
  width: 180px;
  height: 36px;
}
.hotkey-control {
  position: relative;
  :deep(.ant-input) {
    color: transparent !important;
    caret-color: transparent;
    line-height: 30px;

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
    padding: 0px 12px;

    &:hover {
      border-color: #4784ec;
      background: rgba(255, 255, 255, 0.95);
    }

    &:focus {
      border-color: #4784ec;
      box-shadow: 0 0 0 2px rgba(71, 132, 236, 0.2);
      background: #ffffff;
    }
  }
  .hotkey-value {
    background: #076aecf7;
    border: 1px solid #076aecf7;
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
.title-tips {
  font-size: 12px;
  color: #bebcbc;
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
