//! DCC 工具核心 Trait 定义

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc;

use super::types::*;

/// DCC 工具适配器 Trait
#[async_trait]
pub trait DCCToolAdapter: Send + Sync + std::fmt::Debug {
    /// 获取工具类型
    fn tool_type(&self) -> DCCToolType;

    /// 获取工具配置
    fn config(&self) -> &DCCToolConfig;

    /// 更新配置
    async fn update_config(&mut self, config: DCCToolConfig) -> anyhow::Result<()>;

    /// 检查工具是否可用
    async fn is_available(&self) -> bool;

    /// 连接到工具
    async fn connect(&mut self) -> anyhow::Result<()>;

    /// 断开连接
    async fn disconnect(&mut self) -> anyhow::Result<()>;

    /// 获取连接状态
    fn connection_state(&self) -> DCCConnectionState;

    /// 执行操作
    async fn execute_operation(
        &self,
        request: DCCOperationRequest,
    ) -> anyhow::Result<DCCOperationResult>;

    /// 批量执行操作
    async fn execute_batch(
        &self,
        requests: Vec<DCCOperationRequest>,
    ) -> anyhow::Result<Vec<DCCOperationResult>> {
        let mut results = Vec::with_capacity(requests.len());
        for request in requests {
            let result = self.execute_operation(request).await?;
            results.push(result);
        }
        Ok(results)
    }

    /// 订阅事件
    async fn subscribe_events(&self) -> anyhow::Result<mpsc::Receiver<DCCEvent>>;
}

/// 文件操作 Trait
#[async_trait]
pub trait DCCFileOperations: DCCToolAdapter {
    /// 打开文件
    async fn open_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::FileOperation, "open_file")
            .with_param("path", path)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 保存文件
    async fn save_file(&self, path: Option<&PathBuf>) -> anyhow::Result<()> {
        let mut request = DCCOperationRequest::new(DCCOperationType::FileOperation, "save_file");
        if let Some(p) = path {
            request = request.with_param("path", p)?;
        }
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 导入文件
    async fn import_file(&self, path: &PathBuf, options: HashMap<String, serde_json::Value>) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::FileOperation, "import_file")
            .with_param("path", path)?
            .with_param("options", options)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 导出文件
    async fn export_file(
        &self,
        path: &PathBuf,
        format: &str,
        options: HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::FileOperation, "export_file")
            .with_param("path", path)?
            .with_param("format", format)?
            .with_param("options", options)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 获取当前场景信息
    async fn get_scene_info(&self) -> anyhow::Result<SceneInfo> {
        let request = DCCOperationRequest::new(DCCOperationType::Query, "get_scene_info");
        let result = self.execute_operation(request).await?;
        if result.success {
            let info: SceneInfo = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(info)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

/// 场景操作 Trait
#[async_trait]
pub trait DCCSceneOperations: DCCToolAdapter {
    /// 获取所有对象
    async fn get_all_objects(&self) -> anyhow::Result<Vec<ObjectInfo>> {
        let request = DCCOperationRequest::new(DCCOperationType::SceneOperation, "get_all_objects");
        let result = self.execute_operation(request).await?;
        if result.success {
            let objects: Vec<ObjectInfo> = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(objects)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 获取选中对象
    async fn get_selected_objects(&self) -> anyhow::Result<Vec<ObjectInfo>> {
        let request = DCCOperationRequest::new(DCCOperationType::SceneOperation, "get_selected_objects");
        let result = self.execute_operation(request).await?;
        if result.success {
            let objects: Vec<ObjectInfo> = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(objects)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 选择对象
    async fn select_objects(&self, object_ids: &[String]) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::SceneOperation, "select_objects")
            .with_param("object_ids", object_ids)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 删除对象
    async fn delete_objects(&self, object_ids: &[String]) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::SceneOperation, "delete_objects")
            .with_param("object_ids", object_ids)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 创建对象
    async fn create_object(
        &self,
        object_type: &str,
        name: &str,
        location: Option<[f32; 3]>,
    ) -> anyhow::Result<ObjectInfo> {
        let mut request = DCCOperationRequest::new(DCCOperationType::ObjectOperation, "create_object")
            .with_param("object_type", object_type)?
            .with_param("name", name)?;
        if let Some(loc) = location {
            request = request.with_param("location", loc)?;
        }
        let result = self.execute_operation(request).await?;
        if result.success {
            let info: ObjectInfo = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(info)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 复制对象
    async fn duplicate_objects(&self, object_ids: &[String]) -> anyhow::Result<Vec<ObjectInfo>> {
        let request = DCCOperationRequest::new(DCCOperationType::ObjectOperation, "duplicate_objects")
            .with_param("object_ids", object_ids)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            let objects: Vec<ObjectInfo> = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(objects)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 变换对象
    async fn transform_object(
        &self,
        object_id: &str,
        location: Option<[f32; 3]>,
        rotation: Option<[f32; 3]>,
        scale: Option<[f32; 3]>,
    ) -> anyhow::Result<()> {
        let mut request = DCCOperationRequest::new(DCCOperationType::ObjectOperation, "transform_object")
            .with_param("object_id", object_id)?;
        if let Some(loc) = location {
            request = request.with_param("location", loc)?;
        }
        if let Some(rot) = rotation {
            request = request.with_param("rotation", rot)?;
        }
        if let Some(scl) = scale {
            request = request.with_param("scale", scl)?;
        }
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

/// 材质操作 Trait
#[async_trait]
pub trait DCCMaterialOperations: DCCToolAdapter {
    /// 获取所有材质
    async fn get_all_materials(&self) -> anyhow::Result<Vec<MaterialInfo>> {
        let request = DCCOperationRequest::new(DCCOperationType::MaterialOperation, "get_all_materials");
        let result = self.execute_operation(request).await?;
        if result.success {
            let materials: Vec<MaterialInfo> = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(materials)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 创建材质
    async fn create_material(&self, name: &str, material_type: &str) -> anyhow::Result<MaterialInfo> {
        let request = DCCOperationRequest::new(DCCOperationType::MaterialOperation, "create_material")
            .with_param("name", name)?
            .with_param("material_type", material_type)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            let info: MaterialInfo = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(info)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 应用材质到对象
    async fn apply_material(&self, material_id: &str, object_id: &str) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::MaterialOperation, "apply_material")
            .with_param("material_id", material_id)?
            .with_param("object_id", object_id)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

/// 渲染操作 Trait
#[async_trait]
pub trait DCCRenderOperations: DCCToolAdapter {
    /// 设置渲染设置
    async fn set_render_settings(&self, settings: &RenderSettings) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::RenderOperation, "set_render_settings")
            .with_param("settings", settings)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 开始渲染
    async fn start_render(&self) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::RenderOperation, "start_render");
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 渲染单帧
    async fn render_frame(&self, frame: Option<u32>) -> anyhow::Result<PathBuf> {
        let mut request = DCCOperationRequest::new(DCCOperationType::RenderOperation, "render_frame");
        if let Some(f) = frame {
            request = request.with_param("frame", f)?;
        }
        let result = self.execute_operation(request).await?;
        if result.success {
            let path: PathBuf = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(path)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

/// Python 脚本执行 Trait
#[async_trait]
pub trait DCCPythonScripting: DCCToolAdapter {
    /// 执行 Python 脚本字符串
    async fn execute_python(&self, script: &str) -> anyhow::Result<serde_json::Value> {
        let request = DCCOperationRequest::new(DCCOperationType::PythonScript, "execute_python")
            .with_param("script", script)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(result.data.unwrap_or(serde_json::Value::Null))
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 执行 Python 脚本文件
    async fn execute_python_file(&self, script_path: &PathBuf) -> anyhow::Result<serde_json::Value> {
        let request = DCCOperationRequest::new(DCCOperationType::PythonScript, "execute_python_file")
            .with_param("script_path", script_path)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(result.data.unwrap_or(serde_json::Value::Null))
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}

/// 蓝图操作 Trait（UE5 特有）
#[async_trait]
pub trait DCCBlueprintOperations: DCCToolAdapter {
    /// 获取蓝图图表
    async fn get_blueprint_graph(&self, blueprint_path: &str, graph_name: &str) -> anyhow::Result<BlueprintGraph> {
        let request = DCCOperationRequest::new(DCCOperationType::BlueprintOperation, "get_blueprint_graph")
            .with_param("blueprint_path", blueprint_path)?
            .with_param("graph_name", graph_name)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            let graph: BlueprintGraph = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(graph)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 创建蓝图节点
    async fn create_blueprint_node(
        &self,
        blueprint_path: &str,
        graph_name: &str,
        node_type: &str,
        position: [f32; 2],
    ) -> anyhow::Result<BlueprintNode> {
        let request = DCCOperationRequest::new(DCCOperationType::BlueprintOperation, "create_blueprint_node")
            .with_param("blueprint_path", blueprint_path)?
            .with_param("graph_name", graph_name)?
            .with_param("node_type", node_type)?
            .with_param("position", position)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            let node: BlueprintNode = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(node)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 连接蓝图节点
    async fn connect_blueprint_nodes(
        &self,
        blueprint_path: &str,
        graph_name: &str,
        source_node_id: uuid::Uuid,
        source_pin: &str,
        target_node_id: uuid::Uuid,
        target_pin: &str,
    ) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::BlueprintOperation, "connect_blueprint_nodes")
            .with_param("blueprint_path", blueprint_path)?
            .with_param("graph_name", graph_name)?
            .with_param("source_node_id", source_node_id)?
            .with_param("source_pin", source_pin)?
            .with_param("target_node_id", target_node_id)?
            .with_param("target_pin", target_pin)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 编译蓝图
    async fn compile_blueprint(&self, blueprint_path: &str) -> anyhow::Result<Vec<String>> {
        let request = DCCOperationRequest::new(DCCOperationType::BlueprintOperation, "compile_blueprint")
            .with_param("blueprint_path", blueprint_path)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            let errors: Vec<String> = serde_json::from_value(result.data.unwrap_or(serde_json::Value::Null))?;
            Ok(errors)
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    /// 从代码生成蓝图
    async fn generate_blueprint_from_code(
        &self,
        blueprint_path: &str,
        code_description: &str,
    ) -> anyhow::Result<()> {
        let request = DCCOperationRequest::new(DCCOperationType::BlueprintOperation, "generate_blueprint_from_code")
            .with_param("blueprint_path", blueprint_path)?
            .with_param("code_description", code_description)?;
        let result = self.execute_operation(request).await?;
        if result.success {
            Ok(())
        } else {
            Err(anyhow::anyhow!(
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }
}
