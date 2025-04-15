use std::sync::Arc;
use anyhow::Result;
use pingora::proxy::Session;
use tracing::{debug, warn, trace};

use crate::proxy_server::https_proxy::RouterContext;
use crate::proxy_server::HttpResponse;
use crate::plugins::core::{Plugin, PluginStep};

/// Execute plugins at a specific step in the request/response lifecycle
///
/// This function runs all plugins in order of priority and respects the plugin's response
/// to determine whether to continue processing.
///
/// Returns:
/// - `handled`: true if any plugin handled the request and processing should stop
/// - `response`: optional HTTP response to return to the client
pub async fn execute_plugins(
    step: PluginStep,
    session: &mut Session,
    ctx: &mut RouterContext,
    plugins: &[Arc<dyn Plugin>],
) -> Result<(bool, Option<HttpResponse>)> {
    debug!("Executing plugins at step: {:?}, plugin count: {}", step, plugins.len());
    
    // Sort plugins by priority (lower values run first)
    let mut sorted_plugins = plugins.to_vec();
    sorted_plugins.sort_by_key(|p| p.metadata().priority);
    
    for plugin in sorted_plugins {
        let plugin_name = plugin.name();
        trace!("Running plugin '{}' at step {:?}", plugin_name, step);
        
        match step {
            PluginStep::EarlyRequest | PluginStep::Request | PluginStep::ProxyUpstream => {
                match plugin.handle_request(step, session, ctx).await {
                    Ok((handled, response)) => {
                        if handled {
                            debug!("Plugin '{}' handled the request", plugin_name);
                            return Ok((true, response));
                        }
                    },
                    Err(e) => {
                        warn!("Plugin '{}' error at step {:?}: {}", plugin_name, step, e);
                    }
                }
            },
            PluginStep::Response | PluginStep::ResponseBody => {
                if let Some(mut resp) = ctx.upstream_response.take() {
                    match plugin.handle_response(step, session, ctx, &mut resp).await {
                        Ok(modified) => {
                            if modified {
                                debug!("Plugin '{}' modified the response", plugin_name);
                            }
                            ctx.upstream_response = Some(resp);
                        },
                        Err(e) => {
                            warn!("Plugin '{}' error at step {:?}: {}", plugin_name, step, e);
                            ctx.upstream_response = Some(resp);
                        }
                    }
                }
            },
            PluginStep::Log => {
                // Log phase doesn't expect a response modification
                if let Err(e) = plugin.handle_request(step, session, ctx).await {
                    warn!("Plugin '{}' error at log step: {}", plugin_name, e);
                }
            }
        }
    }
    
    Ok((false, None))
}

/// Initialize all registered plugins
pub async fn initialize_plugins(
    plugins: &mut [Arc<dyn Plugin>],
) -> Result<()> {
    debug!("Initializing {} plugins", plugins.len());
    
    for plugin in plugins {
        let plugin_name = plugin.name();
        // Using get_mut to get mutable access to the Arc content
        match Arc::get_mut(plugin) {
            Some(plugin) => {
                debug!("Starting plugin '{}'", plugin_name);
                if let Err(e) = plugin.start().await {
                    warn!("Failed to start plugin '{}': {:?}", plugin_name, e);
                }
            },
            None => {
                warn!("Could not get mutable reference to plugin '{}'", plugin_name);
            }
        }
    }
    
    Ok(())
}

/// Stop all registered plugins
pub async fn shutdown_plugins(
    plugins: &mut [Arc<dyn Plugin>],
) -> Result<()> {
    debug!("Shutting down {} plugins", plugins.len());
    
    for plugin in plugins {
        let plugin_name = plugin.name();
        // Using get_mut to get mutable access to the Arc content
        match Arc::get_mut(plugin) {
            Some(plugin) => {
                debug!("Stopping plugin '{}'", plugin_name);
                if let Err(e) = plugin.stop().await {
                    warn!("Failed to stop plugin '{}': {:?}", plugin_name, e);
                }
            },
            None => {
                warn!("Could not get mutable reference to plugin '{}'", plugin_name);
            }
        }
    }
    
    Ok(())
}