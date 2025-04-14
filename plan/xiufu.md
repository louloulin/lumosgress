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

10. ✅ **新插件系统修复：**
    *   修复了`core.rs`中的`Box<dyn Plugin>`到`Arc<dyn Plugin>`的转换问题，使用`Arc::from()`函数替代直接`Arc::new()`
    *   解决了`executor.rs`中的多次可变借用问题，通过先取出`upstream_response`，处理后再放回
    *   完善了`core_test.rs`中的导入，确保所有类型和trait都能正确解析
    *   改进了`CompliancePlugin`实现，添加了`TenantInfo`结构体，并优化了配置处理

## Implementation Summary
所有构建错误已经解决，`cargo test`现在可以成功通过。我们修复了导入、模块声明、泛型类型参数和结构体定义，确保代码能够正确编译和测试通过。

插件系统的改进包括更好的类型处理、修复内存管理问题（避免可变借用冲突）、以及完善了合规性插件的功能。同时，我们也扩展了租户信息处理，添加了必要的数据结构。

在代码库中仍然存在一些警告，主要与未使用的变量和导入有关，但这些不影响功能，可以作为单独的任务来解决。

## Implementation Date: 2023-10-20