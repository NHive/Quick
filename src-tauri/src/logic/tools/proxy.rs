// file_path: src/logic/tools/proxy.rs
use crate::logic::service::setting_proxies::ProxyInfoService;
use log;
use url::Url;

/// 根据代理ID获取格式化的代理URL
pub async fn get_proxy_url(proxy_id: i32) -> Option<Url> {
    // 获取代理信息
    let proxy_info_result = ProxyInfoService::get_by_id(proxy_id).await;

    match proxy_info_result {
        Ok(proxy) => {
            // 构建代理URL
            let proxy_url_str = match proxy.r#type.as_str() {
                "http" => format!("http://{}:{}", proxy.host, proxy.port),
                "socks5" => format!("socks5://{}:{}", proxy.host, proxy.port),
                _ => {
                    log::warn!("不支持的代理类型: {}", proxy.r#type);
                    String::new()
                }
            };

            // 如果有用户名和密码，添加认证信息
            let proxy_url_with_auth = if !proxy_url_str.is_empty() {
                if let (Some(username), Some(password)) = (proxy.username, proxy.password) {
                    // 将认证信息添加到URL中
                    if let Ok(mut url) = Url::parse(&proxy_url_str) {
                        if url.set_username(&username).is_err() {
                            log::warn!("无法设置代理用户名");
                        }
                        if url.set_password(Some(&password)).is_err() {
                            log::warn!("无法设置代理密码");
                        }
                        url.to_string()
                    } else {
                        proxy_url_str
                    }
                } else {
                    proxy_url_str
                }
            } else {
                String::new()
            };

            // 解析和应用代理配置
            if !proxy_url_with_auth.is_empty() {
                if let Ok(parsed_url) = Url::parse(&proxy_url_with_auth) {
                    return Some(parsed_url);
                } else {
                    log::error!("代理URL格式无效: {}", proxy_url_with_auth);
                }
            }
        }
        Err(err) => {
            log::warn!("获取代理配置失败 (ID: {}): {}", proxy_id, err);
        }
    }

    None
}
