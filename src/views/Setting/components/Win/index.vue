<template>
  <div class="settings-container">
    <h3 class="settings-section-title">{{ t('settings.menu.window') }}</h3>
    <!-- 窗口设置 -->
    <div class="settings-section">
      <div class="settings-options">
        <!-- <div class="hotkey-tilte">
          <div class="title-item">{{ t("communal.url") }}</div>
          <div class="title-item">{{ t("communal.hotkey") }}</div>
        </div> -->
        <div class="hotkey-box" v-for="(item, index) in [1, 2]">
          <div class="hotkey-item">{{ t("communal.url") }}：xxxxx</div>
          <div class="hotkey-item">{{ t("communal.hotkey") }}：yyyy</div>
        </div>

        <a-button size="large" class="addBtn" shape="circle" :icon="h(PlusOutlined)" />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onBeforeUnmount, h } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { message } from 'ant-design-vue';
// import { platform } from "@tauri-apps/plugin-os"
import { useI18n } from 'vue-i18n';
import { emit } from '@tauri-apps/api/event';
import { getLocalShortcut } from '@/utils/init';
import { Storage } from '@/utils/storage';
import { type HotKey, HOTKEY_CONFIG } from '@/utils/hotkeys';
import { PlusOutlined } from '@ant-design/icons-vue';

const { t } = useI18n();
const keyPressTimeout = ref<any>();
const blurTimeout = ref<any>();

const hotKeyMap = ref<HotKey[]>(HOTKEY_CONFIG); // 当前编辑的最新快捷键数组
const lastShortcut = ref<string>(''); // 上一次设置成功的快捷键值
const isFirstFocus = ref<boolean>(true);
const isMacOS = ref(false); // 添加一个响应式变量来存储当前操作系统

// 获取快捷键数据
const initData = (): void => {
  const data = hotKeyMap.value.map((item) =>
    getLocalShortcut(item.key, item.winDefaultVal, item.macDefaultVal).then(
      (val) => ({
        ...item,
        value: val,
      })
    )
  );
  Promise.all(data).then((resData) => {
    hotKeyMap.value = resData;
  });
};

// 更新持久化数据
const setShortcut = async (key: string, shortcut: string): Promise<void> => {
  try {
    await invoke('set_setup', { key, value: shortcut });
    message.success(t('settings.shortcuts.saveSuccess'));
  } catch (error) {
    console.error('Failed to set shortcut:', error);
    message.error(t('settings.shortcuts.saveError'));
  }
};

// 重置lastShortcut、更新input显示
const resetShortcut = async (key: string, newVal: string): Promise<any> => {
  const index = hotKeyMap.value.findIndex((item) => item.key === key);
  lastShortcut.value = await getLocalShortcut(
    key,
    hotKeyMap.value[index].winDefaultVal,
    hotKeyMap.value[index].macDefaultVal
  );
  hotKeyMap.value[index].value = newVal;
};

// input聚焦后，按下键盘会触发多次聚焦
// 第一次聚焦才执行此逻辑
const handleFocusInput = async (key: string, value: string) => {
  try {
    if (isFirstFocus.value) {
      // input显示置空
      await resetShortcut(key, value);

      // 如果是全局快捷键，则注销所有全局快捷键
      const currentHK = hotKeyMap.value.find((item) => item.key === key);
      if (currentHK?.isGlobal) {
        // 注销所有全局快捷键
        emit('unRegisterAll-shortcut');
      }

      // 忽略输入时多次触发的focus
      isFirstFocus.value = false;
    }
  } catch (error) {
    console.log('handleF-Error', error);
  }
};

const handleBlurInput = (key: string, value: string) => {
  // 缩短blur延时，避免与keypress冲突
  clearTimeout(blurTimeout.value);
  blurTimeout.value = setTimeout(async () => {
    console.log('blur!', value);
    const currentHK = hotKeyMap.value.find((item) => item.key === key);
    // input显示重置
    await resetShortcut(key, value);

    // 如果是全局快捷键，重新注册所有全局快捷键
    if (currentHK?.isGlobal) {
      // 重新注册所有全局快捷键
      hotKeyMap.value
        .filter((item) => item.isGlobal && item.value)
        .forEach((item) => {
          emit('register-shortcut', { shortcut: item.value });
        });
    }

    isFirstFocus.value = true;
  }, 1000); // 添加超时时间
};

// 触发输入框失焦
const inputBlur = () => {
  const { activeElement } = document;
  if (activeElement instanceof HTMLInputElement) {
    activeElement.blur();
  }
};

// 检查快捷键是否重复并处理
// 交互优化——设置完成后主动使输入框失焦，因微任务时序问题不在finally统一处理失焦
const handleShortcutComplete = async (
  key: string,
  shortcut: string
): Promise<void> => {
  try {
    // 检查是否有重复的快捷键
    const duplicateItem = hotKeyMap.value.find(
      (item) => item.value === shortcut && item.key !== key
    );

    if (duplicateItem) {
      message.warning(
        t('settings.shortcuts.duplicateWarning', { hotkey: shortcut })
      );
      inputBlur();
      return;
    }

    const currentHotKey = hotKeyMap.value.find((item) => item.key === key);

    if (!currentHotKey) return;

    // 检查是否为组合键
    if (currentHotKey.isGlobal && !shortcut.includes('+')) {
      message.warning(t('settings.shortcuts.requireCombination'));
      inputBlur();
      return;
    }

    // 保存快捷键
    await setShortcut(key, shortcut);
    lastShortcut.value = shortcut;
    inputBlur();
  } catch (error) {
    console.log('handleShortcutComplete-Error', error);
    message.error(t('settings.shortcuts.saveError'));
    inputBlur();
  }
};
// 监听键盘输入，设置快捷键
const captureShortcut = (event: KeyboardEvent, key: string): void => {
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
    const index = hotKeyMap.value.findIndex((item) => item.key === key);
    hotKeyMap.value[index].value = keys.join('+');
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
  const index = hotKeyMap.value.findIndex((item) => item.key === key);
  hotKeyMap.value[index].value = data;

  // 延长完成事件的等待时间
  keyPressTimeout.value = setTimeout(() => {
    const currentHK = hotKeyMap.value.find((item) => item.key === key);
    if (currentHK?.isGlobal && !data.includes('+')) {
      // 如果是全局快捷键但没有组合键，不触发完成事件
      return;
    }
    handleShortcutComplete(key, data);
  }, 350);
};

// 格式化快捷键显示
const formatShortcut = (shortcut: string): string[] =>
  shortcut.split('+').map((key) => {
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

onMounted(async () => {
  isMacOS.value = await Storage.get('isMacOS', false);
  // 初始化数据
  initData();
});

// 在组件卸载时清理
onBeforeUnmount(() => {
  if (keyPressTimeout.value) {
    clearTimeout(keyPressTimeout.value);
  }
  if (blurTimeout.value) {
    clearTimeout(blurTimeout.value);
  }
});
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
  background: var(--bg-color);
  backdrop-filter: blur(20px);
  border-radius: 20px;
  padding: 24px;
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.08);
  margin-bottom: 24px;
  transition: all 0.3s ease;

  :root[data-theme='dark'] & {
    background: var(--bg-color);
  }
}

.settings-section-title {
  font-size: 20px;
  font-weight: 600;
  margin-bottom: 24px;
  text-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);

  :root[data-theme='dark'] & {
    color: #ffffff;
  }
}

.settings-options {
  display: flex;
  flex-direction: column;
  gap: 24px;
  color: #1d1d1f;
}

.hotkey-tilte {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 18px;
  font-weight: 500;
  .title-item {
    flex: 1;
    display: flex;
    justify-content: center;
  }
}

.hotkey-box {
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid #e0e0e0;
  padding-bottom: 24px;
  .hotkey-item {
    flex: 1;
    display: flex;
    justify-content: center;
    flex-grow: 1;
    color: #333;
    text-decoration: none;
    width: 100%;
    text-overflow: ellipsis;
  }
}
.hotkey-box:last-child{
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
  color: transparent !important;
  caret-color: transparent;

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
