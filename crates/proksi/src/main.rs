use std::sync::Arc;
use anyhow::{Result};
use clap::{crate_version, ArgMatches, Command};
use pingora::server::configuration::ServerConf as PingoraServerConf;
use pingora::server::Server as PingoraServer;
use tokio::signal;
use tracing::{error, info};

use proksi::config::load as load_config;
use proksi::http_server::create_server;
use proksi::monitor::init_prometheus;
use proksi::plugins::api_server::{ApiServerConfig, ApiServerPlugin};
use proksi::plugins::core::Plugin;
use proksi::plugins::manager::PluginManager;

pub const SERVER_NAME: &str = "proksi";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let matches = Command::new(SERVER_NAME)
        .version(crate_version!())
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommands([Command::new("start").about("Start the server")])
        .get_matches();

    match matches.subcommand() {
        Some(("start", sub_matches)) => start_server(sub_matches).await,
        _ => unreachable!(),
    }
}

async fn start_server(_matches: &ArgMatches) -> Result<()> {
    // Load configuration
    let config = Arc::new(load_config("proksi.toml")?);

    // Initialize Prometheus metrics
    init_prometheus();

    // Create plugin manager
    let plugin_manager = Arc::new(PluginManager::new());

    // Initialize HTTP server
    let http_server = create_server(config.clone(), plugin_manager.clone())?;

    // Configure Pingora server
    let mut pingora_server = PingoraServer::new(None)?;
    let mut server_conf = PingoraServerConf::default();
    
    // Configure server with proper fields from config
    if let Some(threads) = config.worker_threads {
        server_conf.threads = threads;
    }
    server_conf.daemon = config.daemon;
    server_conf.upgrade_sock = "/tmp/proksi_upgrade.sock".to_string();
    server_conf.error_log = None; // Using our own logging system
    
    // Add services to server
    pingora_server.add_services(vec![http_server]);

    // Initialize and register plugins
    if let Some(plugin_configs) = &config.plugins {
        // Initialize API server plugin if enabled
        if let Some(api_config) = &plugin_configs.api_server {
            if api_config.enabled {
                let plugin_config = ApiServerConfig {
                    listen_addr: api_config.listen_address.clone()
                        .unwrap_or_else(|| "127.0.0.1:8080".to_string()),
                    access_log: api_config.enable_access_log.unwrap_or(true),
                    cors: api_config.enable_cors.unwrap_or(true),
                };

                // Create and initialize plugin
                let mut plugin = ApiServerPlugin::new(plugin_config).await?;
                plugin = plugin.with_system_config(config.clone());
                
                // Start the plugin
                plugin.start().await?;
                
                // Register the initialized plugin
                plugin_manager.register(plugin);
            }
        }

        // Initialize other plugins here as needed
    }

    // Run server
    info!("Starting {} server...", SERVER_NAME);
    tokio::spawn(async move {
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen for ctrl-c signal: {}", e);
        }
        std::process::exit(0);
    });

    pingora_server.run_forever();
    Ok(())
}
