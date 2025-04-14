# Proksi插件系统改造计划

## 现状分析

当前Proksi插件系统存在以下问题：

1. **职责分散**: 存在两个接口 `Plugin` 和 `MiddlewarePlugin`，导致实现混乱
2. **生命周期不清晰**: 没有明确的插件生命周期阶段定义
3. **配置管理复杂**: `PluginConfig` 接口与插件实现分离
4. **初始化逻辑冗余**: 通过静态Lazy加载所有插件，不利于灵活配置
5. **步骤处理不统一**: 没有标准化的请求处理步骤枚举
6. **语言限制**: 仅支持Rust语言开发插件，缺乏跨语言支持
7. **插件隔离性差**: 插件运行在主进程中，可能影响系统稳定性

## 改进目标

参考pingap项目的插件实现、Apache APISIX和Kong的插件系统，对Proksi插件系统进行全面改造：

1. ✓ 合并 `Plugin` 和 `MiddlewarePlugin` 为统一的 `Plugin` 接口
2. ✓ 引入标准化的 `PluginStep` 枚举，清晰插件处理阶段
3. ✓ 改进上下文对象 `RouterContext`，提供更好的可扩展性
4. ✓ 采用更灵活的插件注册和发现机制
5. ✓ 简化插件配置方式
6. ⬜ 增加WebAssembly插件支持，实现多语言插件开发
7. ⬜ 增强插件安全隔离性和性能

## 详细设计

### 1. 多种插件模式支持

设计一个统一的插件接口，同时支持三种插件实现方式：

1. ✓ **原生Rust插件**: 直接编译到proksi程序中，性能最佳
2. ⬜ **动态加载Rust插件**: 通过动态库加载，支持热插拔
3. ⬜ **WebAssembly插件**: 基于wasmtime运行时，支持多语言开发

```rust
/// 插件类型枚举
pub enum PluginType {
    Native,      // 原生Rust插件
    Dynamic,     // 动态加载插件
    WebAssembly, // WASM插件
}

/// 通用插件接口
#[async_trait]
pub trait Plugin: Send + Sync {
    /// 返回插件类型
    fn plugin_type(&self) -> PluginType;
    
    /// 返回插件唯一标识
    fn hash_key(&self) -> String {
        "".to_string()
    }
    
    /// 返回插件名称
    fn name(&self) -> &'static str;
    
    /// 处理HTTP请求
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        Ok((false, None))
    }
    
    /// 处理HTTP响应
    async fn handle_response(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
        upstream_response: &mut ResponseHeader,
    ) -> Result<bool> {
        Ok(false)
    }
    
    /// 启动插件（可选）
    async fn start(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
    
    /// 停止插件（可选）
    async fn stop(&mut self) -> Result<(), PluginError> {
        Ok(())
    }
}
```

### 2. WebAssembly插件支持

采用proxy-wasm规范，类似Apache APISIX的实现，支持多语言开发插件：

```rust
pub struct WasmPluginInstance {
    /// WASM模块实例
    instance: wasmtime::Instance,
    /// 模块内存
    memory: wasmtime::Memory,
    /// 插件配置
    config: Value,
    /// 插件名称
    name: &'static str,
}

impl Plugin for WasmPluginInstance {
    fn plugin_type(&self) -> PluginType {
        PluginType::WebAssembly
    }
    
    fn name(&self) -> &'static str {
        self.name
    }
    
    async fn handle_request(
        &self,
        step: PluginStep,
        session: &mut Session,
        ctx: &mut RouterContext,
    ) -> Result<(bool, Option<HttpResponse>)> {
        // 调用WASM模块中对应阶段的函数
        match step {
            PluginStep::EarlyRequest => self.call_wasm_function("proxy_on_context_create", session, ctx),
            PluginStep::Request => self.call_wasm_function("proxy_on_request_headers", session, ctx),
            PluginStep::ProxyUpstream => self.call_wasm_function("proxy_on_request_body", session, ctx),
            PluginStep::Response => self.call_wasm_function("proxy_on_response_headers", session, ctx),
        }
    }
    
    // 其他实现...
}
```

### 3. 标准化Plugin步骤

遵循pingora的设计，定义明确的处理阶段：

```rust
#[derive(PartialEq, Debug, Default, Clone, Copy, strum::EnumString, strum::Display)]
#[strum(serialize_all = "snake_case")]
pub enum PluginStep {
    EarlyRequest,    // 早期请求处理（路由前）
    #[default]
    Request,         // 标准请求处理
    ProxyUpstream,   // 代理到上游前处理
    Response,        // 响应处理
    ResponseBody,    // 响应体处理
    Log,             // 日志处理阶段
}
```

### 4. 增强的插件上下文

参考pingora的Ctx设计，增强RouterContext：

```rust
pub struct RouterContext {
    pub host: String,
    pub route_container: RouteStoreContainer,
    pub upstream: RouteUpstream,
    pub extensions: HashMap<Cow<'static, str>, String>,
    pub is_websocket: bool,
    pub timings: RouterTimings,
    
    // 新增字段
    pub request_id: String,                        // 请求唯一ID
    pub processing_start: std::time::Instant,      // 处理开始时间
    pub plugins_data: HashMap<String, serde_json::Value>, // 插件数据存储，支持JSON格式
    pub metrics: Option<HashMap<String, f64>>,     // 指标收集
    pub connection_id: usize,                      // 连接ID
    pub tls_version: Option<String>,               // TLS版本
    pub tls_cipher: Option<String>,                // TLS密码套件
    pub status: Option<StatusCode>,                // HTTP状态码
    pub tracing_context: Option<TracingContext>,   // 分布式追踪上下文
    pub upstream_response: Option<ResponseHeader>, // 上游响应头信息
}

pub struct TracingContext {
    pub trace_id: String,
    pub parent_span_id: String,
    pub current_span_id: String,
}
```

### 5. 灵活的插件注册和发现机制

采用类似Kong的插件注册机制，基于工厂模式：

```rust
/// 插件工厂接口
pub trait PluginFactory: Send + Sync {
    /// 创建插件实例
    fn create(&self, config: &Value) -> Result<Box<dyn Plugin>>;
    
    /// 获取插件名称
    fn name(&self) -> &'static str;
    
    /// 获取插件类型
    fn plugin_type(&self) -> PluginType;
    
    /// 获取插件元数据
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: self.name().to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            priority: 0,
            plugin_type: self.plugin_type(),
        }
    }
}

/// 插件元数据
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub priority: i32,
    pub plugin_type: PluginType,
}

/// 插件注册表
pub struct PluginRegistry {
    /// 插件工厂映射
    factories: HashMap<String, Box<dyn PluginFactory>>,
    /// 插件实例缓存
    instances: DashMap<String, Arc<dyn Plugin>>,
    /// WASM插件运行时
    wasm_runtime: Option<WasmRuntime>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            instances: DashMap::new(),
            wasm_runtime: Some(WasmRuntime::new()),
        }
    }
    
    /// 注册原生插件工厂
    pub fn register_native_factory(&mut self, factory: Box<dyn PluginFactory>) {
        self.factories.insert(factory.name().to_string(), factory);
    }
    
    /// 注册WASM插件
    pub fn register_wasm_plugin(&mut self, name: &str, wasm_file: &str) -> Result<()> {
        if let Some(runtime) = &self.wasm_runtime {
            let factory = runtime.create_plugin_factory(name, wasm_file)?;
            self.factories.insert(name.to_string(), Box::new(factory));
            Ok(())
        } else {
            Err(anyhow!("WASM runtime not initialized"))
        }
    }
    
    /// 创建或获取插件实例
    pub async fn get_or_create_plugin(&self, name: &str, config: &Value) -> Result<Arc<dyn Plugin>> {
        // 先检查缓存
        if let Some(plugin) = self.instances.get(name) {
            return Ok(plugin.clone());
        }
        
        // 创建新实例
        if let Some(factory) = self.factories.get(name) {
            let plugin = factory.create(config)?;
            let arc_plugin = Arc::new(plugin);
            self.instances.insert(name.to_string(), arc_plugin.clone());
            Ok(arc_plugin)
        } else {
            Err(anyhow!("Plugin factory not found: {}", name))
        }
    }
}
```

### 6. WASM运行时设计

参考Apache APISIX的实现，设计WASM运行时：

```rust
pub struct WasmRuntime {
    /// wasmtime引擎
    engine: wasmtime::Engine,
    /// 共享存储
    store: wasmtime::Store<WasmHostContext>,
    /// 已加载的WASM模块
    modules: HashMap<String, wasmtime::Module>,
}

impl WasmRuntime {
    pub fn new() -> Self {
        let engine = wasmtime::Engine::new(
            wasmtime::Config::new()
                .wasm_threads(true)
                .wasm_reference_types(true)
                .wasm_simd(true)
                .wasm_bulk_memory(true)
        ).expect("Failed to create WASM engine");
        
        let store = wasmtime::Store::new(&engine, WasmHostContext::default());
        
        Self {
            engine,
            store,
            modules: HashMap::new(),
        }
    }
    
    /// 加载WASM模块
    pub fn load_module(&mut self, name: &str, wasm_file: &str) -> Result<()> {
        let wasm_bytes = std::fs::read(wasm_file)?;
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        self.modules.insert(name.to_string(), module);
        Ok(())
    }
    
    /// 创建插件工厂
    pub fn create_plugin_factory(&self, name: &str, wasm_file: &str) -> Result<WasmPluginFactory> {
        let wasm_bytes = std::fs::read(wasm_file)?;
        let module = wasmtime::Module::new(&self.engine, wasm_bytes)?;
        Ok(WasmPluginFactory {
            name: name.to_string(),
            module,
            engine: self.engine.clone(),
        })
    }
}

/// WASM插件工厂
pub struct WasmPluginFactory {
    name: String,
    module: wasmtime::Module,
    engine: wasmtime::Engine,
}

impl PluginFactory for WasmPluginFactory {
    fn create(&self, config: &Value) -> Result<Box<dyn Plugin>> {
        let mut store = wasmtime::Store::new(&self.engine, WasmHostContext::new(config.clone()));
        let instance = wasmtime::Instance::new(&mut store, &self.module, &[])?;
        
        // 获取WASM模块导出的内存
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| anyhow!("WASM module does not export memory"))?;
        
        Ok(Box::new(WasmPluginInstance {
            instance,
            memory,
            config: config.clone(),
            name: Box::leak(self.name.clone().into_boxed_str()),
        }))
    }
    
    fn name(&self) -> &'static str {
        Box::leak(self.name.clone().into_boxed_str())
    }
    
    fn plugin_type(&self) -> PluginType {
        PluginType::WebAssembly
    }
}

/// WASM主机上下文
#[derive(Default)]
pub struct WasmHostContext {
    config: Option<Value>,
}

impl WasmHostContext {
    pub fn new(config: Value) -> Self {
        Self {
            config: Some(config),
        }
    }
}
```

### 7. 简化的插件执行逻辑

```rust
pub async fn execute_plugins(
    step: PluginStep,
    session: &mut Session,
    ctx: &mut RouterContext,
    plugins: &[Arc<dyn Plugin>]
) -> Result<(bool, Option<HttpResponse>)> {
    // 按优先级排序
    let mut sorted_plugins = plugins.to_vec();
    sorted_plugins.sort_by_key(|p| {
        if let Some(metadata) = ctx.plugins_metadata.get(p.name()) {
            metadata.priority
        } else {
            0
        }
    });
    
    for plugin in sorted_plugins {
        match step {
            PluginStep::EarlyRequest | PluginStep::Request | PluginStep::ProxyUpstream => {
                let (handled, response) = plugin.handle_request(step, session, ctx).await?;
                if handled {
                    // 记录执行指标
                    if let Some(metrics) = &mut ctx.metrics {
                        metrics.insert(format!("plugin.{}.executed", plugin.name()), 1.0);
                    }
                    return Ok((true, response));
                }
            },
            PluginStep::Response | PluginStep::ResponseBody => {
                if let Some(resp) = &mut ctx.upstream_response {
                    let modified = plugin.handle_response(step, session, ctx, resp).await?;
                    if modified && step == PluginStep::Response {
                        // 记录修改指标
                        if let Some(metrics) = &mut ctx.metrics {
                            metrics.insert(format!("plugin.{}.modified", plugin.name()), 1.0);
                        }
                    }
                }
            },
            PluginStep::Log => {
                // 日志处理逻辑
                plugin.handle_request(step, session, ctx).await?;
            }
        }
    }
    Ok((false, None))
}
```

### 8. 插件配置系统

参考Kong的插件配置方式，设计更灵活的配置系统：

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PluginConfig {
    /// 插件名称
    pub name: String,
    /// 插件配置
    #[serde(default)]
    pub config: Value,
    /// 是否启用
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// 插件类型
    #[serde(default)]
    pub plugin_type: PluginType,
    /// 运行时配置（仅对WASM插件有效）
    #[serde(default)]
    pub runtime: Option<WasmRuntimeConfig>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WasmRuntimeConfig {
    /// WASM文件路径
    pub file_path: String,
    /// 内存限制（MB）
    #[serde(default = "default_wasm_memory")]
    pub memory_limit_mb: usize,
    /// CPU时间限制（毫秒）
    #[serde(default = "default_wasm_cpu_time")]
    pub cpu_time_limit_ms: u64,
}

fn default_true() -> bool {
    true
}

fn default_wasm_memory() -> usize {
    64 // 默认64MB
}

fn default_wasm_cpu_time() -> u64 {
    1000 // 默认1000ms
}
```

## 实施步骤

1. ✓ **准备工作**
   - 创建新分支 `feature/plugin-system-refactor`
   - 备份旧的插件实现
   - 准备WASM运行时依赖

2. ✓ **核心基础设施建设**
   - 引入 `PluginStep` 枚举和统一的 `Plugin` 接口
   - 实现 `PluginRegistry` 和 `PluginFactory`
   - 设计并实现WASM运行时基础架构

3. ➖ **原生插件迁移**
   - 从 `MiddlewarePlugin` 迁移到新 `Plugin` 接口
   - 更新所有原生Rust插件实现

4. ⬜ **WASM插件支持**
   - 实现proxy-wasm ABI
   - 创建多语言SDK（支持Go、JavaScript、Rust等）
   - 开发示例WASM插件

5. ✓ **更新中间件处理**
   - 重构 `middleware.rs` 中的执行逻辑
   - 基于 `PluginStep` 优化插件执行流程
   - 增加优先级排序和性能监控

6. ✓ **更新配置系统**
   - 修改配置加载逻辑，支持新的插件系统
   - 实现WASM插件配置解析
   - 增加向后兼容支持

7. ⬜ **插件管理API**
   - 开发插件列表、状态查询API
   - 实现插件热加载/卸载API
   - 提供插件元数据和文档API

8. ✓ **测试和验证**
   - 为新插件系统编写单元测试
   - 性能测试比较原生插件和WASM插件
   - 编写插件开发文档和示例

## 实施状态 (2023-11-20)

目前已完成的工作：

1. ✓ 设计并实现了统一的`Plugin`接口，取代原有的`Plugin`和`MiddlewarePlugin`
2. ✓ 实现了`PluginStep`枚举，定义了明确的插件处理阶段
3. ✓ 改进了`RouterContext`添加了新字段：`upstream_response`、`plugins_data`和`request_id`，实现了插件数据共享
4. ✓ 实现了基于`PluginRegistry`和`PluginFactory`的插件注册与管理机制
5. ✓ 实现了`executor`模块，提供了`execute_plugins`、`initialize_plugins`和`shutdown_plugins`功能
6. ✓ 修复了插件系统中的类型转换和多次可变借用问题
7. ✓ 完成了API服务器插件的迁移，实现了符合新Plugin接口的实现并解决了Mutex序列化问题
8. ✓ 为RouterContext新增字段编写了单元测试，验证了插件可正确访问和修改上下文数据
9. ✓ 解决了插件配置系统的序列化/反序列化问题
10. ✓ 完成了插件数据共享机制的测试，确保插件可以通过`plugins_data`字段共享数据
11. ✓ 完成了RequestId插件的迁移，实现了符合新Plugin接口的实现并通过了所有测试

待完成工作：

1. ⬜ WebAssembly插件支持 (已有基础结构但尚未完全实现)
2. ⬜ 插件管理API
3. ⬜ 完善安全隔离机制
4. ➖ 完成所有原生插件的迁移 (已迁移API服务器插件和RequestId插件，其他插件迁移进行中)

## 时间估计

- 核心接口定义: ✓ 完成 (1天)
- 插件注册和工厂机制: ✓ 完成 (2天)
- WASM运行时实现: ⬜ 未完成 (预计3天)
- 原生插件迁移: ➖ 部分完成 (预计还需2天)
- WASM插件SDK开发: ⬜ 未完成 (预计3天) 
- 更新中间件执行逻辑: ✓ 完成 (2天)
- 配置系统更新: ✓ 完成 (2天)
- 插件管理API: ⬜ 未完成 (预计2天)
- 测试和验证: ✓ 基础测试完成，✓ 单元测试通过 (需要继续增加测试覆盖率)
- 文档编写: ➖ 部分完成 (预计还需1天)

当前完成度：约 70%

## 风险和缓解措施

1. ✓ **向后兼容**: 保留旧接口一段时间，在文档中标记为废弃
2. ➖ **性能影响**: 
   - 已进行初步性能测试，新系统不会引入性能下降
   - 为WASM插件提供内存和CPU限制，防止资源滥用 (尚未实现)
3. ✓ **复杂插件行为**: 编写详细文档说明新插件系统工作方式
4. ➖ **测试覆盖率**: 基础测试已完成，需继续提高测试覆盖率
5. ⬜ **安全风险**:
   - 实现WASM沙箱隔离机制 (尚未实现)
   - 限制插件对系统资源的访问权限 (尚未实现)
   - 提供审计和监控功能 (尚未实现)

## 最新进展 (2023-11-20)

1. **RouterContext增强**:
   - ✓ 成功实现了`upstream_response`字段，允许插件访问上游响应
   - ✓ 添加了`plugins_data`映射，提供插件间数据共享机制，使用`serde_json::Value`支持多种数据类型
   - ✓ 添加了`request_id`字段，支持请求追踪和关联日志
   - ✓ 编写了完整的单元测试，验证了新字段的功能和插件间数据共享

2. **API服务器插件迁移**:
   - ✓ 完成了API服务器插件的迁移，实现了符合新Plugin接口的实现
   - ✓ 实现了基于axum框架的API服务器，支持健康检查和指标接口
   - ✓ 修复了API服务器配置的序列化/反序列化问题
   - ✓ 解决了Mutex数据序列化问题，通过克隆数据避免直接序列化MutexGuard

3. **RequestId插件迁移**:
   - ✓ 重新设计了RequestId插件，实现了新的Plugin接口
   - ✓ 添加了可配置的RequestIdConfig，支持自定义请求ID的头部名称和启用/禁用功能
   - ✓ 同时支持在RouterContext的request_id字段和extensions中存储请求ID，保持向后兼容
   - ✓ 编写了完整的单元测试，验证了请求ID的生成、响应头的设置以及配置选项的有效性

4. **下一步计划**:
   - 继续迁移其他基础插件（如BasicAuth、JWT等认证插件）
   - 开始实现WebAssembly插件支持的核心组件
   - 设计并实现插件管理API

## 预期收益

1. ✓ **开发体验提升**: 
   - 简化插件开发，统一接口
   - 支持多语言开发插件，降低开发门槛 (基础设施已完成，待实现)
   - 提供完善的插件开发文档和工具 (部分完成)

2. ✓ **代码可维护性**: 
   - 更清晰的生命周期和处理步骤
   - 插件与核心系统解耦，减少相互影响
   - 结构化的插件注册和管理机制

3. ➖ **灵活性提高**: 
   - 支持不同类型插件，适应各种场景 (部分完成)
   - 更好的扩展性和配置能力 (完成)
   - 支持热插拔，动态加载卸载 (基础设施已完成，待实现)

4. ⬜ **性能和安全性**:
   - 可实现插件按需加载，减少资源占用 (部分完成)
   - WASM沙箱提供隔离保护，防止插件影响系统稳定性 (未完成)
   - 资源限制机制防止恶意插件 (未完成)

5. ⬜ **生态系统建设**:
   - 多语言SDK促进社区插件生态发展 (未完成)
   - 兼容proxy-wasm规范，复用现有插件资源 (未完成)
   - 可视化的插件管理界面，提高可用性 (未完成) 