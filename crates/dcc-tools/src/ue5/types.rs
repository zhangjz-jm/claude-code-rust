//! Unreal Engine 5 特有类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// UE5 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub changelist: u32,
    pub branch: String,
}

impl UE5Version {
    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}-{}+{}",
            self.major, self.minor, self.patch, self.changelist, self.branch
        )
    }
}

/// UE5 项目信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5ProjectInfo {
    /// 项目名称
    pub name: String,
    /// 项目路径
    pub path: PathBuf,
    /// 引擎版本
    pub engine_version: String,
    /// 项目 ID
    pub project_id: String,
    /// 是否使用源引擎
    pub uses_source_engine: bool,
    /// 模块列表
    pub modules: Vec<String>,
    /// 插件列表
    pub plugins: Vec<String>,
}

/// UE5 关卡信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5LevelInfo {
    /// 关卡名称
    pub name: String,
    /// 关卡路径
    pub path: String,
    /// 是否已修改
    pub is_dirty: bool,
    /// Actor 数量
    pub actor_count: usize,
    /// 世界设置
    pub world_settings: HashMap<String, serde_json::Value>,
}

/// UE5 Actor 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5ActorInfo {
    /// Actor 标签
    pub label: String,
    /// Actor 类型
    pub actor_class: String,
    /// Actor 路径
    pub path: String,
    /// 位置
    pub location: [f32; 3],
    /// 旋转
    pub rotation: [f32; 3],
    /// 缩放
    pub scale: [f32; 3],
    /// 是否可见
    pub is_visible: bool,
    /// 是否选中
    pub is_selected: bool,
    /// 父 Actor
    pub parent: Option<String>,
    /// 子 Actor
    pub children: Vec<String>,
    /// 标签
    pub tags: Vec<String>,
    /// 组件
    pub components: Vec<UE5ComponentInfo>,
}

/// UE5 组件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5ComponentInfo {
    /// 组件名称
    pub name: String,
    /// 组件类型
    pub component_class: String,
    /// 是否可见
    pub is_visible: bool,
    /// 相对位置
    pub relative_location: [f32; 3],
    /// 相对旋转
    pub relative_rotation: [f32; 3],
    /// 相对缩放
    pub relative_scale: [f32; 3],
}

/// UE5 材质实例信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5MaterialInfo {
    /// 材质名称
    pub name: String,
    /// 材质路径
    pub path: String,
    /// 父材质
    pub parent: Option<String>,
    /// 参数
    pub parameters: HashMap<String, serde_json::Value>,
}

/// UE5 渲染设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5RenderSettings {
    /// 渲染器类型
    pub renderer: String,
    /// 输出分辨率
    pub resolution: [u32; 2],
    /// 抗锯齿方法
    pub anti_aliasing: String,
    /// 全局光照方法
    pub global_illumination: String,
    /// 反射方法
    pub reflections: String,
    /// 阴影方法
    pub shadows: String,
    /// 后期处理设置
    pub post_process: HashMap<String, serde_json::Value>,
}

/// UE5 蓝图类信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5BlueprintClass {
    /// 蓝图名称
    pub name: String,
    /// 蓝图路径
    pub path: String,
    /// 父类
    pub parent_class: String,
    /// 是否是组件
    pub is_component: bool,
    /// 接口
    pub interfaces: Vec<String>,
}

/// UE5 蓝图节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UE5NodeType {
    /// 事件
    Event,
    /// 函数调用
    FunctionCall,
    /// 变量获取
    VariableGet,
    /// 变量设置
    VariableSet,
    /// 分支
    Branch,
    /// 序列
    Sequence,
    /// For 循环
    ForLoop,
    /// While 循环
    WhileLoop,
    /// 延迟
    Delay,
    /// 时间轴
    Timeline,
    /// 自定义事件
    CustomEvent,
    /// 宏实例
    MacroInstance,
    /// 转换
    Cast,
    /// 纯函数
    PureFunction,
    /// 构造函数脚本
    ConstructionScript,
}

/// UE5 蓝图编译结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5BlueprintCompileResult {
    /// 是否成功
    pub success: bool,
    /// 错误数量
    pub num_errors: usize,
    /// 警告数量
    pub num_warnings: usize,
    /// 消息
    pub messages: Vec<UE5BlueprintMessage>,
}

/// UE5 蓝图消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5BlueprintMessage {
    /// 消息类型
    pub message_type: String,
    /// 节点 GUID
    pub node_guid: String,
    /// 消息内容
    pub message: String,
    /// 行号
    pub line: Option<usize>,
}

/// UE5 编辑器命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5EditorCommand {
    /// 命令类型
    pub command_type: UE5CommandType,
    /// 命令参数
    pub parameters: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UE5CommandType {
    /// 打开资产
    OpenAsset,
    /// 保存资产
    SaveAsset,
    /// 创建资产
    CreateAsset,
    /// 删除资产
    DeleteAsset,
    /// 运行游戏
    PlayGame,
    /// 停止游戏
    StopGame,
    /// 构建光照
    BuildLighting,
    /// 构建几何体
    BuildGeometry,
    /// 构建反射捕获
    BuildReflectionCaptures,
    /// 构建导航
    BuildNavigation,
    /// 打包项目
    PackageProject,
    /// 运行 Python 脚本
    ExecutePython,
    /// 执行控制台命令
    ExecuteConsoleCommand,
}

/// UE5 资产类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UE5AssetType {
    Blueprint,
    StaticMesh,
    SkeletalMesh,
    Material,
    MaterialInstance,
    Texture,
    TextureCube,
    SoundWave,
    SoundCue,
    Animation,
    AnimBlueprint,
    Level,
    LevelSequence,
    DataTable,
    CurveTable,
    UserDefinedEnum,
    UserDefinedStruct,
}

/// UE5 资产信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5AssetInfo {
    /// 资产名称
    pub name: String,
    /// 资产路径
    pub path: String,
    /// 资产类型
    pub asset_type: UE5AssetType,
    /// 资产大小（字节）
    pub size_bytes: u64,
    /// 创建时间
    pub created_time: chrono::DateTime<chrono::Utc>,
    /// 修改时间
    pub modified_time: chrono::DateTime<chrono::Utc>,
    /// 引用此资产的资产
    pub referencers: Vec<String>,
    /// 此资产引用的资产
    pub dependencies: Vec<String>,
}

/// UE5 编辑器设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5EditorSettings {
    /// 自动保存间隔
    pub autosave_interval_minutes: u32,
    /// 撤销步数
    pub undo_levels: u32,
    /// 实时渲染
    pub realtime: bool,
    /// 显示网格
    pub show_grid: bool,
    /// 显示边框
    pub show_bounds: bool,
    /// 显示碰撞
    pub show_collision: bool,
}

/// UE5 Python 命令
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5PythonCommand {
    /// 脚本内容
    pub script: String,
    /// 是否等待完成
    pub wait_for_completion: bool,
    /// 超时时间
    pub timeout_seconds: Option<u64>,
}

/// UE5 关卡序列信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5LevelSequenceInfo {
    /// 序列名称
    pub name: String,
    /// 序列路径
    pub path: String,
    /// 帧率
    pub frame_rate: f32,
    /// 开始帧
    pub start_frame: i32,
    /// 结束帧
    pub end_frame: i32,
    /// 轨道
    pub tracks: Vec<UE5TrackInfo>,
}

/// UE5 轨道信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5TrackInfo {
    /// 轨道名称
    pub name: String,
    /// 轨道类型
    pub track_type: String,
    /// 绑定的对象
    pub binding: String,
    /// 章节
    pub sections: Vec<UE5SectionInfo>,
}

/// UE5 章节信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5SectionInfo {
    /// 章节名称
    pub name: String,
    /// 开始时间
    pub start_time: f32,
    /// 结束时间
    pub end_time: f32,
    /// 关键帧数量
    pub keyframe_count: usize,
}

/// UE5 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5PluginInfo {
    /// 插件名称
    pub name: String,
    /// 插件路径
    pub path: PathBuf,
    /// 版本
    pub version: String,
    /// 是否启用
    pub enabled: bool,
    /// 描述
    pub description: String,
    /// 类别
    pub category: String,
    /// 创建者
    pub created_by: String,
}

/// UE5 构建设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5BuildSettings {
    /// 构建配置
    pub configuration: String,
    /// 目标平台
    pub target_platform: String,
    /// 包含调试信息
    pub include_debug_info: bool,
    /// 压缩内容
    pub compress_content: bool,
    /// 打包版本
    pub pak_files: bool,
    /// 创建安装程序
    pub create_installer: bool,
}

/// UE5 远程会话信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5RemoteSessionInfo {
    /// 会话 ID
    pub session_id: String,
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 项目路径
    pub project_path: PathBuf,
    /// 连接时间
    pub connected_at: chrono::DateTime<chrono::Utc>,
    /// 是否已认证
    pub authenticated: bool,
}

/// UE5 代码生成设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5CodeGenSettings {
    /// 生成蓝图
    pub generate_blueprint: bool,
    /// 生成 C++ 代码
    pub generate_cpp: bool,
    /// 代码风格
    pub code_style: String,
    /// 命名前缀
    pub naming_prefix: String,
    /// 输出目录
    pub output_directory: PathBuf,
}

/// UE5 资源导入设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5ImportSettings {
    /// 源文件路径
    pub source_path: PathBuf,
    /// 目标路径
    pub destination_path: String,
    /// 是否覆盖
    pub overwrite: bool,
    /// 导入选项
    pub import_options: HashMap<String, serde_json::Value>,
}

/// UE5 导航网格设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UE5NavMeshSettings {
    /// 单元大小
    pub cell_size: f32,
    /// 单元高度
    pub cell_height: f32,
    /// 代理半径
    pub agent_radius: f32,
    /// 代理高度
    pub agent_height: f32,
    /// 最大坡度
    pub agent_max_slope: f32,
    /// 最大步高
    pub agent_max_step_height: f32,
}
