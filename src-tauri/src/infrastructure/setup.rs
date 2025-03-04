use super::entity::prelude::*;
use super::entity::setup;
use super::entity::setup::ActiveModel;

use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter};
use sea_orm::{DbErr, Set};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::datetime;
use super::db::DB;
use super::error::AppError;

#[derive(Clone)]
pub struct SetupService {
    cache: Arc<RwLock<HashMap<String, String>>>,
}

impl Default for SetupService {
    fn default() -> Self {
        Self::new()
    }
}

impl SetupService {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    async fn fetch_from_db(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = DB::get_connection().await?;
        if let Some(setup) = Setup::find()
            .filter(setup::Column::Key.eq(key))
            .one(&conn)
            .await?
        {
            Ok(Some(setup.value))
        } else {
            Ok(None)
        }
    }

    pub async fn get_setup_async(&self, key: &str) -> Result<Option<Value>, AppError> {
        if let Some(cached_value) = self.cache.read().await.get(key) {
            let value: Value =
                serde_json::from_str(cached_value).map_err(|e| DbErr::Custom(e.to_string()))?;
            return Ok(Some(value));
        }

        if let Some(db_value) = self.fetch_from_db(key).await? {
            let value: Value =
                serde_json::from_str(&db_value).map_err(|e| DbErr::Custom(e.to_string()))?;
            self.cache.write().await.insert(key.to_string(), db_value);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    pub async fn set_setup_async(&self, key: &str, value: &Value) -> Result<(), AppError> {
        let value_str = serde_json::to_string(value).map_err(|e| DbErr::Custom(e.to_string()))?;
        let conn = DB::get_connection().await?;
        let setup = Setup::find()
            .filter(setup::Column::Key.eq(key))
            .one(&conn)
            .await?;

        match setup {
            Some(existing_setup) => {
                let mut existing_setup: ActiveModel = existing_setup.into();

                existing_setup.value = Set(value_str.clone());
                existing_setup.updated_at = Set(Some(datetime::get_datetime().naive_utc()));

                existing_setup.update(&conn).await?;
            }
            None => {
                let new_setup = ActiveModel {
                    key: Set(key.to_string()),
                    value: Set(value_str.clone()),
                    updated_at: Set(Some(datetime::get_datetime().naive_utc())),
                    ..Default::default()
                };
                new_setup.insert(&conn).await?;
            }
        }

        self.cache.write().await.insert(key.to_string(), value_str);

        Ok(())
    }

    pub async fn get_all_setups_async(&self) -> Result<HashMap<String, Value>, AppError> {
        let conn = DB::get_connection().await?;
        let setups = Setup::find().all(&conn).await?;
        let mut result = HashMap::new();

        for setup in setups {
            let value: Value =
                serde_json::from_str(&setup.value).map_err(|e| DbErr::Custom(e.to_string()))?;
            result.insert(setup.key, value);
        }

        Ok(result)
    }

    pub fn get_setup(&self, key: &str) -> Result<Option<Value>, AppError> {
        tauri::async_runtime::block_on(self.get_setup_async(key))
    }

    pub fn set_setup(&self, key: &str, value: &Value) -> Result<(), AppError> {
        tauri::async_runtime::block_on(self.set_setup_async(key, value))
    }

    #[allow(dead_code)]
    pub fn get_all_setups(&self) -> Result<HashMap<String, Value>, AppError> {
        tauri::async_runtime::block_on(self.get_all_setups_async())
    }
}
