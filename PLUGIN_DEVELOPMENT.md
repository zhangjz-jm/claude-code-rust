# Claude Code 插件开发指南

本文档详细介绍了如何为 Claude Code 开发、构建和分发插件。

## 概述

Claude Code 插件系统基于 Rust 动态库架构，提供了安全、高性能的插件运行时。插件可以扩展 Claude Code 的功能，添加新的命令、工具或集成外部服务。

## 插件架构

### 核心概念

1. **动态库**：插件编译为 `.so` (Linux)、`.dll` (Windows) 或 `.dylib` (macOS) 文件
2. **能力系统**：插件声明所需的能力，主机根据策略授予访问权限
3. **沙箱机制**：插件在受限环境中运行，保护主机系统安全
4. **签名验证**：插件可选的 ed25519 签名，确保来源可信

### 插件生命周期

```
加载 → 初始化 → 运行 → 停止 → 卸载
```

## 创建第一个插件

### 1. 创建项目结构

```bash
cargo new --lib my-claude-plugin
cd my-claude-plugin
```

### 2. 配置 Cargo.toml

```toml
[package]
name = "my-claude-plugin"
version = "0.1.0"
edition = "2021"
description = "My first Claude Code plugin"
authors = ["Your Name <email@example.com>"]
license = "MIT OR Apache-2.0"

# 插件必须编译为动态库
[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
async-trait = "0.1"
thiserror = "1.0"

# 可选：用于签名插件
ed25519-dalek = "2.0"
rand = "0.8"
```

### 3. 实现插件核心

创建 `src/lib.rs`：

```rust
use std::fmt::Debug;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;

/// 插件元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub entry_point: String,
    pub dependencies: Vec<String>,
    pub capabilities: Vec<String>,
    pub signature: Option<String>,
}

/// 插件状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PluginState {
    Unloaded,
    Loading,
    Loaded,
    Running,
    Error,
    Unloading,
}

/// 插件 API（由主机提供）
#[derive(Debug, Clone)]
pub struct PluginApi {
    version: String,
    plugin_name: String,
    capabilities: Vec<String>,
}

/// 插件特质
#[async_trait]
pub trait Plugin: Debug + Send + Sync {
    fn metadata(&self) -> &PluginMetadata;
    fn state(&self) -> PluginState;
    
    async fn initialize(&mut self, api: PluginApi) -> Result<(), Box<dyn std::error::Error>>;
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    async fn unload(&mut self) -> Result<(), Box<dyn std::error::Error>>;
}

/// 你的插件实现
#[derive(Debug)]
pub struct MyPlugin {
    metadata: PluginMetadata,
    state: PluginState,
    api: Option<PluginApi>,
}

impl MyPlugin {
    pub fn new() -> Self {
        Self {
            metadata: PluginMetadata {
                name: "my-claude-plugin".to_string(),
                version: "0.1.0".to_string(),
                author: "Your Name".to_string(),
                description: "My first Claude Code plugin".to_string(),
                entry_point: "plugin_entry".to_string(),
                dependencies: vec![],
                capabilities: vec![
                    "file:read".to_string(),
                    "file:write".to_string(),
                ],
                signature: None,
            },
            state: PluginState::Unloaded,
            api: None,
        }
    }
}

#[async_trait]
impl Plugin for MyPlugin {
    fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }
    
    fn state(&self) -> PluginState {
        self.state
    }
    
    async fn initialize(&mut self, api: PluginApi) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PluginState::Loading;
        self.api = Some(api);
        
        println!("MyPlugin initialized!");
        self.state = PluginState::Loaded;
        Ok(())
    }
    
    async fn start(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PluginState::Running;
        println!("MyPlugin started!");
        
        // 示例：使用插件API
        if let Some(api) = &self.api {
            // 检查是否有文件读取能力
            if api.has_capability("file:read") {
                // 可以安全地调用文件读取操作
                println!("Plugin has file read capability");
            }
        }
        
        Ok(())
    }
    
    async fn stop(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PluginState::Loaded;
        println!("MyPlugin stopped!");
        Ok(())
    }
    
    async fn unload(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.state = PluginState::Unloading;
        self.api = None;
        self.state = PluginState::Unloaded;
        println!("MyPlugin unloaded!");
        Ok(())
    }
}

/// 插件入口点 - 必须导出此函数
#[no_mangle]
pub extern "C" fn plugin_entry() -> *mut dyn Plugin {
    let plugin = MyPlugin::new();
    let boxed: Box<dyn Plugin> = Box::new(plugin);
    Box::into_raw(boxed)
}
```

### 4. 创建插件元数据文件

创建 `plugin.json` 在项目根目录：

```json
{
  "name": "my-claude-plugin",
  "version": "0.1.0",
  "author": "Your Name",
  "description": "My first Claude Code plugin",
  "entry_point": "plugin_entry",
  "dependencies": [],
  "capabilities": ["file:read", "file:write"],
  "signature": null
}
```

### 5. 构建插件

```bash
# 调试构建
cargo build

# 发布构建
cargo build --release
```

构建后的动态库位于：
- Linux: `target/debug/libmy_claude_plugin.so`
- Windows: `target/debug/my_claude_plugin.dll`
- macOS: `target/debug/libmy_claude_plugin.dylib`

## 插件能力系统

### 内置能力

| 能力 | 描述 | 示例用途 |
|------|------|----------|
| `file:read` | 读取文件 | 读取配置文件、日志文件 |
| `file:write` | 写入文件 | 保存设置、生成报告 |
| `network` | 网络访问 | HTTP请求、API调用 |
| `command:exec` | 执行命令 | 运行系统命令、脚本 |
| `command:register` | 注册命令 | 添加新的CLI命令 |
| `env:access` | 环境变量访问 | 读取配置、获取路径 |
| `system:info` | 系统信息 | 获取CPU、内存、磁盘信息 |
| `event:listen` | 监听事件 | 响应系统事件 |
| `event:emit` | 发送事件 | 通知其他组件 |
| `config:write` | 写入配置 | 保存插件设置 |
| `process:create` | 创建进程 | 启动外部程序 |
| `ipc` | 进程间通信 | 与其他插件通信 |
| `storage` | 持久化存储 | 保存插件数据 |

### 能力声明

插件在元数据中声明所需能力：

```json
{
  "capabilities": ["file:read", "file:write", "network"]
}
```

### 能力检查

在插件代码中检查能力：

```rust
if api.has_capability("file:read") {
    // 安全地执行文件读取操作
    let content = api.read_file("config.txt").await?;
}
```

## 插件API参考

### 文件操作

```rust
// 读取文件
let content = api.read_file("path/to/file.txt").await?;

// 写入文件
api.write_file("path/to/output.txt", "Hello, World!").await?;

// 检查文件是否存在
let exists = api.file_exists("path/to/file.txt").await?;
```

### 命令执行

```rust
// 执行命令
let output = api.execute_command("echo Hello World").await?;

// 执行命令并获取状态码
let (output, status) = api.execute_command_with_status("ls -la").await?;
```

### 网络请求

```rust
// HTTP GET请求
let response = api.http_request("https://api.example.com/data", "GET").await?;

// 带参数的请求
let response = api.http_request_with_body(
    "https://api.example.com/post",
    "POST",
    "application/json",
    r#"{"key": "value"}"#
).await?;
```

### 配置管理

```rust
// 获取配置
let value = api.get_config("my_setting").await?;

// 设置配置
api.set_config("my_setting", "new_value").await?;
```

### 事件系统

```rust
// 注册事件处理器
api.register_event_listener("file_changed", |event_data| {
    println!("File changed: {}", event_data);
    Ok(())
})?;

// 发送事件
api.emit_event("plugin_ready", "My plugin is ready!").await?;
```

### 日志记录

```rust
// 记录日志
api.log("info", "Plugin initialized successfully").await?;
api.log("warn", "Low disk space detected").await?;
api.log("error", "Failed to connect to database").await?;
```

## 插件签名

### 为什么需要签名

插件签名确保：
1. **来源可信**：插件来自可信的开发者
2. **完整性**：插件未被篡改
3. **不可否认**：开发者不能否认创建了插件

### 生成密钥对

```bash
# 安装ed25519工具
cargo install ed25519-dalek-tools

# 生成密钥对
ed25519-keygen generate
```

### 签名插件

```rust
use ed25519_dalek::{Keypair, Signer};
use rand::rngs::OsRng;
use base64;

// 生成密钥对
let mut csprng = OsRng{};
let keypair: Keypair = Keypair::generate(&mut csprng);

// 计算插件哈希
let plugin_bytes = std::fs::read("target/release/libmy_plugin.so")?;
let metadata_json = serde_json::to_string(&metadata)?;
let mut data_to_sign = Vec::new();
data_to_sign.extend(plugin_bytes);
data_to_sign.extend(metadata_json.as_bytes());

// 签名
let signature = keypair.sign(&data_to_sign);

// 编码为base64
let signature_b64 = base64::encode(signature.to_bytes());

// 添加到元数据
metadata.signature = Some(signature_b64);
```

### 验证签名

主机自动验证插件签名。如果验证失败，插件将无法加载。

## 插件分发

### 分发方式

1. **官方市场**：提交到 Claude Code 官方插件市场
2. **GitHub发布**：在 GitHub Releases 中发布插件
3. **私有仓库**：在企业内部部署私有插件仓库
4. **直接文件**：直接分享 `.so`/`.dll` 文件

### 插件包结构

```
my-plugin/
├── plugin.json          # 插件元数据
├── README.md           # 插件文档
├── LICENSE            # 许可证
├── CHANGELOG.md       # 更新日志
├── libmy_plugin.so    # 插件二进制文件 (Linux)
├── my_plugin.dll      # 插件二进制文件 (Windows)
└── my_plugin.dylib    # 插件二进制文件 (macOS)
```

### 提交到官方市场

1. 在 [plugins.claude.ai](https://plugins.claude.ai) 创建账户
2. 填写插件信息
3. 上传插件文件和元数据
4. 等待审核和发布

## 最佳实践

### 安全性

1. **最小权限原则**：只声明必需的能力
2. **输入验证**：验证所有外部输入
3. **错误处理**：妥善处理错误，不泄露敏感信息
4. **资源清理**：及时释放资源，避免内存泄漏

### 性能

1. **异步操作**：使用异步API进行I/O操作
2. **缓存结果**：缓存频繁访问的数据
3. **惰性初始化**：推迟资源密集型操作
4. **批处理**：批量处理相似操作

### 兼容性

1. **API版本**：检查API版本兼容性
2. **向后兼容**：保持插件接口稳定
3. **错误恢复**：优雅处理API变更
4. **降级策略**：提供功能降级方案

### 用户体验

1. **清晰文档**：提供完整的文档
2. **配置示例**：提供配置示例
3. **错误消息**：提供有用的错误消息
4. **日志输出**：输出有意义的日志

## 调试插件

### 调试模式

```bash
# 设置调试环境变量
export CLAUDE_PLUGIN_DEBUG=1
export RUST_LOG=debug

# 运行Claude Code
claude --debug
```

### 日志查看

插件日志输出到：
- 标准输出（调试模式）
- 系统日志文件
- Claude Code 日志文件

### 常见问题

#### 插件无法加载
- 检查动态库格式是否正确
- 检查插件入口点函数名称
- 检查依赖项是否满足
- 检查能力声明是否合理

#### 权限错误
- 确认插件声明了所需能力
- 检查沙箱配置是否允许操作
- 验证文件路径是否在允许范围内

#### 崩溃问题
- 检查内存安全问题
- 验证线程安全性
- 测试边界条件

## 示例插件

### 代码格式化插件

```rust
// 实现自动代码格式化功能
```

### Git集成插件

```rust
// 提供增强的Git操作
```

### 测试运行插件

```rust
// 运行测试并生成报告
```

### 数据库插件

```rust
// 连接和操作数据库
```

## 资源

- [Claude Code 文档](https://docs.claude.ai/code)
- [插件API参考](https://docs.claude.ai/code/plugins/api)
- [示例插件仓库](https://github.com/claude-code/example-plugins)
- [插件开发论坛](https://community.claude.ai/c/plugins)

## 支持

- 问题报告：GitHub Issues
- 功能请求：GitHub Discussions
- 安全漏洞：security@claude.ai
- 商业支持：enterprise@claude.ai

---

*最后更新：2026年4月*
*Claude Code 插件系统版本：1.0.0*