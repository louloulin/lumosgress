# Proksi Build Error Fix Plan (xiufu.md)

## Analysis

Running `cargo test` revealed multiple build errors stemming from recent feature additions (like API Versioning) and potentially older issues (missing dependencies, incorrect module paths).

## Fix Plan

1.  ✅ **Fix `models::tenant` Import:**
    *   Created `models` module with `tenant.rs` in `crates/proksi/src/models/mod.rs`
    *   Implemented `Tenant`, `TenantStatus`, `ResourceQuota`, and `ResourceUsage` structs in `crates/proksi/src/models/tenant.rs`
    *   Added `pub mod models;` to `crates/proksi/src/lib.rs`

2.  ✅ **Add `thiserror` Dependency:**
    *   Added `thiserror = "1.0"` to `[dependencies]` in `crates/proksi/Cargo.toml`

3.  ✅ **Fix `PluginError` Trait Bound:**
    *   Added proper error derive with `thiserror::Error` imported and used in `plugins/mod.rs`

4.  ✅ **Fix Missing `Arc` Import:**
    *   Added `use std::sync::Arc;` to `crates/proksi/src/plugins/compliance/mod.rs`

5.  ✅ **Declare Plugin Modules:**
    *   Added `pub mod manager;` to `crates/proksi/src/plugins/mod.rs`
    *   Updated `plugins::manager` to expose module-level `register` and `get_plugin` functions for plugin management

6.  ✅ **Fix `server` Module Path:**
    *   Created `services/server.rs` with `start_api_server` function implementation
    *   Added `pub mod server;` to `services/mod.rs`
    *   Updated main.rs to use `services::server`

7.  ✅ **Update `RouteStoreContainer` for `match_with`:**
    *   Modified `RouteStoreContainer` to use `match_with: Option<RouteMatcher>` instead of `path_matcher`
    *   Updated `add_route_to_router` in `discovery/mod.rs` to correctly set `match_with`
    *   Fixed unit tests to work with the new structure

8.  ✅ **Initialize `RouteMatcher` Correctly:**
    *   Updated code to initialize `RouteMatcher` with `header: None` in `discovery/mod.rs`

9.  ✅ **Other Fixes:**
    *   Fixed temporary value dropped errors in `https_proxy.rs` by using `Box::leak` to create 'static strings for error messages
    *   Updated imports in main.rs to resolve unresolved modules
    *   Fixed the Plugin trait reference in `plugins/manager.rs` by correctly specifying the Config type parameter to resolve generic `Plugin` trait references

## Implementation Summary
All build errors have been resolved, and `cargo test` now passes successfully. We have fixed imports, module declarations, generic type parameters, and struct definitions to ensure that the code compiles and tests pass properly.

There are still numerous warnings in the codebase, primarily related to unused variables and imports, but these do not affect functionality and can be addressed as a separate task.

## Implementation Date: 2023-10-07