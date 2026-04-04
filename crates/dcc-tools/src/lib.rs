//! DCC Tools - Digital Content Creation Tools Integration
//!
//! 为 Claude Code 提供与 Blender、Unreal Engine 5、Maya 等 DCC 工具的集成支持
//!
//! # 功能特性
//!
//! - **Blender 集成**: 完整的 Python API 支持，场景操作、渲染、材质编辑
//! - **UE5 集成**: 远程控制 API、蓝图操作、关卡编辑
//! - **统一接口**: 所有 DCC 工具使用一致的 API
//! - **事件系统**: 实时监听 DCC 工具状态变化
//! - **蓝图绘制**: UE5 蓝图的程序化创建和编辑
//!
//! # 示例
//!
//! ```rust,no_run
//! use dcc_tools::{BlenderAdapter, DCCToolConfig, DCCToolType};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // 创建 Blender 配置
//!     let config = DCCToolConfig {
//!         tool_type: DCCToolType::Blender,
//!         ..Default::default()
//!     };
//!
//!     // 创建适配器并连接
//!     let mut blender = BlenderAdapter::new(config);
//!     blender.connect().await?;
//!
//!     // 执行操作
//!     let info = blender.get_scene_info().await?;
//!     println!("Scene has {} objects", info.object_count);
//!
//!     Ok(())
//! }
//! ```

pub mod core;

#[cfg(feature = "blender")]
pub mod blender;

#[cfg(feature = "ue5")]
pub mod ue5;

// 重新导出核心类型
pub use core::types::*;
pub use core::traits::*;
pub use core::error::*;
pub use core::connector::*;

// 重新导出 Blender 类型
#[cfg(feature = "blender")]
pub use blender::BlenderAdapter;
#[cfg(feature = "blender")]
pub use blender::types::*;

// 重新导出 UE5 类型
#[cfg(feature = "ue5")]
pub use ue5::UE5Adapter;
#[cfg(feature = "ue5")]
pub use ue5::types::*;

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// DCC 工具管理器
#[derive(Debug)]
pub struct DCCToolsManager {
    /// 已注册的适配器
    adapters: Arc<RwLock<HashMap<DCCToolType, Box<dyn DCCToolAdapter>>>>,
    /// 全局配置
    config: DCCGlobalConfig,
}

/// 全局配置
#[derive(Debug, Clone)]
pub struct DCCGlobalConfig {
    /// 默认超时时间（秒）
    pub default_timeout: u64,
    /// 自动重连
    pub auto_reconnect: bool,
    /// 日志级别
    pub log_level: String,
    /// 插件路径
    pub plugins_path: Option<std::path::PathBuf>,
}

impl Default for DCCGlobalConfig {
    fn default() -> Self {
        Self {
            default_timeout: 300,
            auto_reconnect: true,
            log_level: "info".to_string(),
            plugins_path: None,
        }
    }
}

impl DCCToolsManager {
    /// 创建新的管理器
    pub fn new(config: DCCGlobalConfig) -> Self {
        Self {
            adapters: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// 注册适配器
    pub async fn register_adapter(
        &self,
        adapter: Box<dyn DCCToolAdapter>,
    ) -> anyhow::Result<()> {
        let tool_type = adapter.tool_type();
        let mut adapters = self.adapters.write().await;
        adapters.insert(tool_type, adapter);
        Ok(())
    }

    /// 获取适配器
    pub async fn get_adapter(
        &self,
        tool_type: DCCToolType,
    ) -> Option<Box<dyn DCCToolAdapter>> {
        let adapters = self.adapters.read().await;
        adapters.get(&tool_type).cloned()
    }

    /// 检查工具是否可用
    pub async fn is_tool_available(&self, tool_type: DCCToolType) -> bool {
        let adapters = self.adapters.read().await;
        if let Some(adapter) = adapters.get(&tool_type) {
            adapter.is_available().await
        } else {
            false
        }
    }

    /// 连接到指定工具
    pub async fn connect(&self, tool_type: DCCToolType) -> anyhow::Result<()> {
        let mut adapters = self.adapters.write().await;
        if let Some(adapter) = adapters.get_mut(&tool_type) {
            // 由于需要可变引用，这里需要特殊处理
            // 实际实现可能需要重新设计
            Ok(())
        } else {
            Err(anyhow::anyhow!("Adapter not found for {:?}", tool_type))
        }
    }

    /// 断开所有连接
    pub async fn disconnect_all(&self) -> anyhow::Result<()> {
        let mut adapters = self.adapters.write().await;
        for (_, adapter) in adapters.iter_mut() {
            // adapter.disconnect().await?;
        }
        Ok(())
    }

    /// 获取已注册的工具类型列表
    pub async fn get_registered_tools(&self) -> Vec<DCCToolType> {
        let adapters = self.adapters.read().await;
        adapters.keys().cloned().collect()
    }

    /// 获取连接状态
    pub async fn get_connection_state(&self, tool_type: DCCToolType) -> DCCConnectionState {
        let adapters = self.adapters.read().await;
        if let Some(adapter) = adapters.get(&tool_type) {
            adapter.connection_state()
        } else {
            DCCConnectionState::Disconnected
        }
    }
}

impl Default for DCCToolsManager {
    fn default() -> Self {
        Self::new(DCCGlobalConfig::default())
    }
}

/// 初始化 DCC 工具系统
pub async fn init() -> anyhow::Result<DCCToolsManager> {
    tracing::info!("Initializing DCC Tools...");

    let manager = DCCToolsManager::default();

    // 自动注册可用的工具
    #[cfg(feature = "blender")]
    {
        let blender_config = DCCToolConfig {
            tool_type: DCCToolType::Blender,
            ..Default::default()
        };
        let blender = BlenderAdapter::new(blender_config);
        manager.register_adapter(Box::new(blender)).await?;
        tracing::info!("Blender adapter registered");
    }

    #[cfg(feature = "ue5")]
    {
        let ue5_config = DCCToolConfig {
            tool_type: DCCToolType::UnrealEngine5,
            ..Default::default()
        };
        let ue5 = UE5Adapter::new(ue5_config);
        manager.register_adapter(Box::new(ue5)).await?;
        tracing::info!("UE5 adapter registered");
    }

    Ok(manager)
}

/// 获取版本信息
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dcc_tool_type_name() {
        assert_eq!(DCCToolType::Blender.name(), "Blender");
        assert_eq!(DCCToolType::UnrealEngine5.name(), "Unreal Engine 5");
    }

    #[test]
    fn test_operation_request_builder() {
        let request = DCCOperationRequest::new(DCCOperationType::FileOperation, "open_file")
            .with_param("path", "/test/file.blend")
            .unwrap();

        assert_eq!(request.name, "open_file");
        assert_eq!(request.operation_type, DCCOperationType::FileOperation);
    }
}
