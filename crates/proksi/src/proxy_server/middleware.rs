use std::collections::HashMap;

use pingora::Result;

use crate::plugins::MiddlewarePlugin;

/// Executes the request and response plugins
pub async fn execute_response_plugins(
    session: &mut pingora::proxy::Session,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
) -> Result<()> {
    for (name, value) in ctx.route_container.plugins.clone() {
        match name.as_str() {
            "oauth2" => {
                use crate::plugins::MiddlewarePlugin;

                if crate::plugins::PLUGINS
                    .oauth2
                    .response_filter(session, ctx, &value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(());
                }
            }
            "request_id" => continue,
            // AI Gateway plugins
            "llm_router" => {
                if crate::plugins::PLUGINS
                    .llm_router
                    .response_filter(session, ctx, &value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(());
                }
            },
            "prompt_transform" => {
                if crate::plugins::PLUGINS
                    .prompt_transform
                    .response_filter(session, ctx, &value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(());
                }
            },
            "ai_security" => {
                if crate::plugins::PLUGINS
                    .ai_security
                    .response_filter(session, ctx, &value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(());
                }
            },
            "llm_aggregator" => {
                if crate::plugins::PLUGINS
                    .llm_aggregator
                    .response_filter(session, ctx, &value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(());
                }
            },
            _ => {}
        }
    }
    Ok(())
}

/// Executes the request plugins
pub async fn execute_request_plugins(
    session: &mut pingora::proxy::Session,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
    plugins: &HashMap<String, crate::config::RoutePlugin>,
) -> Result<bool> {
    use crate::plugins::MiddlewarePlugin;
    for (name, value) in plugins {
        match name.as_str() {
            "oauth2" => {
                if crate::plugins::PLUGINS
                    .oauth2
                    .request_filter(session, ctx, value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(true);
                }
            }
            "request_id" => {
                crate::plugins::PLUGINS
                    .request_id
                    .request_filter(session, ctx, value)
                    .await
                    .ok();
            }
            "basic_auth" => {
                if crate::plugins::PLUGINS
                    .basic_auth
                    .request_filter(session, ctx, value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(true);
                }
            }
            // AI Gateway plugins
            "llm_router" => {
                if crate::plugins::PLUGINS
                    .llm_router
                    .request_filter(session, ctx, value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(true);
                }
            },
            "prompt_transform" => {
                if crate::plugins::PLUGINS
                    .prompt_transform
                    .request_filter(session, ctx, value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(true);
                }
            },
            "ai_security" => {
                if crate::plugins::PLUGINS
                    .ai_security
                    .request_filter(session, ctx, value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(true);
                }
            },
            "llm_aggregator" => {
                if crate::plugins::PLUGINS
                    .llm_aggregator
                    .request_filter(session, ctx, value)
                    .await
                    .is_ok_and(|v| v)
                {
                    return Ok(true);
                }
            },
            _ => {}
        }
    }
    Ok(false)
}

/// Executes the upstream request plugins
pub async fn execute_upstream_request_plugins(
    session: &mut pingora::proxy::Session,
    upstream_request: &mut pingora::http::RequestHeader,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
) -> Result<()> {
    for (name, value) in ctx.route_container.plugins.clone() {
        match name.as_str() {
            "request_id" => {
                crate::plugins::PLUGINS
                    .request_id
                    .upstream_request_filter(session, upstream_request, ctx)
                    .await
                    .ok();
            }
            // AI Gateway plugins
            "llm_router" => {
                crate::plugins::PLUGINS
                    .llm_router
                    .upstream_request_filter(session, upstream_request, ctx)
                    .await
                    .ok();
            },
            "prompt_transform" => {
                crate::plugins::PLUGINS
                    .prompt_transform
                    .upstream_request_filter(session, upstream_request, ctx)
                    .await
                    .ok();
            },
            "ai_security" => {
                crate::plugins::PLUGINS
                    .ai_security
                    .upstream_request_filter(session, upstream_request, ctx)
                    .await
                    .ok();
            },
            "llm_aggregator" => {
                crate::plugins::PLUGINS
                    .llm_aggregator
                    .upstream_request_filter(session, upstream_request, ctx)
                    .await
                    .ok();
            },
            "other" => continue,
            _ => {}
        }
    }
    Ok(())
}

/// Executes the upstream response plugins
pub fn execute_upstream_response_plugins(
    session: &mut pingora::proxy::Session,
    upstream_response: &mut pingora::http::ResponseHeader,
    ctx: &mut crate::proxy_server::https_proxy::RouterContext,
) {
    for (name, value) in ctx.route_container.plugins.clone() {
        match name.as_str() {
            "request_id" => {
                crate::plugins::PLUGINS
                    .request_id
                    .upstream_response_filter(session, upstream_response, ctx)
                    .ok();
            }
            // AI Gateway plugins
            "llm_router" => {
                crate::plugins::PLUGINS
                    .llm_router
                    .upstream_response_filter(session, upstream_response, ctx)
                    .ok();
            },
            "prompt_transform" => {
                crate::plugins::PLUGINS
                    .prompt_transform
                    .upstream_response_filter(session, upstream_response, ctx)
                    .ok();
            },
            "ai_security" => {
                crate::plugins::PLUGINS
                    .ai_security
                    .upstream_response_filter(session, upstream_response, ctx)
                    .ok();
            },
            "llm_aggregator" => {
                crate::plugins::PLUGINS
                    .llm_aggregator
                    .upstream_response_filter(session, upstream_response, ctx)
                    .ok();
            },
            "other" => continue,
            _ => {}
        }
    }
}
