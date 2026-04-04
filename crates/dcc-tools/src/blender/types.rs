//! Blender 特有类型定义

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Blender 版本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderVersion {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub release_type: String,
    pub hash: String,
}

impl BlenderVersion {
    pub fn to_string(&self) -> String {
        format!(
            "{}.{}.{}-{} ({})",
            self.major, self.minor, self.patch, self.release_type, self.hash
        )
    }
}

/// Blender 场景模式
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlenderMode {
    Object,
    Edit,
    Sculpt,
    VertexPaint,
    WeightPaint,
    TexturePaint,
    Pose,
    ParticleEdit,
}

/// Blender 渲染引擎
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlenderRenderEngine {
    Eevee,
    Cycles,
    Workbench,
}

/// Blender 对象类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlenderObjectType {
    Mesh,
    Curve,
    Surface,
    Meta,
    Font,
    Armature,
    Lattice,
    Empty,
    GPencil,
    Camera,
    Light,
    LightProbe,
    Speaker,
}

/// Blender 修改器类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlenderModifierType {
    Subdivision,
    Mirror,
    Boolean,
    Array,
    Solidify,
    Bevel,
    Displace,
    Subsurf,
    Multires,
    Skin,
    Ocean,
    Cloth,
    SoftBody,
    Fluid,
    Smoke,
    DynamicPaint,
    ParticleSystem,
}

/// Blender 材质节点类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlenderNodeType {
    PrincipledBSDF,
    DiffuseBSDF,
    GlossyBSDF,
    GlassBSDF,
    Emission,
    MixShader,
    AddShader,
    ImageTexture,
    NoiseTexture,
    VoronoiTexture,
    Mapping,
    TextureCoordinate,
    OutputMaterial,
}

/// Blender 场景统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderSceneStats {
    pub vertices: usize,
    pub edges: usize,
    pub faces: usize,
    pub triangles: usize,
    pub objects: usize,
    pub memory_usage_mb: f64,
}

/// Blender 渲染层设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderRenderLayer {
    pub name: String,
    pub use_solid: bool,
    pub use_halo: bool,
    pub use_ztransp: bool,
    pub use_strand: bool,
    pub use_freestyle: bool,
}

/// Blender 合成器节点树
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderCompositorTree {
    pub name: String,
    pub use_nodes: bool,
    pub nodes: Vec<BlenderCompositorNode>,
    pub links: Vec<BlenderCompositorLink>,
}

/// Blender 合成器节点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderCompositorNode {
    pub name: String,
    pub node_type: String,
    pub location: [f32; 2],
    pub inputs: Vec<BlenderNodeSocket>,
    pub outputs: Vec<BlenderNodeSocket>,
}

/// Blender 节点接口
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderNodeSocket {
    pub name: String,
    pub socket_type: String,
    pub is_linked: bool,
    pub default_value: Option<serde_json::Value>,
}

/// Blender 合成器链接
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderCompositorLink {
    pub from_node: String,
    pub from_socket: String,
    pub to_node: String,
    pub to_socket: String,
}

/// Blender 动画关键帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderKeyframe {
    pub frame: f32,
    pub value: f32,
    pub interpolation: String, // BEZIER, LINEAR, CONSTANT
    pub handle_left: [f32; 2],
    pub handle_right: [f32; 2],
}

/// Blender 动画曲线
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderFCurve {
    pub data_path: String,
    pub array_index: i32,
    pub keyframes: Vec<BlenderKeyframe>,
}

/// Blender 骨骼
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderBone {
    pub name: String,
    pub head: [f32; 3],
    pub tail: [f32; 3],
    pub roll: f32,
    pub parent: Option<String>,
    pub children: Vec<String>,
    pub use_deform: bool,
}

/// Blender 动作
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderAction {
    pub name: String,
    pub fcurves: Vec<BlenderFCurve>,
    pub frame_start: i32,
    pub frame_end: i32,
}

/// Blender 几何节点修改器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderGeometryNodes {
    pub name: String,
    pub node_tree: String,
    pub inputs: HashMap<String, serde_json::Value>,
}

/// Blender Python API 包装
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderPythonCommand {
    pub command_type: BlenderPythonCommandType,
    pub script: String,
    pub wait_for_response: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlenderPythonCommandType {
    Execute,
    Eval,
    Interactive,
}

/// Blender 插件信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderAddonInfo {
    pub name: String,
    pub version: [u32; 3],
    pub author: String,
    pub description: String,
    pub enabled: bool,
    pub filepath: Option<PathBuf>,
}

/// Blender 偏好设置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlenderPreferences {
    pub autosave_interval: u32,
    pub undo_steps: u32,
    pub language: String,
    pub gpu_backend: String,
    pub render_device: String,
}
