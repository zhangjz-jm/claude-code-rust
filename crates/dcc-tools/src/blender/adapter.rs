//! Blender 适配器实现
//!
//! 通过 Python API 与 Blender 通信

use async_trait::async_trait;
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::{mpsc, Mutex, RwLock};
use tracing::{debug, error, info, warn};

use crate::core::types::*;
use crate::core::traits::*;
use crate::core::error::*;
use super::types::*;

/// Blender 适配器
#[derive(Debug)]
pub struct BlenderAdapter {
    config: DCCToolConfig,
    state: Arc<RwLock<DCCConnectionState>>,
    process: Arc<Mutex<Option<Child>>>,
    stdin: Arc<Mutex<Option<tokio::process::ChildStdin>>>,
    stdout_reader: Arc<Mutex<Option<BufReader<tokio::process::ChildStdout>>>>,
    event_sender: Option<mpsc::Sender<DCCEvent>>,
}

impl BlenderAdapter {
    /// 创建新的 Blender 适配器
    pub fn new(config: DCCToolConfig) -> Self {
        Self {
            config,
            state: Arc::new(RwLock::new(DCCConnectionState::Disconnected)),
            process: Arc::new(Mutex::new(None)),
            stdin: Arc::new(Mutex::new(None)),
            stdout_reader: Arc::new(Mutex::new(None)),
            event_sender: None,
        }
    }

    /// 查找 Blender 可执行文件
    pub async fn find_blender_executable() -> Option<PathBuf> {
        // 首先检查环境变量
        if let Ok(path) = std::env::var("BLENDER_EXECUTABLE") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // 检查常见安装路径
        let common_paths: Vec<PathBuf> = vec![
            #[cfg(target_os = "windows")]
            {
                let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
                let program_files_x86 = std::env::var("PROGRAMFILES(X86)").unwrap_or_default();
                let local_app_data = std::env::var("LOCALAPPDATA").unwrap_or_default();
                vec![
                    PathBuf::from(program_files).join("Blender Foundation").join("Blender 4.2").join("blender.exe"),
                    PathBuf::from(program_files).join("Blender Foundation").join("Blender 4.1").join("blender.exe"),
                    PathBuf::from(program_files).join("Blender Foundation").join("Blender 4.0").join("blender.exe"),
                    PathBuf::from(program_files).join("Blender Foundation").join("Blender 3.6").join("blender.exe"),
                    PathBuf::from(program_files_x86).join("Blender Foundation").join("Blender 4.2").join("blender.exe"),
                    PathBuf::from(local_app_data).join("Blender Foundation").join("Blender").join("4.2").join("blender.exe"),
                    PathBuf::from("C:\\Program Files\\Blender Foundation\\Blender 4.2\\blender.exe"),
                    PathBuf::from("C:\\Program Files\\Blender Foundation\\Blender 4.1\\blender.exe"),
                    PathBuf::from("C:\\Program Files\\Blender Foundation\\Blender 4.0\\blender.exe"),
                ]
            },
            #[cfg(target_os = "macos")]
            vec![
                PathBuf::from("/Applications/Blender.app/Contents/MacOS/Blender"),
                PathBuf::from("/usr/local/bin/blender"),
                PathBuf::from("/opt/homebrew/bin/blender"),
            ],
            #[cfg(target_os = "linux")]
            vec![
                PathBuf::from("/usr/bin/blender"),
                PathBuf::from("/usr/local/bin/blender"),
                PathBuf::from("/snap/bin/blender"),
                PathBuf::from("/app/bin/blender"), // Flatpak
            ],
        ]
        .into_iter()
        .flatten()
        .collect();

        for path in common_paths {
            if path.exists() {
                return Some(path);
            }
        }

        // 尝试在 PATH 中查找
        if let Ok(output) = Command::new("which")
            .arg("blender")
            .output()
            .await
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(PathBuf::from(path));
                }
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(output) = Command::new("where")
                .arg("blender.exe")
                .output()
                .await
            {
                if output.status.success() {
                    let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    if !path.is_empty() {
                        return Some(PathBuf::from(path));
                    }
                }
            }
        }

        None
    }

    /// 获取 Blender 版本
    pub async fn get_version(&self) -> anyhow::Result<BlenderVersion> {
        let script = r#"
import bpy
import json
version = bpy.app.version
version_string = bpy.app.version_string
version_hash = bpy.app.version_hash
print(json.dumps({
    "major": version[0],
    "minor": version[1],
    "patch": version[2],
    "release_type": version_string.split('-')[1] if '-' in version_string else "stable",
    "hash": version_hash[:12]
}))
"#;

        let result = self.execute_python(script).await?;
        let version: BlenderVersion = serde_json::from_value(result)?;
        Ok(version)
    }

    /// 发送 Python 脚本到 Blender
    async fn send_python_script(&self, script: &str) -> anyhow::Result<()> {
        let mut stdin = self.stdin.lock().await;
        if let Some(stdin) = stdin.as_mut() {
            // 将脚本编码为 base64 以避免转义问题
            let encoded = base64::encode(script);
            let command = format!(
                "import base64; exec(base64.b64decode('{}').decode('utf-8'))",
                encoded
            );
            stdin.write_all(command.as_bytes()).await?;
            stdin.write_all(b"\n").await?;
            stdin.flush().await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Not connected to Blender"))
        }
    }

    /// 读取响应
    async fn read_response(&self) -> anyhow::Result<String> {
        let mut stdout_reader = self.stdout_reader.lock().await;
        if let Some(reader) = stdout_reader.as_mut() {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            Ok(line.trim().to_string())
        } else {
            Err(anyhow::anyhow!("Not connected to Blender"))
        }
    }
}

#[async_trait]
impl DCCToolAdapter for BlenderAdapter {
    fn tool_type(&self) -> DCCToolType {
        DCCToolType::Blender
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
                    Self::find_blender_executable().await
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

        let executable = match &self.config.executable_path {
            Some(path) => path.clone(),
            None => Self::find_blender_executable()
                .await
                .ok_or_else(|| anyhow::anyhow!("Blender executable not found"))?,
        };

        info!("Connecting to Blender at {:?}", executable);

        // 启动 Blender 的 Python 控制台模式
        let mut cmd = Command::new(&executable);
        cmd.arg("--python-console")
            .arg("--no-window-focus")
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = &self.config.working_directory {
            cmd.current_dir(dir);
        }

        for (key, value) in &self.config.env_vars {
            cmd.env(key, value);
        }

        let mut process = cmd.spawn()?;
        let stdin = process.stdin.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdin"))?;
        let stdout = process.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to open stdout"))?;

        let stdout_reader = BufReader::new(stdout);

        *self.process.lock().await = Some(process);
        *self.stdin.lock().await = Some(stdin);
        *self.stdout_reader.lock().await = Some(stdout_reader);

        // 等待 Blender 启动
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        *state = DCCConnectionState::Connected;
        info!("Connected to Blender successfully");

        Ok(())
    }

    async fn disconnect(&mut self) -> anyhow::Result<()> {
        let mut state = self.state.write().await;

        if let Some(mut process) = self.process.lock().await.take() {
            let _ = process.kill().await;
        }

        *self.stdin.lock().await = None;
        *self.stdout_reader.lock().await = None;
        *state = DCCConnectionState::Disconnected;

        info!("Disconnected from Blender");
        Ok(())
    }

    fn connection_state(&self) -> DCCConnectionState {
        // 使用 try_read 避免阻塞
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

        let python_script = self.build_python_script(&request)?;

        match self.send_python_script(&python_script).await {
            Ok(()) => {
                // 读取响应
                match self.read_response().await {
                    Ok(response) => {
                        let result = if response.starts_with("ERROR:") {
                            DCCOperationResult {
                                operation_id: request.id,
                                success: false,
                                data: None,
                                error: Some(response.trim_start_matches("ERROR:").trim().to_string()),
                                execution_time_ms: start_time.elapsed().as_millis() as u64,
                                timestamp: chrono::Utc::now(),
                            }
                        } else {
                            let data = serde_json::from_str(&response).ok();
                            DCCOperationResult {
                                operation_id: request.id,
                                success: true,
                                data,
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
                        error: Some(format!("Failed to read response: {}", e)),
                        execution_time_ms: start_time.elapsed().as_millis() as u64,
                        timestamp: chrono::Utc::now(),
                    }),
                }
            }
            Err(e) => Ok(DCCOperationResult {
                operation_id: request.id,
                success: false,
                data: None,
                error: Some(format!("Failed to send script: {}", e)),
                execution_time_ms: start_time.elapsed().as_millis() as u64,
                timestamp: chrono::Utc::now(),
            }),
        }
    }

    async fn subscribe_events(&self) -> anyhow::Result<mpsc::Receiver<DCCEvent>> {
        let (tx, rx) = mpsc::channel(100);
        // 事件监听实现...
        Ok(rx)
    }
}

impl BlenderAdapter {
    /// 构建 Python 脚本
    fn build_python_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let script = match request.operation_type {
            DCCOperationType::FileOperation => self.build_file_script(request),
            DCCOperationType::SceneOperation => self.build_scene_script(request),
            DCCOperationType::ObjectOperation => self.build_object_script(request),
            DCCOperationType::MaterialOperation => self.build_material_script(request),
            DCCOperationType::RenderOperation => self.build_render_script(request),
            DCCOperationType::Query => self.build_query_script(request),
            DCCOperationType::PythonScript => {
                // 直接执行 Python 脚本
                let script = request.parameters.get("script")
                    .and_then(|s| s.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing script parameter"))?;
                Ok(script.to_string())
            }
            _ => Err(anyhow::anyhow!("Unsupported operation type for Blender: {:?}", request.operation_type)),
        }?;

        // 包装脚本以捕获输出
        let wrapped = format!(
            r#"import bpy
import json
import sys

try:
    result = {{}}
{}
    print("RESULT:" + json.dumps(result))
except Exception as e:
    print("ERROR:" + str(e))
    sys.stderr.write(str(e))
"#,
            script.lines().map(|l| format!("    {}", l)).collect::<Vec<_>>().join("\n")
        );

        Ok(wrapped)
    }

    fn build_file_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let name = request.name.as_str();
        let params = &request.parameters;

        let script = match name {
            "open_file" => {
                let path = params.get("path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
                format!(
                    r#"bpy.ops.wm.open_mainfile(filepath="{}")
result["success"] = True"#,
                    path.replace("\\", "\\\\").replace("\"", "\\\"")
                )
            }
            "save_file" => {
                if let Some(path) = params.get("path").and_then(|p| p.as_str()) {
                    format!(
                        r#"bpy.ops.wm.save_as_mainfile(filepath="{}", check_existing=False)
result["success"] = True"#,
                        path.replace("\\", "\\\\").replace("\"", "\\\"")
                    )
                } else {
                    r#"bpy.ops.wm.save_mainfile()
result["success"] = True"#.to_string()
                }
            }
            "import_file" => {
                let path = params.get("path").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing path parameter"))?;
                let ext = PathBuf::from(path)
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("")
                    .to_lowercase();

                match ext.as_str() {
                    "obj" => format!(r#"bpy.ops.import_scene.obj(filepath="{}")"#, path),
                    "fbx" => format!(r#"bpy.ops.import_scene.fbx(filepath="{}")"#, path),
                    "gltf" | "glb" => format!(r#"bpy.ops.import_scene.gltf(filepath="{}")"#, path),
                    "usd" | "usda" | "usdc" => format!(r#"bpy.ops.wm.usd_import(filepath="{}")"#, path),
                    _ => return Err(anyhow::anyhow!("Unsupported import format: {}", ext)),
                } + "\nresult[\"success\"] = true"
            }
            _ => return Err(anyhow::anyhow!("Unknown file operation: {}", name)),
        };

        Ok(script)
    }

    fn build_scene_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let name = request.name.as_str();

        let script = match name {
            "get_all_objects" => r#"
result["objects"] = [{
    "id": obj.name,
    "name": obj.name,
    "object_type": obj.type,
    "location": list(obj.location),
    "rotation": list(obj.rotation_euler),
    "scale": list(obj.scale),
    "visible": obj.visible_get(),
    "selected": obj.select_get(),
    "parent_id": obj.parent.name if obj.parent else None,
    "children_ids": [child.name for child in obj.children],
    "custom_properties": {}
} for obj in bpy.context.scene.objects]
"#.to_string(),
            "get_selected_objects" => r#"
result["objects"] = [{
    "id": obj.name,
    "name": obj.name,
    "object_type": obj.type,
    "location": list(obj.location),
    "rotation": list(obj.rotation_euler),
    "scale": list(obj.scale),
    "visible": obj.visible_get(),
    "selected": True,
    "parent_id": obj.parent.name if obj.parent else None,
    "children_ids": [child.name for child in obj.children],
    "custom_properties": {}
} for obj in bpy.context.selected_objects]
"#.to_string(),
            "select_objects" => {
                let ids = request.parameters.get("object_ids")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| anyhow::anyhow!("Missing object_ids parameter"))?;
                let names: Vec<String> = ids.iter()
                    .filter_map(|v| v.as_str().map(|s| format!("'{}'", s)))
                    .collect();
                format!(
                    r#"bpy.ops.object.select_all(action='DESELECT')
for name in [{}]:
    if name in bpy.data.objects:
        bpy.data.objects[name].select_set(True)
result["success"] = True"#,
                    names.join(", ")
                )
            }
            _ => return Err(anyhow::anyhow!("Unknown scene operation: {}", name)),
        };

        Ok(script)
    }

    fn build_object_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let name = request.name.as_str();
        let params = &request.parameters;

        let script = match name {
            "create_object" => {
                let obj_type = params.get("object_type").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing object_type parameter"))?;
                let obj_name = params.get("name").and_then(|p| p.as_str())
                    .unwrap_or("NewObject");

                let create_code = match obj_type {
                    "CUBE" => "bpy.ops.mesh.primitive_cube_add()",
                    "SPHERE" | "UV_SPHERE" => "bpy.ops.mesh.primitive_uv_sphere_add()",
                    "ICO_SPHERE" => "bpy.ops.mesh.primitive_ico_sphere_add()",
                    "CYLINDER" => "bpy.ops.mesh.primitive_cylinder_add()",
                    "CONE" => "bpy.ops.mesh.primitive_cone_add()",
                    "TORUS" => "bpy.ops.mesh.primitive_torus_add()",
                    "MONKEY" => "bpy.ops.mesh.primitive_monkey_add()",
                    "PLANE" => "bpy.ops.mesh.primitive_plane_add()",
                    "LIGHT" => "bpy.ops.object.light_add(type='POINT')",
                    "CAMERA" => "bpy.ops.object.camera_add()",
                    "EMPTY" => "bpy.ops.object.empty_add(type='PLAIN_AXES')",
                    _ => return Err(anyhow::anyhow!("Unknown object type: {}", obj_type)),
                };

                format!(
                    r#"{}
obj = bpy.context.active_object
obj.name = "{}"
if {}:
    obj.location = {}
result["id"] = obj.name
result["name"] = obj.name
result["object_type"] = obj.type
result["location"] = list(obj.location)
result["rotation"] = list(obj.rotation_euler)
result["scale"] = list(obj.scale)
"#,
                    create_code,
                    obj_name,
                    params.get("location").is_some(),
                    serde_json::to_string(&params.get("location")).unwrap_or_else(|_| "[0, 0, 0]".to_string())
                )
            }
            "delete_objects" => {
                r#"bpy.ops.object.delete()
result["success"] = True"#.to_string()
            }
            "transform_object" => {
                let obj_id = params.get("object_id").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing object_id parameter"))?;
                let location = params.get("location").cloned();
                let rotation = params.get("rotation").cloned();
                let scale = params.get("scale").cloned();

                let mut code = format!(r#"obj = bpy.data.objects["{}"]"#, obj_id);
                if let Some(loc) = location {
                    code.push_str(&format!("\nobj.location = {}", loc));
                }
                if let Some(rot) = rotation {
                    code.push_str(&format!("\nobj.rotation_euler = {}", rot));
                }
                if let Some(scl) = scale {
                    code.push_str(&format!("\nobj.scale = {}", scl));
                }
                code.push_str("\nresult[\"success\"] = True");
                code
            }
            _ => return Err(anyhow::anyhow!("Unknown object operation: {}", name)),
        };

        Ok(script)
    }

    fn build_material_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let name = request.name.as_str();
        let params = &request.parameters;

        let script = match name {
            "get_all_materials" => r#"
result["materials"] = [{
    "id": mat.name,
    "name": mat.name,
    "material_type": "PrincipledBSDF" if mat.use_nodes and mat.node_tree else "Basic",
    "base_color": list(mat.diffuse_color) if not mat.use_nodes else [1, 1, 1, 1],
    "metallic": 0.0,
    "roughness": mat.roughness,
    "normal_map": None,
    "texture_maps": {}
} for mat in bpy.data.materials]
"#.to_string(),
            "create_material" => {
                let mat_name = params.get("name").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;
                format!(
                    r#"mat = bpy.data.materials.new(name="{}")
mat.use_nodes = True
result["id"] = mat.name
result["name"] = mat.name
result["material_type"] = "PrincipledBSDF"
result["base_color"] = [1, 1, 1, 1]
result["metallic"] = 0.0
result["roughness"] = 0.5
"#,
                    mat_name
                )
            }
            "apply_material" => {
                let mat_id = params.get("material_id").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing material_id parameter"))?;
                let obj_id = params.get("object_id").and_then(|p| p.as_str())
                    .ok_or_else(|| anyhow::anyhow!("Missing object_id parameter"))?;
                format!(
                    r#"obj = bpy.data.objects["{}"]
mat = bpy.data.materials["{}"]
if obj.data.materials:
    obj.data.materials[0] = mat
else:
    obj.data.materials.append(mat)
result["success"] = True
"#,
                    obj_id, mat_id
                )
            }
            _ => return Err(anyhow::anyhow!("Unknown material operation: {}", name)),
        };

        Ok(script)
    }

    fn build_render_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let name = request.name.as_str();
        let params = &request.parameters;

        let script = match name {
            "set_render_settings" => {
                let settings = params.get("settings").ok_or_else(|| anyhow::anyhow!("Missing settings parameter"))?;
                let engine = settings.get("engine").and_then(|e| e.as_str()).unwrap_or("CYCLES");
                let width = settings.get("width").and_then(|w| w.as_u64()).unwrap_or(1920);
                let height = settings.get("height").and_then(|h| h.as_u64()).unwrap_or(1080);
                let samples = settings.get("samples").and_then(|s| s.as_u64()).unwrap_or(128);

                format!(
                    r#"bpy.context.scene.render.engine = "{}"
bpy.context.scene.render.resolution_x = {}
bpy.context.scene.render.resolution_y = {}
if "{}" == "CYCLES":
    bpy.context.scene.cycles.samples = {}
result["success"] = True
"#,
                    engine, width, height, engine, samples
                )
            }
            "start_render" => {
                let output_path = params.get("output_path").and_then(|p| p.as_str())
                    .unwrap_or("/tmp/render.png");
                format!(
                    r#"bpy.context.scene.render.filepath = "{}"
bpy.ops.render.render(write_still=True)
result["output_path"] = "{}"
"#,
                    output_path, output_path
                )
            }
            "render_frame" => {
                let frame = params.get("frame").and_then(|f| f.as_u64());
                let mut code = String::new();
                if let Some(f) = frame {
                    code.push_str(&format!("bpy.context.scene.frame_current = {}\n", f));
                }
                code.push_str(r#"bpy.ops.render.render(write_still=True)
result["output_path"] = bpy.context.scene.render.filepath
"#);
                code
            }
            _ => return Err(anyhow::anyhow!("Unknown render operation: {}", name)),
        };

        Ok(script)
    }

    fn build_query_script(&self, request: &DCCOperationRequest) -> anyhow::Result<String> {
        let name = request.name.as_str();

        let script = match name {
            "get_scene_info" => r#"
result["name"] = bpy.context.scene.name
result["file_path"] = bpy.data.filepath
result["is_modified"] = bpy.data.is_dirty
result["object_count"] = len(bpy.context.scene.objects)
result["material_count"] = len(bpy.data.materials)
result["texture_count"] = len(bpy.data.textures)
result["custom_data"] = {}
"#.to_string(),
            _ => return Err(anyhow::anyhow!("Unknown query operation: {}", name)),
        };

        Ok(script)
    }
}

// 实现其他 Trait
#[async_trait]
impl DCCFileOperations for BlenderAdapter {}

#[async_trait]
impl DCCSceneOperations for BlenderAdapter {}

#[async_trait]
impl DCCMaterialOperations for BlenderAdapter {}

#[async_trait]
impl DCCRenderOperations for BlenderAdapter {}

#[async_trait]
impl DCCPythonScripting for BlenderAdapter {}
