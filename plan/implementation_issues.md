# Proksi 项目改进计划 (Improvement Plan)

## 1. 概述 (Overview)

通过对 `cargo test --package proksi` 的结果分析，我们发现了一系列需要修复的问题。主要包括未解析的导入、未实现的接口以及大量未使用的导入和变量。本文档将提供一个综合的改进计划。

## 2. 关键问题 (Key Issues)

### 2.1 严重错误 (Critical Errors)

1. **缺失模块结构**: ✅ 
   - `crate::stores` 模块未在根目录定义，影响了多个文件的引用
   - `crate::MsgProxy` 和 `crate::MsgRoute` 未在根目录定义

2. **接口实现缺失**: ✅
   - `http_server::HttpServer` 未实现 `pingora::services::Service` 接口
   - 可能是 API 改变或结构调整导致

3. **字段访问错误**: ✅
   - 在 `discovery/mod.rs` 中尝试访问不存在的字段: `matcher.path`

4. **RouterContext 插件数据访问错误**: ✅
   - 在 `plugins/request_id/mod.rs` 中尝试使用不存在的 `ctx.get()` 和 `ctx.insert()` 方法
   - 修复方法：改为使用 `ctx.plugins_data.get()` 和 `ctx.plugins_data.insert()`
   - 相关修复: 修复了 header 名称字符串的生命周期问题

### 2.2 结构性问题 (Structural Issues)

1. **导入组织混乱**: ⚠️ (部分完成)
   - 大量未使用的导入（116+ 警告）
   - 文件级别和测试模块级别的导入冗余
   - 已修复: 在 `plugins/mod.rs` 中添加了缺失的 `std::borrow::Cow` 导入

2. **代码质量问题**: ⚠️ (部分完成)
   - 大量未使用的变量
   - 不必要的可变声明 (`mut`)
   - 未使用的参数，特别是在回调函数中
   - 已修复: RequestId 插件中的方法签名匹配 Plugin trait 要求

3. **类型不匹配问题**: ✅
   - `Certificate` 结构体字段不匹配 - 已修复
   - 方法调用参数类型不匹配 - 已修复
   - 序列化/反序列化失败 - 已添加错误实现

## 3. 改进计划 (Improvement Plan)

### 3.1 结构重组 (Structural Reorganization)

1. **重新设计模块结构**: ✅
   ```rust
   // 在 lib.rs 中添加
   pub mod stores;
   pub use models::{MsgProxy, MsgRoute}; // 已实现，位置调整为在 models 中
   ```

2. **设置正确的重导出**: ✅
   - 确保核心组件在根模块可见
   - 建立清晰的模块边界和公共接口

### 3.2 接口实现 (Interface Implementation)

1. **实现 `Service` 接口**: ✅
   ```rust
   // 在 http_server.rs 中添加
   #[async_trait]
   impl Service for HttpServer {
       async fn start_service(&mut self, _fds: Option<ListenFds>, _shutdown: ShutdownWatch) {
           tracing::info!("HTTP Server starting...");
       }

       fn name(&self) -> &'static str {
           "http_server"
       }

       fn threads(&self) -> Option<usize> {
           Some(4)
       }
   }
   ```

2. **更新 `RoutePathMatcher` 字段访问**: ✅
   - 修正 `discovery/mod.rs` 中的 253 行，使用解引用和重新构造方式
   ```rust
   *matcher = RouteMatcher {
       path: Some(RoutePathMatcher {
           patterns: pattern.clone(),
       }),
       header: matcher.header.clone(),
   };
   ```

3. **修复 RouterContext 插件数据访问**: ✅
   - 修复 `plugins/request_id/mod.rs` 中的数据访问方法
   ```rust
   // 错误示例
   let request_id = ctx.get("request_id");
   ctx.insert("request_id".to_owned(), json!(uuid));
   
   // 正确实现
   let request_id = ctx.plugins_data.get("request_id");
   ctx.plugins_data.insert("request_id".to_string(), json!(uuid));
   ```

### 3.3 代码清理 (Code Cleanup)

1. **移除未使用导入**: ⚠️ (部分完成)
   - 使用 IDE 工具或 `cargo fix --allow-dirty` 自动移除
   - 特别关注插件模块中的未使用导入
   - 已完成: 修复了模块级别的导入问题 (如 `std::borrow::Cow`)

2. **参数命名修复**: ⚠️ (部分完成)
   - 对未使用的参数添加前缀 `_`
   - 例如: `session` → `_session`
   - 已完成: 修复了核心插件的方法参数匹配问题

3. **移除不必要的可变性**: ⚠️ (部分完成)
   - 检查并移除不必要的 `mut` 关键字

### 3.4 类型修复 (New)

1. **修复 Certificate 结构体不一致问题**: ✅
   - 检查并统一 `Certificate` 结构体的字段定义
   - 修复相关使用点的字段访问

2. **修复类型不匹配问题**: ✅
   - 解决 `String` 和 `PKey<Private>` 之间的类型转换
   - 解决 `Vec<RouteHeaderAdd>` 和 `Vec<(Cow<'static, str>, Cow<'static, str>)>` 之间的类型转换
   - 解决序列化问题，特别是 `Arc<X509>` 类型

3. **添加必要的 trait 实现**: ✅
   - 为 `StoreError` 添加 `std::error::Error` trait 实现

### 3.5 优先级任务 (Priority Tasks)

1. **立即修复**: 
   - 模块结构和导入问题 ✅
   - `HttpServer` 的 `Service` 实现 ✅
   - 类型不匹配问题 ✅
   - RouterContext 插件数据访问 ✅

2. **次要修复**:
   - 未使用的导入和变量 ⚠️ (部分完成)
   - 命名约定调整 ⚠️ (部分完成)

3. **长期改进**:
   - 代码结构重组 ❌
   - 测试覆盖率提高 ❌

## 4. 实施时间表 (Implementation Timeline)

1. **阶段一** (1-2天): ✅
   - 修复关键导入和模块结构
   - 实现必要接口
   - 修复类型不匹配问题
   - 修复 RouterContext 插件数据访问问题

2. **阶段二** (2-3天): ⚠️ (部分完成)
   - 清理未使用的导入和变量
   - 修复字段访问错误
   - 修复类型不匹配问题

3. **阶段三** (3-5天): ❌
   - 代码质量改进
   - 增加测试覆盖率

## 5. 技术债务识别 (Technical Debt Identification)

1. **代码冗余**:
   - 插件系统中存在大量重复模式
   - 考虑使用宏或特征来减少冗余

2. **测试不足**:
   - 许多模块似乎缺乏足够的测试
   - 建议增加单元测试和集成测试

3. **配置管理**:
   - 配置结构分散，可考虑统一管理

4. **类型系统改进**:
   - 需要统一处理第三方类型的序列化和反序列化
   - 考虑使用包装类型或自定义序列化

## 6. 结论 (Conclusion)

Proksi 项目目前面临一些结构性和接口实现问题，但这些问题都是可修复的。通过系统性地解决模块组织、接口实现和代码质量问题，可以使项目回到健康状态。

### 当前完成进度

- ✅ 结构重组: 已完成核心模块结构重组和重导出
- ✅ 接口实现: 已实现 `HttpServer` 的 `Service` 接口
- ✅ 字段访问修复: 已修复 `matcher.path` 的访问方式
- ✅ 类型修复: 已修复 `Certificate` 结构体字段不匹配问题，以及相关类型转换问题
- ✅ 错误处理改进: 已为 `StoreError` 添加 `std::error::Error` trait 实现
- ✅ RouterContext 数据访问: 修复了 RequestId 插件中的 RouterContext 数据访问问题
- ⚠️ 代码清理: 部分完成未使用导入和变量的清理，以及参数命名修复

建议接下来优先解决代码清理问题，特别是删除未使用的导入和变量以及修复不必要的可变声明，这些可以大大减少编译警告并提高代码质量。