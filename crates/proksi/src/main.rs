use ::pingora::server::Server;

use bytes::Bytes;
use clap::crate_version;
use config::{load, LogFormat, RouteHeaderAdd, RouteHeaderRemove, RoutePlugin};
use tracing_subscriber::EnvFilter;

use std::{borrow::Cow, sync::Arc};

use pingora::{listeners::tls::TlsSettings, proxy::http_proxy_service, server::configuration::Opt};

use proxy_server::cert_store::CertStore;
use services::{logger::ProxyLoggerReceiver, BackgroundFunctionService};

use plugins::{Plugin, tenant::TenantPlugin, compliance::CompliancePlugin, api_server::ApiServerPlugin};
use models::tenant::{ResourceQuota, ResourceUsage, TenantStatus};

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
    if let Some(plugins_config) = &proxy_config.plugins {
        // 租户插件
        if let Some(tenant_config) = &plugins_config.tenant {
            if tenant_config.enabled {
                tracing::info!("Initializing tenant plugin...");
                let tenant_plugin = TenantPlugin::new(plugins::tenant::TenantPluginConfig {
                    default_quota: ResourceQuota { 
                        requests: tenant_config.default_requests.unwrap_or(1000) as u64, 
                        tokens: tenant_config.default_tokens.unwrap_or(10000) as u64 
                    },
                    isolation_enabled: tenant_config.isolation_enabled.unwrap_or(true),
                }).await?;
                plugins::manager::register(tenant_plugin);
                tracing::info!("Tenant plugin registered");
            }
        }
        
        // 合规插件
        if let Some(compliance_config) = &plugins_config.compliance {
            if compliance_config.enabled {
                tracing::info!("Initializing compliance plugin...");
                let compliance_plugin = CompliancePlugin::new(plugins::compliance::ComplianceConfig {
                    retention_days: 90,
                    enabled: true,
                    storage_path: "/var/log/proksi/compliance".to_string(),
                })
                .await
                .unwrap();
                plugins::manager::register(compliance_plugin);
                tracing::info!("Compliance plugin registered");
            }
        }
        
        // API服务器插件
        if let Some(api_config) = &plugins_config.api_server {
            if api_config.enabled {
                tracing::info!("Initializing API server plugin...");
                let plugin_config = plugins::api_server::ApiServerConfig {
                    listen_address: api_config.listen_address.clone().unwrap_or_else(|| "127.0.0.1:8080".to_string()),
                    enable_access_log: api_config.enable_access_log.unwrap_or(true),
                    enable_cors: api_config.enable_cors.unwrap_or(true),
                };
                
                let mut api_server_plugin = ApiServerPlugin::new(plugin_config).await?;
                api_server_plugin = api_server_plugin.with_system_config(proxy_config.clone());
                
                // 启动API服务器
                if let Err(e) = api_server_plugin.start().await {
                    tracing::error!("API server start error: {}", e);
                } else {
                    plugins::manager::register(api_server_plugin);
                    tracing::info!("API server plugin registered and started");
                }
            }
        }
    } else {
        // 如果没有显式配置插件，使用默认配置初始化核心插件
        tracing::info!("No plugin configuration found, initializing with defaults...");
        
        // 默认初始化租户插件
        let tenant_plugin = TenantPlugin::new(plugins::tenant::TenantPluginConfig {
            default_quota: ResourceQuota { requests: 1000, tokens: 10000 },
            isolation_enabled: true,
        }).await?;
        plugins::manager::register(tenant_plugin);
        tracing::info!("Tenant plugin registered (default)");
        
        // 默认初始化合规插件
        let compliance_plugin = CompliancePlugin::new(plugins::compliance::ComplianceConfig {
            retention_days: 90,
            enabled: true,
            storage_path: "/var/log/proksi/compliance".to_string(),
        })
        .await
        .unwrap();
        plugins::manager::register(compliance_plugin);
        tracing::info!("Compliance plugin registered (default)");
        
        // 默认初始化API服务器插件
        let api_config = plugins::api_server::ApiServerConfig {
            listen_address: "127.0.0.1:8080".to_string(),
            enable_access_log: true,
            enable_cors: true,
        };
        let mut api_server_plugin = ApiServerPlugin::new(api_config).await?;
        api_server_plugin = api_server_plugin.with_system_config(proxy_config);
        
        if let Err(e) = api_server_plugin.start().await {
            tracing::error!("API server start error: {}", e);
        } else {
            plugins::manager::register(api_server_plugin);
            tracing::info!("API server plugin registered and started (default)");
        }
    }

    // 输出已加载的插件列表
    let plugins = plugins::manager::list_plugins();
    tracing::info!("Loaded plugins: {:?}", plugins);

    Ok(())
}
