use ::pingora::server::Server;

use bytes::Bytes;
use clap::crate_version;
use config::{load, LogFormat, RouteHeaderAdd, RouteHeaderRemove, RoutePlugin};
use tracing_subscriber::EnvFilter;

use std::{borrow::Cow, sync::Arc};

use pingora::{listeners::tls::TlsSettings, proxy::http_proxy_service, server::configuration::Opt};

use proxy_server::cert_store::CertStore;
use services::{logger::ProxyLoggerReceiver, BackgroundFunctionService};

use crate::config::Config;
use crate::plugins::api_server::{start_api_server, ApiServerConfig, ApiServerPlugin};
use crate::plugins::compliance::CompliancePlugin;
use crate::plugins::core::Plugin;
use crate::plugins::manager::PluginManager;
use crate::plugins::tenant::TenantPlugin;

mod cache;
mod channel;
mod config;
mod models;
mod plugins;
mod proxy_server;
mod services;
mod stores;
mod tools;
mod wasm;

#[derive(Clone, Default)]
pub struct MsgRoute {
    host: Cow<'static, str>,
    upstreams: Vec<String>,
    path_matchers: Vec<String>,
    host_headers_add: Vec<RouteHeaderAdd>,
    host_headers_remove: Vec<RouteHeaderRemove>,
    plugins: Vec<RoutePlugin>,

    self_signed_certs: bool,
}

#[derive(Clone)]
pub struct MsgCert {
    _cert: Bytes,
    _key: Bytes,
}

#[derive(Clone)]
pub enum MsgProxy {
    NewRoute(MsgRoute),
    NewCertificate(MsgCert),
    ConfigUpdate(()),
}

#[deny(
    clippy::all,
    clippy::pedantic,
    clippy::perf,
    clippy::correctness,
    clippy::style,
    clippy::suspicious,
    clippy::complexity
)]
fn main() -> Result<(), anyhow::Error> {
    // Configuration can be refreshed on file change

    // Loads configuration from command-line, YAML or TOML sources
    let proxy_config =
        Arc::new(load("/etc/proksi/configs").expect("Failed to load configuration: "));

    // 启动 API 服务器及其他插件
    let config_clone = proxy_config.clone();
    tokio::spawn(async move {
        if let Err(e) = initialize_plugins(config_clone).await {
            tracing::error!("Plugin initialization error: {}", e);
        }
    });

    let https_address = proxy_config
        .server
        .https_address
        .clone()
        .unwrap_or_default();
    let le_address = proxy_config.server.http_address.clone().unwrap_or_default();

    // Logging channel
    let (log_sender, log_receiver) = tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();

    // Receiver channel for Routes/Certificates/etc
    let (sender, mut _receiver) = tokio::sync::broadcast::channel::<MsgProxy>(10);
    let appender = services::logger::ProxyLog::new(
        log_sender,
        proxy_config.logging.enabled,
        proxy_config.logging.access_logs_enabled,
        proxy_config.logging.error_logs_enabled,
    );

    // Creates a tracing/logging subscriber based on the configuration provided
    if proxy_config.logging.format == LogFormat::Json {
        tracing_subscriber::fmt()
            .json()
            .with_env_filter(EnvFilter::from_default_env())
            .with_max_level(&proxy_config.logging.level)
            .with_writer(appender)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::from_default_env())
            .with_max_level(&proxy_config.logging.level)
            .with_ansi(proxy_config.logging.path.is_none())
            .with_writer(appender)
            .init();
    };

    // Pingora load balancer server
    let pingora_opts = Opt {
        daemon: proxy_config.daemon,
        upgrade: proxy_config.upgrade,
        conf: None,
        nocapture: false,
        test: false,
    };

    let mut pingora_server = Server::new(Some(pingora_opts))?;
    pingora_server.bootstrap();

    // Service: HTTP Load Balancer (only used by acme-challenges)
    // As we don't necessarily need an upstream to handle the acme-challenges,
    // we can use a simple mock LoadBalancer
    let mut http_public_service = http_proxy_service(
        &pingora_server.configuration,
        proxy_server::http_proxy::HttpLB {},
    );

    // Service: HTTPS Load Balancer (main service)
    // The router will also handle health checks and failover in case of upstream failure
    let router = proxy_server::https_proxy::Router {};
    let mut https_secure_service = http_proxy_service(&pingora_server.configuration, router);
    http_public_service.add_tcp(&le_address);

    // Worker threads per configuration
    https_secure_service.threads = proxy_config.worker_threads;

    // Setup tls settings and Enable HTTP/2
    let cert_store = CertStore::new();
    let mut tls_settings = TlsSettings::with_callbacks(Box::new(cert_store)).unwrap();
    tls_settings.enable_h2();

    // tls_settings.set_session_cache_mode(SslSessionCacheMode::SERVER);
    tls_settings.set_servername_callback(move |ssl_ref, _| CertStore::sni_callback(ssl_ref));

    // For now this is a hardcoded recommendation based on
    // https://developers.cloudflare.com/ssl/reference/protocols/
    // but will be made configurable in the future
    tls_settings.set_min_proto_version(Some(pingora::tls::ssl::SslVersion::TLS1_2))?;
    tls_settings.set_max_proto_version(Some(pingora::tls::ssl::SslVersion::TLS1_3))?;

    // Add TLS settings to the HTTPS service
    https_secure_service.add_tls_with_settings(&https_address, None, tls_settings);

    // Add Prometheus service
    // let mut prometheus_service_http = Service::prometheus_http_service();
    // prometheus_service_http.add_tcp("0.0.0.0:9090");
    // pingora_server.add_service(prometheus_service_http);

    // Non-dedicated background services
    pingora_server.add_service(BackgroundFunctionService::new(proxy_config.clone(), sender));

    // Dedicated logger service
    pingora_server.add_service(ProxyLoggerReceiver::new(log_receiver, proxy_config.clone()));

    // Listen on HTTP and HTTPS ports
    pingora_server.add_service(http_public_service);
    pingora_server.add_service(https_secure_service);

    let server_info = format!(
        "running HTTPS service on {} and HTTP service on {}",
        &https_address, &le_address
    );
    tracing::info!(
        version = crate_version!(),
        workers = proxy_config.worker_threads,
        server_info,
    );

    pingora_server.run_forever();

    Ok(())
}

/// 初始化和注册系统插件
async fn initialize_plugins(proxy_config: Arc<config::Config>) -> Result<(), Box<dyn std::error::Error>> {
    // 根据配置动态加载插件
    let plugin_manager = PluginManager::new();

    // 如果提供了插件配置
    if let Some(plugin_configs) = &proxy_config.plugins {
        // API 服务器插件
        if let Some(api_config_wrapper) = &plugin_configs.api_server {
            if api_config_wrapper.enabled {
                tracing::info!("Initializing API server plugin...");
                let actual_api_config = ApiServerConfig {
                    listen_addr: api_config_wrapper.listen_address.clone().unwrap_or_else(|| crate::plugins::api_server::default_listen_addr()),
                    access_log: api_config_wrapper.enable_access_log.unwrap_or(true),
                    cors: api_config_wrapper.enable_cors.unwrap_or(true),
                };
                let plugin_instance = ApiServerPlugin::new(actual_api_config);
                plugin_manager.register(plugin_instance);
            }
        }

        // 合规性插件
        if let Some(compliance_config_wrapper) = &plugin_configs.compliance {
            if compliance_config_wrapper.enabled {
                tracing::info!("Initializing compliance plugin...");
                let plugin_instance = CompliancePlugin::default();
                plugin_manager.register(plugin_instance);
            }
        }

        // 租户插件
        if let Some(_tenant_config_wrapper) = &plugin_configs.tenant {
            tracing::info!("Initializing tenant plugin...");
            let plugin_instance = TenantPlugin::default();
            plugin_manager.register(plugin_instance);
        }
        // ... 可以添加更多插件的初始化逻辑 ...
    } else {
        // 如果没有提供插件配置，则使用默认配置初始化必要的插件
        tracing::warn!("No plugin configurations found in config file. Initializing default plugins.");

        // 默认初始化 API 服务器插件
        let default_api_config = ApiServerConfig {
            listen_addr: crate::plugins::api_server::default_listen_addr(),
            access_log: true, // Assuming default
            cors: true,       // Assuming default
        };
        let plugin_instance = ApiServerPlugin::new(default_api_config);
        plugin_manager.register(plugin_instance);
        tracing::info!("API server plugin registered (default)");

        // 默认初始化合规性插件
        let plugin_instance = CompliancePlugin::default();
        plugin_manager.register(plugin_instance);
        tracing::info!("Compliance plugin registered (default)");

        // 默认初始化租户插件
        let plugin_instance = TenantPlugin::default();
        plugin_manager.register(plugin_instance);
        tracing::info!("Tenant plugin registered (default)");
    }

    // 启动所有已注册的插件
    // TODO: Implement a method in PluginManager to retrieve registered plugins for iteration
    // for plugin in plugin_manager.get_all_plugins() { // Example method name
    //     plugin.start().await?;
    // }

    // TODO: Decide how to manage the plugin_manager instance (e.g., store globally)

    Ok(())
}
