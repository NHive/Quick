// Types
export interface HotKey {
  key: string
  value: string
  winDefaultVal: string
  macDefaultVal: string
  label: string
  isGlobal?: boolean
}

// Constants
export const HOTKEY_CONFIG: HotKey[] = [
  {
    key: "showMainWinHotKey",
    label: "settings.shortcuts.showMainWindow",
    value: "",
    winDefaultVal: "alt+v",
    macDefaultVal: "option+v",
    isGlobal: true
  },
  {
    key: "previewHotKey",
    label: "settings.shortcuts.preview",
    value: "",
    winDefaultVal: " ",
    macDefaultVal: " "
  },
  {
    key: "pasteHotKey",
    label: "settings.shortcuts.paste",
    value: "",
    winDefaultVal: "ctrl+enter",
    macDefaultVal: "command+enter"
  },
  {
    key: "previousPageHotKey",
    label: "settings.shortcuts.previousPage",
    value: "",
    winDefaultVal: "[",
    macDefaultVal: "["
  },
  {
    key: "nextPageHotKey",
    label: "settings.shortcuts.nextPage",
    value: "",
    winDefaultVal: "]",
    macDefaultVal: "]"
  },
  {
    key: "deleteSelectedHotKey",
    label: "settings.shortcuts.deleteSelected",
    value: "",
    winDefaultVal: "backspace",
    macDefaultVal: "backspace"
  },
  {
    key: "insertSelectedHotKey",
    label: "settings.shortcuts.insertSelected",
    value: "",
    winDefaultVal: "enter",
    macDefaultVal: "enter"
  },
  {
    key: "focusSearchHotKey",
    label: "settings.shortcuts.focusSearch",
    value: "",
    winDefaultVal: "ctrl+f",
    macDefaultVal: "command+f"
  }
]
