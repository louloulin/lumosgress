use std::{borrow::Cow, collections::HashMap, sync::Arc};

use http::{HeaderName, HeaderValue};
use path_tree::PathTree;
use pingora::lb::{selection::RoundRobin, LoadBalancer};

use crate::config::{RouteCache, RoutePlugin, RouteUpstream, RouteMatcher, RoutePathMatcher};

#[derive(Debug, Default, Clone)]
pub struct RouteStorePathMatcher {
    pub pattern: Option<PathTree<usize>>,
}

impl RouteStorePathMatcher {
    pub fn new() -> Self {
        RouteStorePathMatcher::default()
    }

    // From a given list of patterns, generate a tree structure
    // to match against incoming requests
    pub fn with_pattern(&mut self, pattern: &[Cow<'_, str>]) -> &mut Self {
        if pattern.is_empty() {
            return self;
        }

        let mut path_tree = PathTree::new();
        for (index, value) in pattern.iter().enumerate() {
            let _ = path_tree.insert(value, index);
        }

        self.pattern = Some(path_tree);
        self
    }
}

#[derive(Clone)]
pub struct RouteStoreContainer {
    pub load_balancer: Arc<LoadBalancer<RoundRobin>>,
    pub match_with: Option<RouteMatcher>,
    pub host_header_remove: Vec<String>,
    pub host_header_add: Vec<(HeaderName, HeaderValue)>,

    pub upstreams: Vec<RouteUpstream>,
    pub self_signed_certificate: bool,

    pub plugins: HashMap<String, RoutePlugin>,

    pub cache: Option<RouteCache>,
}

impl Default for RouteStoreContainer {
    fn default() -> Self {
        RouteStoreContainer {
            load_balancer: Arc::new(
                LoadBalancer::<RoundRobin>::try_from_iter(vec!["127.0.0.1:80"]).unwrap(),
            ),
            match_with: None,
            host_header_remove: Vec::with_capacity(0),
            host_header_add: Vec::with_capacity(0),
            self_signed_certificate: false,
            plugins: HashMap::new(),
            upstreams: Vec::with_capacity(0),
            cache: None,
        }
    }
}

impl RouteStoreContainer {
    pub fn new(load_balancer: LoadBalancer<RoundRobin>) -> Self {
        RouteStoreContainer {
            load_balancer: Arc::new(load_balancer),
            match_with: None,
            host_header_remove: Vec::with_capacity(5),
            host_header_add: Vec::with_capacity(5),
            self_signed_certificate: false,
            plugins: HashMap::new(),
            upstreams: Vec::with_capacity(5),
            cache: None,
        }
    }
}

// LoadBalancer<RoundRobin>
/// A store for routes that is updated in a background thread
pub type RouteStore = papaya::HashMap<String, RouteStoreContainer>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_router_container_defaults_empty_pattern() {
        let load_balancer = LoadBalancer::<RoundRobin>::try_from_iter(vec!["1.1.1.1:80"]).unwrap();
        let route_store = RouteStoreContainer::new(load_balancer);

        assert!(route_store.match_with.is_none());
    }

    #[test]
    fn test_router_container_works_with_valid_and_invalid_pattern() {
        let load_balancer = LoadBalancer::<RoundRobin>::try_from_iter(vec!["1.1.1.1:80"]).unwrap();
        let mut route_store = RouteStoreContainer::new(load_balancer);
        
        // Initialize match_with with a RouteMatcher that has a path
        route_store.match_with = Some(RouteMatcher {
            path: Some(RoutePathMatcher {
                patterns: vec![Cow::Borrowed("/auth")],
            }),
            header: None,
        });

        assert!(route_store.match_with.is_some());

        // Get the path pattern from the match_with field
        let pattern = route_store.match_with.as_ref()
            .unwrap()
            .path.as_ref()
            .unwrap()
            .patterns.as_slice();
        
        // Create a PathTree for testing path matching
        let mut path_tree = PathTree::new();
        for (index, value) in pattern.iter().enumerate() {
            let _ = path_tree.insert(value, index);
        }
        
        assert!(path_tree.find("/auth").is_some());

        let (h, p) = path_tree.find("/auth").unwrap();
        assert_eq!(h, &0);
        assert_eq!(p.pattern(), "/auth");

        assert!(path_tree.find("/invalid").is_none());
    }
}
