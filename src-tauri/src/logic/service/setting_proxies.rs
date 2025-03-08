// file_path: src/logic/service/setting_proxies.rs
use crate::infrastructure::data_access::curd_proxies;
use crate::infrastructure::data_access::curd_windows;
use crate::infrastructure::db::DB;
use crate::infrastructure::error::AppError;
use crate::types::proxies::{CreateProxyRequest, ProxyInfo, UpdateProxyRequest};

// 设置页添加代理
pub struct ProxyInfoService;

impl ProxyInfoService {
    // 添加新代理
    pub async fn create(
        proxy_type: String,
        host: String,
        port: i32,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<ProxyInfo, AppError> {
        let db = DB::get_connection().await?;

        // 创建代理
        let proxy =
            curd_proxies::Insert::create(&db, proxy_type, host, port, username, password).await?;

        // 转换为返回类型
        Ok(ProxyInfo {
            id: proxy.id,
            r#type: proxy.r#type,
            host: proxy.host,
            port: proxy.port,
            username: proxy.username,
            password: proxy.password,
            created_at: proxy.created_at,
            updated_at: proxy.updated_at,
        })
    }

    // 从请求创建代理
    pub async fn create_from_request(request: CreateProxyRequest) -> Result<ProxyInfo, AppError> {
        Self::create(
            request.r#type,
            request.host,
            request.port,
            request.username,
            request.password,
        )
        .await
    }

    // 更新代理
    pub async fn update(request: UpdateProxyRequest) -> Result<ProxyInfo, AppError> {
        let db = DB::get_connection().await?;

        // 更新代理
        let proxy = curd_proxies::Update::update(
            &db,
            request.id,
            request.r#type,
            request.host,
            request.port,
            request.username,
            request.password,
        )
        .await?;

        // 转换为返回类型
        Ok(ProxyInfo {
            id: proxy.id,
            r#type: proxy.r#type,
            host: proxy.host,
            port: proxy.port,
            username: proxy.username,
            password: proxy.password,
            created_at: proxy.created_at,
            updated_at: proxy.updated_at,
        })
    }

    // 删除代理（同时删除与之关联的窗口）
    pub async fn delete(id: i32) -> Result<bool, AppError> {
        let db = DB::get_connection().await?;

        // 首先删除与代理关联的所有窗口
        curd_windows::Delete::delete_by_proxy_id(&db, id).await?;

        // 然后删除代理
        curd_proxies::Delete::delete(&db, id).await
    }

    // 获取所有代理
    pub async fn get_all() -> Result<Vec<ProxyInfo>, AppError> {
        let db = DB::get_connection().await?;
        let proxies = curd_proxies::Query::find_all(&db).await?;

        let proxy_infos = proxies
            .into_iter()
            .map(|proxy| ProxyInfo {
                id: proxy.id,
                r#type: proxy.r#type,
                host: proxy.host,
                port: proxy.port,
                username: proxy.username,
                password: proxy.password,
                created_at: proxy.created_at,
                updated_at: proxy.updated_at,
            })
            .collect();

        Ok(proxy_infos)
    }

    // 获取单个代理
    pub async fn get_by_id(id: i32) -> Result<ProxyInfo, AppError> {
        let db = DB::get_connection().await?;
        let proxy = curd_proxies::Query::find_by_id_or_error(&db, id).await?;

        Ok(ProxyInfo {
            id: proxy.id,
            r#type: proxy.r#type,
            host: proxy.host,
            port: proxy.port,
            username: proxy.username,
            password: proxy.password,
            created_at: proxy.created_at,
            updated_at: proxy.updated_at,
        })
    }
}
