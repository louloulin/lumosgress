use std::borrow::Cow;
use crate::config::RoutePlugin;

/// Message types for internal communication
#[derive(Debug, Clone)]
pub enum MsgProxy {
    /// Message for new routes being added
    NewRoute(MsgRoute),
}

/// Route message structure for internal communication
#[derive(Debug, Clone)]
pub struct MsgRoute {
    /// The host name for this route
    pub host: String,
    /// Path patterns to match for this route
    pub path_matchers: Vec<String>,
    /// Headers to add to the host
    pub host_headers_add: Vec<(Cow<'static, str>, Cow<'static, str>)>,
    /// Headers to remove from the host
    pub host_headers_remove: Vec<Cow<'static, str>>,
    /// Upstream servers for this route
    pub upstreams: Vec<String>,
    /// Plugin configurations for this route
    pub plugins: Vec<RoutePlugin>,
    /// Whether to use self-signed certificates
    pub self_signed_certs: bool,
} 