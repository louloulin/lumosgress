use std::collections::HashMap;

use pingora::Result;

/// Executes the request and response plugins
#[allow(unused_variables)] // Temporarily allow unused vars until logic is replaced
pub async fn execute_response_plugins(
    session: &mut pingora::proxy::Session,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
) -> Result<()> {
    tracing::debug!("Executing response plugins (Old middleware - logic needs update)");
    // TODO: Replace this entire function body with calls to executor::execute_plugins for PluginStep::Response and PluginStep::Log
    /*
    for (name, value) in ctx.route_container.plugins.clone() {
        match name.as_str() {
            "oauth2" => {
                // use crate::plugins::MiddlewarePlugin;

                // if crate::plugins::PLUGINS
                //     .oauth2
                //     .response_filter(session, ctx, &value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(());
                // }
            }
            "request_id" => continue,
            // AI Gateway plugins
            "llm_router" => {
                // if crate::plugins::PLUGINS
                //     .llm_router
                //     .response_filter(session, ctx, &value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(());
                // }
            },
            "prompt_transform" => {
                // if crate::plugins::PLUGINS
                //     .prompt_transform
                //     .response_filter(session, ctx, &value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(());
                // }
            },
            "ai_security" => {
                // if crate::plugins::PLUGINS
                //     .ai_security
                //     .response_filter(session, ctx, &value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(());
                // }
            },
            "llm_aggregator" => {
                // if crate::plugins::PLUGINS
                //     .llm_aggregator
                //     .response_filter(session, ctx, &value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(());
                // }
            },
            _ => {}
        }
    }
    */
    Ok(())
}

/// Executes the request plugins
#[allow(unused_variables)] // Temporarily allow unused vars until logic is replaced
pub async fn execute_request_plugins(
    session: &mut pingora::proxy::Session,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
    plugins: &HashMap<String, crate::config::RoutePlugin>,
) -> Result<bool> {
    tracing::debug!("Executing request plugins (Old middleware - logic needs update)");
    // TODO: Replace this entire function body with calls to executor::execute_plugins for PluginStep::Request
    /*
    for (name, value) in plugins {
        match name.as_str() {
            "oauth2" => {
                // use crate::plugins::MiddlewarePlugin;

                // if crate::plugins::PLUGINS
                //     .oauth2
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            }
            "request_id" => {
                // if crate::plugins::PLUGINS
                //     .request_id
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            }
            "basic_auth" => {
                // if crate::plugins::PLUGINS
                //     .basic_auth
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            }
            // AI Gateway plugins
            "llm_router" => {
                // if crate::plugins::PLUGINS
                //     .llm_router
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "prompt_transform" => {
                // if crate::plugins::PLUGINS
                //     .prompt_transform
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "ai_security" => {
                // if crate::plugins::PLUGINS
                //     .ai_security
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "llm_aggregator" => {
                // if crate::plugins::PLUGINS
                //     .llm_aggregator
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "vector_db" => {
                // if crate::plugins::PLUGINS
                //     .vector_db
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "ai_analytics" => {
                // if crate::plugins::PLUGINS
                //     .ai_analytics
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "ai_request_builder" => {
                // if crate::plugins::PLUGINS
                //     .ai_request_builder
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "prompt_debugger" => {
                // if crate::plugins::PLUGINS
                //     .prompt_debugger
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "performance_analyzer" => {
                // if crate::plugins::PLUGINS
                //     .performance_analyzer
                //     .request_filter(session, ctx, value)
                //     .await
                //     .is_ok_and(|v| v)
                // {
                //     return Ok(true);
                // }
            },
            "other" => continue,
            _ => {}
        }
    }
    */
    Ok(false)
}

/// Executes the upstream request plugins
#[allow(unused_variables)] // Temporarily allow unused vars until logic is replaced
pub async fn execute_upstream_request_plugins(
    session: &mut pingora::proxy::Session,
    upstream_request: &mut pingora::http::RequestHeader,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
) -> Result<()> {
    tracing::debug!("Executing upstream request plugins (Old middleware - logic needs update)");
    // TODO: Replace this entire function body with calls to executor::execute_plugins for PluginStep::ProxyUpstream
    /*
    for (name, value) in ctx.route_container.plugins.clone() {
        match name.as_str() {
            "request_id" => {
                // crate::plugins::PLUGINS
                //     .request_id
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            }
            // AI Gateway plugins
            "llm_router" => {
                // crate::plugins::PLUGINS
                //     .llm_router
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "prompt_transform" => {
                // crate::plugins::PLUGINS
                //     .prompt_transform
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "ai_security" => {
                // crate::plugins::PLUGINS
                //     .ai_security
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "llm_aggregator" => {
                // crate::plugins::PLUGINS
                //     .llm_aggregator
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "vector_db" => {
                // crate::plugins::PLUGINS
                //     .vector_db
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "ai_analytics" => {
                // crate::plugins::PLUGINS
                //     .ai_analytics
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "ai_request_builder" => {
                // crate::plugins::PLUGINS
                //     .ai_request_builder
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "prompt_debugger" => {
                // crate::plugins::PLUGINS
                //     .prompt_debugger
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            "performance_analyzer" => {
                // crate::plugins::PLUGINS
                //     .performance_analyzer
                //     .upstream_request_filter(session, upstream_request, ctx)
                //     .await
                //     .ok();
            },
            _ => {}
        }
    }
    */
    Ok(())
}

/// Executes the upstream response plugins
#[allow(unused_variables)] // Temporarily allow unused vars until logic is replaced
pub fn execute_upstream_response_plugins(
    session: &mut pingora::proxy::Session,
    upstream_response: &mut pingora::http::ResponseHeader,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
) {
    tracing::debug!("Executing upstream response plugins (Old middleware - logic needs update)");
    // TODO: Replace this entire function body with calls to executor::execute_plugins for PluginStep::Response
    /*
    for (name, value) in ctx.route_container.plugins.clone() {
        match name.as_str() {
            "oauth2" => {
                // use crate::plugins::MiddlewarePlugin;
                // let _ = crate::plugins::PLUGINS
                //     .oauth2
                //     .response_filter(session, ctx, &value);
            }
            "request_id" => {
                // let _ = crate::plugins::PLUGINS
                //     .request_id
                //     .response_filter(session, ctx, &value);
            }
            // AI Gateway plugins
            "llm_router" => {
                // let _ = crate::plugins::PLUGINS
                //     .llm_router
                //     .response_filter(session, ctx, &value);
            },
            "prompt_transform" => {
                // let _ = crate::plugins::PLUGINS
                //     .prompt_transform
                //     .response_filter(session, ctx, &value);
            },
            "ai_security" => {
                // let _ = crate::plugins::PLUGINS
                //     .ai_security
                //     .response_filter(session, ctx, &value);
            },
            "llm_aggregator" => {
                // let _ = crate::plugins::PLUGINS
                //     .llm_aggregator
                //     .response_filter(session, ctx, &value);
            },
            _ => {}
        }
    }
    */
}
