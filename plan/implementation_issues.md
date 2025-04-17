# Proksi 项目改进计划 (Improvement Plan)

## 1. 概述 (Overview)

通过对 `cargo test --package proksi` 的结果分析，我们发现了一系列需要修复的问题。主要包括未解析的导入、未实现的接口以及大量未使用的导入和变量。本文档将提供一个综合的改进计划。

## 2. 关键问题 (Key Issues)

### 2.1 严重错误 (Critical Errors)

1. **缺失模块结构**: 
   - `crate::stores` 模块未在根目录定义，影响了多个文件的引用
   - `crate::MsgProxy` 和 `crate::MsgRoute` 未在根目录定义

2. **接口实现缺失**:
   - `http_server::HttpServer` 未实现 `pingora::services::Service` 接口
   - 可能是 API 改变或结构调整导致

3. **字段访问错误**:
   - 在 `discovery/mod.rs` 中尝试访问不存在的字段: `matcher.path`

### 2.2 结构性问题 (Structural Issues)

1. **导入组织混乱**:
   - 大量未使用的导入（116+ 警告）
   - 文件级别和测试模块级别的导入冗余

2. **代码质量问题**:
   - 大量未使用的变量
   - 不必要的可变声明 (`mut`)
   - 未使用的参数，特别是在回调函数中

## 3. 改进计划 (Improvement Plan)

### 3.1 结构重组 (Structural Reorganization)

1. **重新设计模块结构**:
   ```rust
   // 在 lib.rs 中添加
   pub mod stores;
   pub use services::{MsgProxy, MsgRoute}; // 根据实际位置调整
   ```

2. **设置正确的重导出**:
   - 确保核心组件在根模块可见
   - 建立清晰的模块边界和公共接口

### 3.2 接口实现 (Interface Implementation)

1. **实现 `Service` 接口**:
   ```rust
   // 在 http_server.rs 中添加
   impl pingora::services::Service for HttpServer {
       // 实现必要的方法
       fn start(&self) -> Result<()> { ... }
       fn stop(&self) { ... }
       // 其他必要的方法
   }
   ```

2. **更新 `RoutePathMatcher` 字段访问**:
   - 检查 `discovery/mod.rs` 中的 253 行
   - 更正 `matcher.path = Some(...)` 的赋值方式

### 3.3 代码清理 (Code Cleanup)

1. **移除未使用导入**:
   - 使用 IDE 工具或 `cargo fix --allow-dirty` 自动移除
   - 特别关注插件模块中的未使用导入

2. **参数命名修复**:
   - 对未使用的参数添加前缀 `_`
   - 例如: `session` → `_session`

3. **移除不必要的可变性**:
   - 检查并移除不必要的 `mut` 关键字

### 3.4 优先级任务 (Priority Tasks)

1. **立即修复**:
   - 模块结构和导入问题
   - `HttpServer` 的 `Service` 实现

2. **次要修复**:
   - 未使用的导入和变量
   - 命名约定调整

3. **长期改进**:
   - 代码结构重组
   - 测试覆盖率提高

## 4. 实施时间表 (Implementation Timeline)

1. **阶段一** (1-2天):
   - 修复关键导入和模块结构
   - 实现必要接口

2. **阶段二** (2-3天):
   - 清理未使用的导入和变量
   - 修复字段访问错误

3. **阶段三** (3-5天):
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

## 6. 结论 (Conclusion)

Proksi 项目目前面临一些结构性和接口实现问题，但这些问题都是可修复的。通过系统性地解决模块组织、接口实现和代码质量问题，可以使项目回到健康状态。建议优先解决导入和接口实现问题，然后逐步改善代码质量和测试覆盖率。 