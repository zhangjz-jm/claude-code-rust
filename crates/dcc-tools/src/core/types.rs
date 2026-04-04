//! DCC 工具核心类型定义
//!
//! 定义所有 DCC 工具通用的类型系统

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// DCC 工具类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DCCToolType {
    /// Blender
    Blender,
    /// Unreal Engine 5
    UnrealEngine5,
    /// Autodesk Maya
    Maya,
    /// Autodesk 3ds Max
    ThreeDSMax,
    /// Houdini
    Houdini,
    /// Cinema 4D
    Cinema4D,
    /// Substance Painter
    SubstancePainter,
    /// ZBrush
    ZBrush,
    /// 自定义工具
    Custom(String),
}

impl DCCToolType {
    /// 获取工具名称
    pub fn name(&self) -> String {
        match self {
            DCCToolType::Blender => "Blender".to_string(),
            DCCToolType::UnrealEngine5 => "Unreal Engine 5".to_string(),
            DCCToolType::Maya => "Autodesk Maya".to_string(),
            DCCToolType::ThreeDSMax => "Autodesk 3ds Max".to_string(),
            DCCToolType::Houdini => "SideFX Houdini".to_string(),
            DCCToolType::Cinema4D => "Cinema 4D".to_string(),
            DCCToolType::SubstancePainter => "Substance Painter".to_string(),
            DCCToolType::ZBrush => "ZBrush".to_string(),
            DCCToolType::Custom(name) => name.clone(),
        }
    }

    /// 获取默认可执行文件名
    pub fn default_executable(&self) -> String {
        match self {
            DCCToolType::Blender => {
                #[cfg(target_os = "windows")]
                return "blender.exe".to_string();
                #[cfg(target_os = "macos")]
                return "Blender".to_string();
                #[cfg(target_os = "linux")]
                return "blender".to_string();
            }
            DCCToolType::UnrealEngine5 => {
                #[cfg(target_os = "windows")]
                return "UnrealEditor.exe".to_string();
                #[cfg(target_os = "macos")]
                return "UnrealEditor".to_string();
                #[cfg(target_os = "linux")]
                return "UnrealEditor".to_string();
            }
            DCCToolType::Maya => "maya.exe".to_string(),
            DCCToolType::ThreeDSMax => "3dsmax.exe".to_string(),
            DCCToolType::Houdini => "houdini".to_string(),
            DCCToolType::Cinema4D => "Cinema 4D.exe".to_string(),
            DCCToolType::SubstancePainter => "Substance Painter.exe".to_string(),
            DCCToolType::ZBrush => "ZBrush.exe".to_string(),
            DCCToolType::Custom(_) => "".to_string(),
        }
    }
}

/// DCC 工具配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCCToolConfig {
    /// 工具类型
    pub tool_type: DCCToolType,
    /// 可执行文件路径
    pub executable_path: Option<PathBuf>,
    /// 工作目录
    pub working_directory: Option<PathBuf>,
    /// 额外环境变量
    pub env_vars: HashMap<String, String>,
    /// Python 脚本路径（用于支持 Python API 的工具）
    pub python_scripts_path: Option<PathBuf>,
    /// 插件路径
    pub plugins_path: Option<PathBuf>,
    /// 远程连接配置
    pub remote: Option<RemoteConfig>,
    /// 超时设置（秒）
    pub timeout_seconds: u64,
    /// 是否启用
    pub enabled: bool,
}

impl Default for DCCToolConfig {
    fn default() -> Self {
        Self {
            tool_type: DCCToolType::Blender,
            executable_path: None,
            working_directory: None,
            env_vars: HashMap::new(),
            python_scripts_path: None,
            plugins_path: None,
            remote: None,
            timeout_seconds: 300,
            enabled: true,
        }
    }
}

/// 远程连接配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 认证令牌
    pub auth_token: Option<String>,
    /// 使用 WebSocket
    pub use_websocket: bool,
    /// 使用 TCP Socket
    pub use_tcp: bool,
}

/// DCC 工具连接状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DCCConnectionState {
    /// 未连接
    Disconnected,
    /// 正在连接
    Connecting,
    /// 已连接
    Connected,
    /// 忙碌中
    Busy,
    /// 错误状态
    Error,
}

/// 操作类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DCCOperationType {
    /// 文件操作
    FileOperation,
    /// 场景操作
    SceneOperation,
    /// 对象操作
    ObjectOperation,
    /// 材质操作
    MaterialOperation,
    /// 动画操作
    AnimationOperation,
    /// 渲染操作
    RenderOperation,
    /// 蓝图操作（UE5）
    BlueprintOperation,
    /// Python 脚本执行
    PythonScript,
    /// 命令执行
    CommandExecution,
    /// 查询操作
    Query,
}

/// 操作请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCCOperationRequest {
    /// 操作 ID
    pub id: uuid::Uuid,
    /// 操作类型
    pub operation_type: DCCOperationType,
    /// 操作名称
    pub name: String,
    /// 操作参数
    pub parameters: serde_json::Value,
    /// 优先级（0-100，数值越大优先级越高）
    pub priority: u8,
    /// 超时时间（秒）
    pub timeout_seconds: Option<u64>,
    /// 回调 URL
    pub callback_url: Option<String>,
}

impl DCCOperationRequest {
    /// 创建新的操作请求
    pub fn new(operation_type: DCCOperationType, name: impl Into<String>) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            operation_type,
            name: name.into(),
            parameters: serde_json::Value::Object(serde_json::Map::new()),
            priority: 50,
            timeout_seconds: None,
            callback_url: None,
        }
    }

    /// 添加参数
    pub fn with_param(mut self, key: impl Into<String>, value: impl Serialize) -> anyhow::Result<Self> {
        let value = serde_json::to_value(value)?;
        self.parameters[key.into()] = value;
        Ok(self)
    }
}

/// 操作结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DCCOperationResult {
    /// 操作 ID
    pub operation_id: uuid::Uuid,
    /// 是否成功
    pub success: bool,
    /// 结果数据
    pub data: Option<serde_json::Value>,
    /// 错误信息
    pub error: Option<String>,
    /// 执行时间（毫秒）
    pub execution_time_ms: u64,
    /// 时间戳
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// 场景信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SceneInfo {
    /// 场景名称
    pub name: String,
    /// 文件路径
    pub file_path: Option<PathBuf>,
    /// 是否已修改
    pub is_modified: bool,
    /// 对象数量
    pub object_count: usize,
    /// 材质数量
    pub material_count: usize,
    /// 纹理数量
    pub texture_count: usize,
    /// 自定义数据
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// 对象信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectInfo {
    /// 对象 ID
    pub id: String,
    /// 对象名称
    pub name: String,
    /// 对象类型
    pub object_type: String,
    /// 位置
    pub location: [f32; 3],
    /// 旋转
    pub rotation: [f32; 3],
    /// 缩放
    pub scale: [f32; 3],
    /// 是否可见
    pub visible: bool,
    /// 是否选中
    pub selected: bool,
    /// 父对象 ID
    pub parent_id: Option<String>,
    /// 子对象 ID 列表
    pub children_ids: Vec<String>,
    /// 自定义属性
    pub custom_properties: HashMap<String, serde_json::Value>,
}

/// 材质信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialInfo {
    /// 材质 ID
    pub id: String,
    /// 材质名称
    pub name: String,
    /// 材质类型
    pub material_type: String,
    /// 基础颜色
    pub base_color: [f32; 4],
    /// 金属度
    pub metallic: f32,
    /// 粗糙度
    pub roughness: f32,
    /// 法线贴图路径
    pub normal_map: Option<PathBuf>,
    /// 其他贴图
    pub texture_maps: HashMap<String, PathBuf>,
}

/// 渲染设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderSettings {
    /// 渲染引擎
    pub engine: String,
    /// 输出宽度
    pub width: u32,
    /// 输出高度
    pub height: u32,
    /// 采样数
    pub samples: u32,
    /// 输出格式
    pub output_format: String,
    /// 输出路径
    pub output_path: PathBuf,
    /// 帧范围
    pub frame_range: Option<(u32, u32)>,
    /// 额外设置
    pub extra_settings: HashMap<String, serde_json::Value>,
}

impl Default for RenderSettings {
    fn default() -> Self {
        Self {
            engine: "CYCLES".to_string(),
            width: 1920,
            height: 1080,
            samples: 128,
            output_format: "PNG".to_string(),
            output_path: PathBuf::from("render.png"),
            frame_range: None,
            extra_settings: HashMap::new(),
        }
    }
}

/// 蓝图节点信息（UE5）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintNode {
    /// 节点 ID
    pub id: uuid::Uuid,
    /// 节点类型
    pub node_type: String,
    /// 节点名称
    pub name: String,
    /// 位置
    pub position: [f32; 2],
    /// 输入引脚
    pub input_pins: Vec<BlueprintPin>,
    /// 输出引脚
    pub output_pins: Vec<BlueprintPin>,
    /// 自定义数据
    pub custom_data: HashMap<String, serde_json::Value>,
}

/// 蓝图引脚
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintPin {
    /// 引脚 ID
    pub id: uuid::Uuid,
    /// 引脚名称
    pub name: String,
    /// 引脚类型
    pub pin_type: String,
    /// 数据类型
    pub data_type: String,
    /// 是否连接
    pub is_connected: bool,
    /// 连接的目标节点 ID
    pub connected_to: Vec<uuid::Uuid>,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
}

/// 蓝图连接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintConnection {
    /// 源节点 ID
    pub source_node_id: uuid::Uuid,
    /// 源引脚 ID
    pub source_pin_id: uuid::Uuid,
    /// 目标节点 ID
    pub target_node_id: uuid::Uuid,
    /// 目标引脚 ID
    pub target_pin_id: uuid::Uuid,
}

/// 蓝图图信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintGraph {
    /// 图名称
    pub name: String,
    /// 图类型
    pub graph_type: BlueprintGraphType,
    /// 节点列表
    pub nodes: Vec<BlueprintNode>,
    /// 连接列表
    pub connections: Vec<BlueprintConnection>,
    /// 变量
    pub variables: Vec<BlueprintVariable>,
    /// 函数
    pub functions: Vec<BlueprintFunction>,
}

/// 蓝图图类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlueprintGraphType {
    /// 事件图表
    EventGraph,
    /// 函数图表
    FunctionGraph,
    /// 宏图表
    MacroGraph,
    /// 动画图表
    AnimationGraph,
    /// 构造脚本
    ConstructionScript,
}

/// 蓝图变量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintVariable {
    /// 变量名
    pub name: String,
    /// 变量类型
    pub variable_type: String,
    /// 默认值
    pub default_value: Option<serde_json::Value>,
    /// 是否可编辑
    pub editable: bool,
    /// 类别
    pub category: String,
}

/// 蓝图函数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlueprintFunction {
    /// 函数名
    pub name: String,
    /// 输入参数
    pub inputs: Vec<BlueprintPin>,
    /// 输出参数
    pub outputs: Vec<BlueprintPin>,
    /// 是否是纯函数
    pub pure: bool,
}

/// DCC 工具事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DCCEvent {
    /// 连接状态改变
    ConnectionStateChanged {
        tool_type: DCCToolType,
        state: DCCConnectionState,
    },
    /// 场景加载
    SceneLoaded {
        scene_path: PathBuf,
        scene_name: String,
    },
    /// 场景保存
    SceneSaved {
        scene_path: PathBuf,
    },
    /// 对象选中改变
    SelectionChanged {
        selected_ids: Vec<String>,
    },
    /// 渲染完成
    RenderCompleted {
        output_path: PathBuf,
        render_time_ms: u64,
    },
    /// 错误
    Error {
        message: String,
        details: Option<String>,
    },
    /// 自定义事件
    Custom {
        event_type: String,
        data: serde_json::Value,
    },
}
