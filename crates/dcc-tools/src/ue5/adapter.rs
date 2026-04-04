//! Unreal Engine 5 适配器实现
//!
//! 通过 Python API 和远程控制插件与 UE5 通信

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use futures::{SinkExt, StreamExt};

use crate::core::types::*;
use crate::core::traits::*;
use crate::core::error::*;
use super::types::*;

/// UE5 适配器
#[derive(Debug)]
pub struct UE5Adapter {
    config: DCCToolConfig,
    state: Arc<RwLock<DCCConnectionState>>,
    process: Arc<Mutex<Option<Child>>>,
    ws_connection: Arc<Mutex<Option<WebSocketStream<MaybeTlsStream<TcpStream>>>>>,
    project_path: Arc<RwLock<Option<PathBuf>>>,
    event_sender: Option<mpsc::Sender<DCCEvent>>,
}

impl UE5Adapter {
    /// 创建新的 UE5 适配器
    pub fn new(config: DCCToolConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(DCCConnectionState::Disconnected)),
            process: Arc::new(Mutex::new(None)),
            ws_connection: Arc::new(Mutex::new(None)),
            project_path: Arc::new(RwLock::new(None)),
            event_sender: None,
        }
    }

    /// 查找 UE5 可执行文件
    pub async fn find_ue5_executable() -> Option<PathBuf> {
        // 检查环境变量
        if let Ok(path) = std::env::var("UE5_EDITOR") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // 检查 Epic Games Launcher 安装路径
        #[cfg(target_os = "windows")]
        {
            let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
            let program_files_x86 = std::env::var("PROGRAMFILES(X86)").unwrap_or_default();

            let epic_paths = vec![
                PathBuf::from(program_files).join("Epic Games").join("UE_5.4").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from(program_files).join("Epic Games").join("UE_5.3").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from(program_files).join("Epic Games").join("UE_5.2").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from(program_files).join("Epic Games").join("UE_5.1").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from(program_files).join("Epic Games").join("UE_5.0").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from(program_files_x86).join("Epic Games").join("UE_5.4").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from(program_files_x86).join("Epic Games").join("UE_5.3").join("Engine").join("Binaries").join("Win64").join("UnrealEditor.exe"),
                PathBuf::from("C:\\Program Files\\Epic Games\\UE_5.4\\Engine\\Binaries\\Win64\\UnrealEditor.exe"),
                PathBuf::from("C:\\Program Files\\Epic Games\\UE_5.3\\Engine\\Binaries\\Win64\\UnrealEditor.exe"),
                PathBuf::from("C:\\Program Files\\Epic Games\\UE_5.2\\Engine\\Binaries\\Win64\\UnrealEditor.exe"),
            ];

            for path in epic_paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            let paths = vec![
                PathBuf::from("/Users/Shared/Epic Games/UE_5.4/Engine/Binaries/Mac/UnrealEditor.app/Contents/MacOS/UnrealEditor"),
                PathBuf::from("/Users/Shared/Epic Games/UE_5.3/Engine/Binaries/Mac/UnrealEditor.app/Contents/MacOS/UnrealEditor"),
                PathBuf::from("/Applications/Unreal Engine.app/Contents/MacOS/UnrealEditor"),
            ];

            for path in paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            let paths = vec![
                PathBuf::from("/opt/UnrealEngine/Engine/Binaries/Linux/UnrealEditor"),
                PathBuf::from("/usr/local/UnrealEngine/Engine/Binaries/Linux/UnrealEditor"),
                PathBuf::from("~/UnrealEngine/Engine/Binaries/Linux/UnrealEditor"),
            ];

            for path in paths {
                if path.exists() {
                    return Some(path);
                }
            }
        }

        None
    }

    /// 连接到 UE5 远程控制 WebSocket
    async fn connect_websocket(&self,
        host: &str,
        port: u16,
        auth_token: Option<&str>,
    ) -> anyhow::Result<()> {
        let url = format!("ws://{}:{}/remote/object/call", host, port);
        info!("Connecting to UE5 Remote Control at {}", url);

        let (mut ws, _) = connect_async(&url)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;

        // 发送认证消息（如果需要）
        if let Some(token) = auth_token {
            let auth_msg = serde_json::json!({
                "type": "auth",
                "token": token
            });
            ws.send(tokio_tungstenite::tungstenite::Message::Text(
                auth_msg.to_string()
            )).await?;
        }

        *self.ws_connection.lock().await = Some(ws);
        info!("Connected to UE5 Remote Control successfully");
        Ok(())
    }

    /// 发送 WebSocket 消息并等待响应
    async fn send_ws_message(&self,
        message: &str
    ) -> anyhow::Result<String> {
        let mut ws_guard = self.ws_connection.lock().await;

        if let Some(ws) = ws_guard.as_mut() {
            // 发送消息
            ws.send(tokio_tungstenite::tungstenite::Message::Text(message.to_string()))
                .await
                .map_err(|e| anyhow::anyhow!("Failed to send WebSocket message: {}", e))?;

            // 等待响应
            loop {
                match ws.next().await {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        return Ok(text);
                    }
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Close(_))) => {
                        return Err(anyhow::anyhow!("WebSocket connection closed"));
                    }
                    Some(Err(e)) => {
                        return Err(anyhow::anyhow!("WebSocket error: {}", e));
                    }
                    _ => continue,
                }
            }
        } else {
            Err(anyhow::anyhow!("WebSocket not connected"))
        }
    }

    /// 执行 Python 脚本
    async fn execute_ue_python(
        &self,
        script: &str,
        sync: bool,
    ) -> anyhow::Result<String> {
        let command = serde_json::json!({
            "objectPath": "/Engine/EditorBlueprintResources/StandardMacros.StandardMacros",
            "functionName": "ExecutePythonCommand",
            "parameters": {
                "Command": script,
                "IsLiteral": true
            }
        });

        if sync {
            // 使用 WebSocket 同步执行
            let response = self.send_ws_message(&command.to_string()).await?;
            Ok(response)
        } else {
            // 通过远程执行
            self.send_ws_message(&command.to_string()).await?;
            Ok("{}".to_string())
        }
    }

    /// 构建蓝图操作脚本
    fn build_blueprint_script(
        &self,
        request: &DCCOperationRequest,
    ) -> anyhow::Result<String> {
        let name = request.name.as_str();
        let params = &request.parameters;

        let script = match name {
            "get_blueprint_graph" => {
                let blueprint_path = params.get("blueprint_path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing blueprint_path parameter"))?;
                let graph_name = params.get("graph_name").and_then(|g| g.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing graph_name parameter"))?;

                format!(
                    r#"import unreal

blueprint = unreal.EditorAssetLibrary.load_blueprint_class("{0}")
if blueprint:
    bp = unreal.get_editor_subsystem(unreal.EditorUtilitySubsystem).get_blueprint_from_class(blueprint)
    graph = next((g for g in bp.graphs if g.graph_name == "{1}"), None)
    if graph:
        nodes = []
        for node in graph.nodes:
            node_info = {{
                "id": str(node.node_guid),
                "node_type": node.get_class().get_name(),
                "name": node.get_name(),
                "position": [node.node_pos_x, node.node_pos_y],
                "input_pins": [],
                "output_pins": [],
                "custom_data": {{}}
            }}
            # 获取引脚信息
            for pin in node.get_all_pins():
                pin_info = {{
                    "id": str(pin.pin_id),
                    "name": pin.pin_name,
                    "pin_type": pin.pin_type.get_display_name(),
                    "data_type": str(pin.pin_category),
                    "is_connected": pin.has_any_connections(),
                    "connected_to": [str(p.pin_id) for p in pin.linked_to],
                    "default_value": None
                }}
                if pin.direction == unreal.EdGraphPinDirection.EGPD_Input:
                    node_info["input_pins"].append(pin_info)
                else:
                    node_info["output_pins"].append(pin_info)
            nodes.append(node_info)

        result = {{"name": graph.graph_name, "nodes": nodes, "connections": []}}
    else:
        result = {{"error": "Graph not found"}}
else:
    result = {{"error": "Blueprint not found"}}

json.dumps(result)
"#,
                    blueprint_path, graph_name
                )
            }
            "create_blueprint_node" => {
                let blueprint_path = params.get("blueprint_path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing blueprint_path parameter"))?;
                let graph_name = params.get("graph_name").and_then(|g| g.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing graph_name parameter"))?;
                let node_type = params.get("node_type").and_then(|t| t.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing node_type parameter"))?;
                let position = params.get("position").and_then(|p| p.as_array())
                    .map(|arr| {
                        let x = arr.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        let y = arr.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                        (x, y)
                    })
                    .unwrap_or((0.0, 0.0));

                format!(
                    r#"import unreal

blueprint = unreal.EditorAssetLibrary.load_blueprint_class("{0}")
if blueprint:
    bp = unreal.get_editor_subsystem(unreal.EditorUtilitySubsystem).get_blueprint_from_class(blueprint)
    graph = next((g for g in bp.graphs if g.graph_name == "{1}"), None)
    if graph:
        # 根据节点类型创建节点
        node_class_name = "{3}"
        if "Event" in node_class_name:
            new_node = unreal.EdGraphNode()
        elif "Function" in node_class_name:
            new_node = unreal.EdGraphNode_Comment()
        else:
            new_node = unreal.EdGraphNode()

        graph.add_node(new_node)
        new_node.node_pos_x = {4}
        new_node.node_pos_y = {5}

        result = {{
            "id": str(new_node.node_guid),
            "node_type": node_class_name,
            "name": new_node.get_name(),
            "position": [new_node.node_pos_x, new_node.node_pos_y],
            "input_pins": [],
            "output_pins": []
        }}
    else:
        result = {{"error": "Graph not found"}}
else:
    result = {{"error": "Blueprint not found"}}

json.dumps(result)
"#,
                    blueprint_path, graph_name, "", node_type, position.0, position.1
                )
            }
            "connect_blueprint_nodes" => {
                let blueprint_path = params.get("blueprint_path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing blueprint_path parameter"))?;
                let graph_name = params.get("graph_name").and_then(|g| g.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing graph_name parameter"))?;
                let source_node_id = params.get("source_node_id").and_then(|u| u.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing source_node_id parameter"))?;
                let source_pin = params.get("source_pin").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing source_pin parameter"))?;
                let target_node_id = params.get("target_node_id").and_then(|u| u.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing target_node_id parameter"))?;
                let target_pin = params.get("target_pin").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing target_pin parameter"))?;

                format!(
                    r#"import unreal

blueprint = unreal.EditorAssetLibrary.load_blueprint_class("{0}")
if blueprint:
    bp = unreal.get_editor_subsystem(unreal.EditorUtilitySubsystem).get_blueprint_from_class(blueprint)
    graph = next((g for g in bp.graphs if g.graph_name == "{1}"), None)
    if graph:
        source_node = next((n for n in graph.nodes if str(n.node_guid) == "{2}"), None)
        target_node = next((n for n in graph.nodes if str(n.node_guid) == "{4}"), None)

        if source_node and target_node:
            source_pin_obj = next((p for p in source_node.get_all_pins() if p.pin_name == "{3}"), None)
            target_pin_obj = next((p for p in target_node.get_all_pins() if p.pin_name == "{5}"), None)

            if source_pin_obj and target_pin_obj:
                unreal.EdGraphPin.make_link_to(source_pin_obj, target_pin_obj)
                result = {{"success": True}}
            else:
                result = {{"error": "Pins not found"}}
        else:
            result = {{"error": "Nodes not found"}}
    else:
        result = {{"error": "Graph not found"}}
else:
    result = {{"error": "Blueprint not found"}}

json.dumps(result)
"#,
                    blueprint_path, graph_name, source_node_id, source_pin, target_node_id, target_pin
                )
            }
            "compile_blueprint" => {
                let blueprint_path = params.get("blueprint_path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing blueprint_path parameter"))?;

                format!(
                    r#"import unreal

blueprint = unreal.EditorAssetLibrary.load_blueprint_class("{0}")
if blueprint:
    bp = unreal.get_editor_subsystem(unreal.EditorUtilitySubsystem).get_blueprint_from_class(blueprint)
    # 编译蓝图
    result_compile = unreal.EditorAssetLibrary.compile_blueprint(bp)

    # 获取编译消息
    messages = []
    for msg in result_compile:
        messages.append({{
            "message_type": msg.severity.get_display_name(),
            "node_guid": str(msg.node_guid) if msg.node_guid else "",
            "message": msg.message,
            "line": msg.line_number
        }})

    result = {{
        "success": result_compile[0].severity != unreal.EMessageSeverity.Error if result_compile else True,
        "messages": messages
    }}
else:
    result = {{"error": "Blueprint not found"}}

json.dumps(result)
"#,
                    blueprint_path
                )
            }
            "generate_blueprint_from_code" => {
                let blueprint_path = params.get("blueprint_path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing blueprint_path parameter"))?;
                let code_description = params.get("code_description").and_then(|c| c.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing code_description parameter"))?;

                format!(
                    r#"import unreal
import json

# 使用 AI 生成蓝图结构
# 这里是一个示例实现，实际应该调用 LLM API

description = """{1}"""

# 创建新蓝图
factory = unreal.BlueprintFactory()
factory.parent_class = unreal.Actor

asset_tools = unreal.AssetToolsHelpers.get_asset_tools()
blueprint = asset_tools.create_asset(
    asset_name="{0}".split('/')[-1],
    package_path="/".join("{0}".split('/')[:-1]),
    asset_class=unreal.Blueprint,
    factory=factory
)

# 获取蓝图图表
bp = unreal.get_editor_subsystem(unreal.EditorUtilitySubsystem).get_blueprint_from_class(blueprint)
graph = bp.get_editor_property('ubergraph_pages')[0]

# 根据描述创建节点（简化示例）
if "BeginPlay" in description:
    # 创建 BeginPlay 事件节点
    begin_play_node = graph.graph_add_node_template_from_filter(unreal.EdGraphNode())
    begin_play_node.node_pos_x = 0
    begin_play_node.node_pos_y = 0

result = {{"success": True, "blueprint_path": "{0}"}}
json.dumps(result)
"#,
                    blueprint_path, code_description.replace("\"", "\\\"")
                )
            }
            _ => return Err(anyhow::anyhow!("Unknown blueprint operation: {}", name)),
        };

        Ok(script)
    }
}

#[async_trait]
impl DCCToolAdapter for UE5Adapter {
    fn tool_type(&self) -> DCCToolType {
        DCCToolType::UnrealEngine5
    }

    fn config(&self) -> &DCCToolConfig {
        &self.config
    }

    async fn update_config(&mut self, config: DCCToolConfig) -> anyhow::Result<()> {
        self.config = config;
        Ok(())
    }

    async fn is_available(&self) -> bool {
        let executable = self
            .config
            .executable_path
            .clone()
            .or_else(|| {
                std::future::block_on(async {
                    Self::find_ue5_executable().await
                })
            });

        match executable {
            Some(path) => path.exists(),
            None => false,
        }
    }

    async fn connect(&mut self) -> anyhow::Result<()> {
        let mut state = self.state.write().await;
        *state = DCCConnectionState::Connecting;

        // 优先使用远程连接
        if let Some(remote_config) = &self.config.remote {
            if remote_config.use_websocket {
                self.connect_websocket(
                    &remote_config.host,
                    remote_config.port,
                    remote_config.auth_token.as_deref(),
                ).await?;

                *state = DCCConnectionState::Connected;
                return Ok(());
            }
        }

        // 否则启动本地 UE5 编辑器
        let executable = match &self.config.executable_path {
            Some(path) => path.clone(),
            None => Self::find_ue5_executable()
                .await
                .ok_or_else(|| anyhow::anyhow!("UE5 executable not found"))?,
        };

        info!("Starting UE5 Editor at {:?}", executable);

        // 启动 UE5 编辑器
        let mut cmd = Command::new(&executable);
        cmd.arg("-remotecontrol")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = &self.config.working_directory {
            cmd.current_dir(dir);
        }

        for (key, value) in &self.config.env_vars {
            cmd.env(key, value);
        }

        let process = cmd.spawn()?;
        *self.process.lock().await = Some(process);

        // 等待远程控制服务启动
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;

        // 连接 WebSocket
        self.connect_websocket("localhost", 30010, None).await?;

        *state = DCCConnectionState::Connected;
        info!("Connected to UE5 successfully");

        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        let mut state = self.state.write().await;

        // 关闭 WebSocket 连接
        if let Some(mut ws) = self.ws_connection.lock().await.take() {
            let _ = ws.close(None).await;
        }

        // 停止进程
        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.kill().await;
        }

        *state = DCCConnectionState::Disconnected;
        info!("Disconnected from UE5");
        Ok(())
    }

    fn connection_state(&self) -> DCCConnectionState {
        match self.state.try_read() {
            Ok(state) => *state,
            Err(_) => DCCConnectionState::Busy,
        }
    }

    async fn execute_operation(
        &self,
        request: DCCOperationRequest,
    ) -> anyhow::Result<DCCOperationResult> {
        let start_time = std::time::Instant::now();

        let python_script = match request.operation_type {
            DCCOperationType::BlueprintOperation => {
                self.build_blueprint_script(&request)?
            }
            DCCOperationType::FileOperation |
            DCCOperationType::SceneOperation |
            DCCOperationType::ObjectOperation |
            DCCOperationType::Query |
            DCCOperationType::RenderOperation |
            DCCOperationType::PythonScript => {
                // 构建通用 Python 脚本
                let script = request.parameters.get("script")
                    .and_then(|s| s.as_str())
                    .unwrap_or("");
                script.to_string()
            }
            _ => return Err(anyhow::anyhow!(
                "Unsupported operation type for UE5: {:?}",
                request.operation_type
            )),
        };

        match self.execute_ue_python(&python_script, true).await {
            Ok(response) => {
                let result = match serde_json::from_str::<serde_json::Value>(&response) {
                    Ok(json) => {
                        if json.get("error").is_some() {
                            DCCOperationResult {
                                operation_id: request.id,
                                success: false,
                                data: None,
                                error: json.get("error").and_then(|e| e.as_str()).map(|s| s.to_string()),
                                execution_time_ms: start_time.elapsed().as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                            }
                        } else {
                            DCCOperationResult {
                                operation_id: request.id,
                                success: true,
                                data: Some(json),
                                error: None,
                                execution_time_ms: start_time.elapsed().as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                            }
                        }
                    }
                    Err(_) => DCCOperationResult {
                        operation_id: request.id,
                        success: true,
                        data: Some(serde_json::Value::String(response)),
                        error: None,
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                    }
                };
                Ok(result)
            }
            Err(e) => Ok(DCCOperationResult {
                operation_id: request.id,
                success: false,
                data: None,
                error: Some(format!("Failed to execute Python: {}", e)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
            }),
        }
    }

    async fn subscribe_events(&self) -> anyhow::Result<mpsc::Receiver<DCCEvent>> {
        let (tx, rx) = mpsc::channel(100);
        Ok(rx)
    }
}

// 实现其他 Trait
#[async_trait]
impl DCCFileOperations for UE5Adapter {}

#[async_trait]
impl DCCSceneOperations for UE5Adapter {}

#[async_trait]
impl DCCMaterialOperations for UE5Adapter {}

#[async_trait]
impl DCCRenderOperations for UE5Adapter {}

#[async_trait]
impl DCCPythonScripting for UE5Adapter {}

#[async_trait]
impl DCCBlueprintOperations for UE5Adapter {}
