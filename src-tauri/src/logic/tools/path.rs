// file_path: src/logic/tools/path.rs
use crate::infrastructure::error::AppError;
use directories::ProjectDirs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct AppPath {
    pub base_path: PathBuf,
    #[allow(dead_code)]
    pub temp_path: PathBuf,
}

impl AppPath {
    pub fn new() -> Result<Self, AppError> {
        let (base_path, temp_path) = if cfg!(debug_assertions) {
            // 开发环境使用特定的测试目录
            let base_dir = std::env::current_dir()?.join("debug_data");
            let temp_dir = base_dir.join("temp");
            (base_dir, temp_dir)
        } else {
            // 生产环境使用标准的应用目录
            let proj_dirs = ProjectDirs::from("com", "Newbee", "Quick").unwrap();

            (
                proj_dirs.data_dir().to_path_buf(),
                proj_dirs.cache_dir().to_path_buf(),
            )
        };

        // 确保目录存在
        std::fs::create_dir_all(&base_path)?;
        std::fs::create_dir_all(&temp_path)?;

        Ok(Self {
            base_path,
            temp_path,
        })
    }
}
