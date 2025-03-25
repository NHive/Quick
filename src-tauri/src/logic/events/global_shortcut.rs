use lazy_static::lazy_static;
use log::{error, info, warn};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tauri::{self, App};
use tauri_plugin_global_shortcut::{
    Code, GlobalShortcutExt, Modifiers, Shortcut, ShortcutEvent, ShortcutState,
};

use crate::infrastructure::error::AppError;
use crate::window::control;

lazy_static! {
    pub static ref SHORTCUT_MANAGER: Arc<RwLock<ShortcutManager>> =
        Arc::new(RwLock::new(ShortcutManager::new()));
}
/// 快捷键配置结构体
pub struct ShortcutConfig {
    /// 快捷键
    shortcut: Shortcut,
    /// 快捷键描述
    description: String,
    /// 快捷键处理函数
    handler: Box<dyn Fn(&tauri::AppHandle) + Send + Sync + 'static>,
}

impl ShortcutConfig {
    pub fn new<F>(modifiers: Option<Modifiers>, code: Code, description: &str, handler: F) -> Self
    where
        F: Fn(&tauri::AppHandle) + Send + Sync + 'static,
    {
        ShortcutConfig {
            shortcut: Shortcut::new(modifiers, code),
            description: description.to_string(),
            handler: Box::new(handler),
        }
    }

    /// 从字符串创建快捷键配置
    /// 例如: "alt+g", "shift+ctrl+KeyX", "option+v"
    pub fn from_string<F>(
        shortcut_str: &str,
        description: &str,
        handler: F,
    ) -> Result<Self, AppError>
    where
        F: Fn(&tauri::AppHandle) + Send + Sync + 'static,
    {
        let tokens: Vec<&str> = shortcut_str.split('+').collect();

        let mut modifiers = Modifiers::empty();
        let mut code = None;

        for token in tokens.iter() {
            let token = token.trim().to_uppercase();
            match token.as_str() {
                "ALT" | "OPTION" => {
                    modifiers |= Modifiers::ALT;
                }
                "CTRL" | "CONTROL" => {
                    modifiers |= Modifiers::CONTROL;
                }
                "SHIFT" => {
                    modifiers |= Modifiers::SHIFT;
                }
                "SUPER" | "CMD" | "COMMAND" => {
                    modifiers |= Modifiers::SUPER;
                }
                _ => {
                    // 尝试将token解析为Code
                    if let Ok(key_code) = parse_key_code(token.as_str()) {
                        code = Some(key_code);
                    } else {
                        return Err(AppError::new_validation_error(format!(
                            "无法识别的按键: {}",
                            token
                        )));
                    }
                }
            }
        }

        if let Some(key_code) = code {
            Ok(Self::new(
                if modifiers.is_empty() {
                    None
                } else {
                    Some(modifiers)
                },
                key_code,
                description,
                handler,
            ))
        } else {
            Err(AppError::new_validation_error(
                "快捷键格式无效，必须包含一个有效的按键".to_string(),
            ))
        }
    }
}

/// 解析字符串为Code
fn parse_key_code(key: &str) -> Result<Code, AppError> {
    use tauri_plugin_global_shortcut::Code::*;
    match key {
        "A" | "KEYA" => Ok(KeyA),
        "B" | "KEYB" => Ok(KeyB),
        "C" | "KEYC" => Ok(KeyC),
        "D" | "KEYD" => Ok(KeyD),
        "E" | "KEYE" => Ok(KeyE),
        "F" | "KEYF" => Ok(KeyF),
        "G" | "KEYG" => Ok(KeyG),
        "H" | "KEYH" => Ok(KeyH),
        "I" | "KEYI" => Ok(KeyI),
        "J" | "KEYJ" => Ok(KeyJ),
        "K" | "KEYK" => Ok(KeyK),
        "L" | "KEYL" => Ok(KeyL),
        "M" | "KEYM" => Ok(KeyM),
        "N" | "KEYN" => Ok(KeyN),
        "O" | "KEYO" => Ok(KeyO),
        "P" | "KEYP" => Ok(KeyP),
        "Q" | "KEYQ" => Ok(KeyQ),
        "R" | "KEYR" => Ok(KeyR),
        "S" | "KEYS" => Ok(KeyS),
        "T" | "KEYT" => Ok(KeyT),
        "U" | "KEYU" => Ok(KeyU),
        "V" | "KEYV" => Ok(KeyV),
        "W" | "KEYW" => Ok(KeyW),
        "X" | "KEYX" => Ok(KeyX),
        "Y" | "KEYY" => Ok(KeyY),
        "Z" | "KEYZ" => Ok(KeyZ),
        "0" | "DIGIT0" => Ok(Digit0),
        "1" | "DIGIT1" => Ok(Digit1),
        "2" | "DIGIT2" => Ok(Digit2),
        "3" | "DIGIT3" => Ok(Digit3),
        "4" | "DIGIT4" => Ok(Digit4),
        "5" | "DIGIT5" => Ok(Digit5),
        "6" | "DIGIT6" => Ok(Digit6),
        "7" | "DIGIT7" => Ok(Digit7),
        "8" | "DIGIT8" => Ok(Digit8),
        "9" | "DIGIT9" => Ok(Digit9),
        "F1" => Ok(F1),
        "F2" => Ok(F2),
        "F3" => Ok(F3),
        "F4" => Ok(F4),
        "F5" => Ok(F5),
        "F6" => Ok(F6),
        "F7" => Ok(F7),
        "F8" => Ok(F8),
        "F9" => Ok(F9),
        "F10" => Ok(F10),
        "F11" => Ok(F11),
        "F12" => Ok(F12),
        "SPACE" => Ok(Space),
        "ENTER" => Ok(Enter),
        "ESCAPE" | "ESC" => Ok(Escape),
        "TAB" => Ok(Tab),
        "BACKSPACE" => Ok(Backspace),
        "DELETE" => Ok(Delete),
        "UP" | "ARROWUP" => Ok(ArrowUp),
        "DOWN" | "ARROWDOWN" => Ok(ArrowDown),
        "LEFT" | "ARROWLEFT" => Ok(ArrowLeft),
        "RIGHT" | "ARROWRIGHT" => Ok(ArrowRight),
        _ => Err(AppError::new_validation_error(format!(
            "不支持的按键: {}",
            key
        ))),
    }
}

/// 快捷键管理器
pub struct ShortcutManager {
    /// 存储所有注册的快捷键配置
    shortcuts: HashMap<String, ShortcutConfig>,
    /// 保存当前app句柄的弱引用，用于注册/注销快捷键
    app_handle: Option<Arc<tauri::AppHandle>>,
}

impl ShortcutManager {
    /// 创建新的快捷键管理器
    fn new() -> Self {
        info!("初始化快捷键管理器");
        ShortcutManager {
            shortcuts: HashMap::new(),
            app_handle: None,
        }
    }

    /// 获取快捷键管理器实例
    pub fn instance() -> Arc<RwLock<ShortcutManager>> {
        SHORTCUT_MANAGER.clone()
    }

    /// 设置应用句柄
    pub fn set_app_handle(&mut self, app_handle: &tauri::AppHandle) {
        info!("设置快捷键管理器的应用句柄");
        self.app_handle = Some(Arc::new(app_handle.clone()));
    }

    /// 检查快捷键是否与已注册快捷键冲突
    fn check_conflict(
        &self,
        shortcut: &Shortcut,
        current_id: &str,
    ) -> Option<(&String, &ShortcutConfig)> {
        for (id, config) in &self.shortcuts {
            if &config.shortcut == shortcut && id != current_id {
                warn!(
                    "检测到快捷键冲突: '{}' 与 '{}({})' 冲突",
                    current_id, config.description, id
                );
                return Some((id, config));
            }
        }
        None
    }

    /// 注册或更新快捷键
    /// 如果快捷键ID已存在，则更新；否则添加新快捷键
    /// 会自动检测快捷键冲突
    pub fn register_shortcut<F>(
        &mut self,
        id: &str,
        modifiers: Option<Modifiers>,
        code: Code,
        description: &str,
        handler: F,
    ) -> Result<(), AppError>
    where
        F: Fn(&tauri::AppHandle) + Send + Sync + 'static,
    {
        info!("注册快捷键: ID={}, 描述={}", id, description);
        let new_config = ShortcutConfig::new(modifiers, code, description, handler);

        // 检查是否与其他快捷键冲突
        if let Some((conflict_id, conflict_config)) = self.check_conflict(&new_config.shortcut, id)
        {
            error!(
                "快捷键冲突: ID={} 的快捷键与 '{}({})' 冲突",
                id, conflict_config.description, conflict_id
            );
            return Err(AppError::new_validation_error(format!(
                "快捷键冲突: 当前设置的快捷键与 '{}({})' 冲突",
                conflict_config.description, conflict_id
            )));
        }

        // 检查是否需要更新现有快捷键
        let need_update = if let Some(old_config) = self.shortcuts.get(id) {
            // 如果快捷键内容不同，需要先注销旧快捷键
            if old_config.shortcut != new_config.shortcut {
                info!(
                    "更新快捷键: ID={}, 从 '{}' 更新为 '{}'",
                    id, old_config.description, new_config.description
                );
                if let Some(app) = &self.app_handle {
                    // 注销旧快捷键
                    if let Err(e) = app.global_shortcut().unregister(old_config.shortcut) {
                        warn!(
                            "注销快捷键失败: ID={}, 描述={}, 错误: {:?}",
                            id, old_config.description, e
                        );
                    } else {
                        info!("已注销旧快捷键: ID={}, 描述={}", id, old_config.description);
                    }
                }
                true
            } else {
                // 快捷键相同，但描述不同，只需要更新数据结构
                if old_config.description != new_config.description {
                    info!(
                        "更新快捷键描述: ID={}, 从 '{}' 更新为 '{}'",
                        id, old_config.description, new_config.description
                    );
                }
                old_config.description != new_config.description
            }
        } else {
            // 新增快捷键
            info!("新增快捷键: ID={}, 描述={}", id, new_config.description);
            true
        };

        // 如果需要注册新快捷键
        if need_update {
            if let Some(app) = &self.app_handle {
                match app.global_shortcut().register(new_config.shortcut) {
                    Ok(_) => info!("成功注册快捷键: ID={}, 描述={}", id, new_config.description),
                    Err(e) => {
                        error!(
                            "注册快捷键失败: ID={}, 描述={}, 错误: {:?}",
                            id, new_config.description, e
                        );
                        return Err(AppError::new_unknown(format!("注册快捷键失败: {}", e)));
                    }
                }
            } else {
                warn!("应用句柄未设置，快捷键将在句柄设置后注册: ID={}", id);
            }
        }

        // 更新快捷键配置
        self.shortcuts.insert(id.to_string(), new_config);
        Ok(())
    }

    /// 使用字符串格式注册快捷键
    /// 例如: "alt+g", "shift+ctrl+x", "option+v"
    pub fn register_shortcut_from_string<F>(
        &mut self,
        id: &str,
        shortcut_str: &str,
        description: &str,
        handler: F,
    ) -> Result<(), AppError>
    where
        F: Fn(&tauri::AppHandle) + Send + Sync + 'static,
    {
        info!(
            "从字符串注册快捷键: ID={}, 快捷键={}, 描述={}",
            id, shortcut_str, description
        );

        let config = match ShortcutConfig::from_string(shortcut_str, description, handler) {
            Ok(config) => config,
            Err(e) => {
                error!(
                    "快捷键字符串解析失败: ID={}, 快捷键={}, 错误: {:?}",
                    id, shortcut_str, e
                );
                return Err(e);
            }
        };

        // 检查是否与其他快捷键冲突
        if let Some((conflict_id, conflict_config)) = self.check_conflict(&config.shortcut, id) {
            error!(
                "快捷键冲突: ID={} 的快捷键与 '{}({})' 冲突",
                id, conflict_config.description, conflict_id
            );
            return Err(AppError::new_validation_error(format!(
                "快捷键冲突: 当前设置的快捷键与 '{}({})' 冲突",
                conflict_config.description, conflict_id
            )));
        }

        // 检查是否需要更新现有快捷键
        let need_update = if let Some(old_config) = self.shortcuts.get(id) {
            // 如果快捷键内容不同，需要先注销旧快捷键
            if old_config.shortcut != config.shortcut {
                info!(
                    "更新快捷键: ID={}, 从 '{}' 更新为 '{}'",
                    id, old_config.description, config.description
                );
                if let Some(app) = &self.app_handle {
                    // 注销旧快捷键
                    if let Err(e) = app.global_shortcut().unregister(old_config.shortcut) {
                        warn!(
                            "注销快捷键失败: ID={}, 描述={}, 错误: {:?}",
                            id, old_config.description, e
                        );
                    } else {
                        info!("已注销旧快捷键: ID={}, 描述={}", id, old_config.description);
                    }
                }
                true
            } else {
                // 快捷键相同，但描述不同，只需要更新数据结构
                if old_config.description != config.description {
                    info!(
                        "更新快捷键描述: ID={}, 从 '{}' 更新为 '{}'",
                        id, old_config.description, config.description
                    );
                }
                old_config.description != config.description
            }
        } else {
            // 新增快捷键
            info!(
                "新增快捷键: ID={}, 描述={}, 快捷键={}",
                id, config.description, shortcut_str
            );
            true
        };

        // 如果需要注册新快捷键
        if need_update {
            if let Some(app) = &self.app_handle {
                match app.global_shortcut().register(config.shortcut) {
                    Ok(_) => info!("成功注册快捷键: ID={}, 描述={}", id, config.description),
                    Err(e) => {
                        error!(
                            "注册快捷键失败: ID={}, 描述={}, 错误: {:?}",
                            id, config.description, e
                        );
                        return Err(AppError::new_unknown(format!("注册快捷键失败: {}", e)));
                    }
                }
            } else {
                warn!("应用句柄未设置，快捷键将在句柄设置后注册: ID={}", id);
            }
        }

        // 更新快捷键配置
        self.shortcuts.insert(id.to_string(), config);
        Ok(())
    }

    /// 移除快捷键
    pub fn remove_shortcut(&mut self, id: &str) -> Result<(), AppError> {
        info!("移除快捷键: ID={}", id);
        if let Some(config) = self.shortcuts.remove(id) {
            // 如果已经有应用句柄，注销这个快捷键
            if let Some(app) = &self.app_handle {
                match app.global_shortcut().unregister(config.shortcut) {
                    Ok(_) => info!("成功注销快捷键: ID={}, 描述={}", id, config.description),
                    Err(e) => {
                        error!(
                            "注销快捷键失败: ID={}, 描述={}, 错误: {:?}",
                            id, config.description, e
                        );
                        return Err(AppError::new_unknown(format!("注销快捷键失败: {}", e)));
                    }
                }
            } else {
                warn!("应用句柄未设置，无法注销快捷键: ID={}", id);
            }
        } else {
            warn!("尝试移除不存在的快捷键: ID={}", id);
        }
        Ok(())
    }

    /// 获取当前所有快捷键配置
    pub fn get_all_shortcuts(&self) -> Vec<(&String, &ShortcutConfig)> {
        self.shortcuts.iter().collect()
    }

    /// 处理快捷键事件
    pub fn handle_shortcut(
        &self,
        app: &tauri::AppHandle,
        shortcut: &Shortcut,
        event: ShortcutEvent,
    ) {
        // 按下后立即触发
        if event.state != ShortcutState::Pressed {
            return;
        }

        // 查找并执行对应的处理函数
        for (id, config) in &self.shortcuts {
            if &config.shortcut == shortcut {
                info!("触发快捷键: ID={}, 描述={}", id, config.description);
                (config.handler)(app);
                break;
            }
        }
    }

    /// 注册所有快捷键
    pub fn register_all(&self) -> Result<(), AppError> {
        info!("开始注册所有快捷键");
        if let Some(app) = &self.app_handle {
            for (id, config) in &self.shortcuts {
                match app.global_shortcut().register(config.shortcut) {
                    Ok(_) => info!("成功注册快捷键: ID={}, 描述={}", id, config.description),
                    Err(e) => {
                        error!(
                            "注册快捷键失败: ID={}, 描述={}, 错误: {:?}",
                            id, config.description, e
                        );
                        return Err(AppError::new_unknown(format!("注册快捷键失败: {}", e)));
                    }
                }
            }
            info!("所有快捷键注册完成，共 {} 个", self.shortcuts.len());
        } else {
            error!("应用句柄未设置，无法注册快捷键");
            return Err(AppError::new_validation_error("App handle not set"));
        }
        Ok(())
    }

    /// 注销所有快捷键
    pub fn unregister_all(&self) -> Result<(), AppError> {
        info!("开始注销所有快捷键");
        if let Some(app) = &self.app_handle {
            let mut success_count = 0;
            let mut fail_count = 0;
            for (id, config) in &self.shortcuts {
                if let Err(e) = app.global_shortcut().unregister(config.shortcut) {
                    fail_count += 1;
                    warn!(
                        "注销快捷键失败: ID={}, 描述={}, 错误: {:?}",
                        id, config.description, e
                    );
                } else {
                    success_count += 1;
                    info!("成功注销快捷键: ID={}, 描述={}", id, config.description);
                }
            }
            info!(
                "快捷键注销完成，成功: {}，失败: {}",
                success_count, fail_count
            );
            Ok(())
        } else {
            error!("应用句柄未设置，无法注销快捷键");
            Err(AppError::new_validation_error("App handle not set"))
        }
    }

    /// 刷新所有快捷键
    pub fn refresh_all(&self) -> Result<(), AppError> {
        info!("开始刷新所有快捷键");
        self.unregister_all()?;
        self.register_all()?;
        info!("所有快捷键刷新成功，共 {} 个", self.shortcuts.len());
        Ok(())
    }
}

/// 处理全局快捷键事件
pub fn global_shortcuts_handle(app: &tauri::AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    match ShortcutManager::instance().read() {
        Ok(manager) => {
            manager.handle_shortcut(app, shortcut, event);
        }
        Err(e) => {
            error!("获取快捷键管理器失败，无法处理快捷键事件: {:?}", e);
        }
    }
}

/// 注册全局快捷键
pub fn register_shortcuts(app: &App) -> Result<(), AppError> {
    info!("开始初始化全局快捷键系统");

    // Use SHORTCUT_MANAGER directly instead of a temporary variable
    match SHORTCUT_MANAGER.write() {
        Ok(mut manager) => {
            // 设置应用句柄
            manager.set_app_handle(app.handle());

            // 注册所有快捷键
            match manager.register_all() {
                Ok(_) => {
                    info!("全局快捷键系统初始化完成");
                    Ok(())
                }
                Err(e) => {
                    error!("全局快捷键注册失败: {:?}", e);
                    Err(e)
                }
            }
        }
        Err(e) => {
            error!("获取快捷键管理器写锁失败: {:?}", e);
            Err(AppError::new_unknown(format!(
                "获取快捷键管理器写锁失败: {:?}",
                e
            )))
        }
    }
}

/// 注销所有已注册的快捷键
pub fn unregister_all_shortcuts(app: &tauri::AppHandle) -> Result<(), AppError> {
    info!("开始注销所有全局快捷键");
    match ShortcutManager::instance().read() {
        Ok(manager) => match manager.unregister_all() {
            Ok(_) => {
                info!("所有全局快捷键已成功注销");
                Ok(())
            }
            Err(e) => {
                error!("注销全局快捷键失败: {:?}", e);
                Err(e)
            }
        },
        Err(e) => {
            error!("获取快捷键管理器读锁失败: {:?}", e);
            Err(AppError::new_unknown(format!(
                "获取快捷键管理器读锁失败: {:?}",
                e
            )))
        }
    }
}

/// 刷新全局快捷键注册
pub fn refresh_shortcuts(app: &tauri::AppHandle) -> Result<(), AppError> {
    info!("开始刷新所有全局快捷键");
    match ShortcutManager::instance().read() {
        Ok(manager) => match manager.refresh_all() {
            Ok(_) => {
                info!("所有全局快捷键已成功刷新");
                Ok(())
            }
            Err(e) => {
                error!("刷新全局快捷键失败: {:?}", e);
                Err(e)
            }
        },
        Err(e) => {
            error!("获取快捷键管理器读锁失败: {:?}", e);
            Err(AppError::new_unknown(format!(
                "获取快捷键管理器读锁失败: {:?}",
                e
            )))
        }
    }
}
