//! Unreal Engine 5 集成模块
//!
//! 提供与 UE5 的完整集成支持，包括蓝图操作

pub mod types;
pub mod adapter;

pub use types::*;
pub use adapter::UE5Adapter;

/// UE5 远程控制默认端口
pub const UE5_REMOTE_CONTROL_DEFAULT_PORT: u16 = 30010;

/// 获取 UE5 安装路径列表
pub fn get_default_ue5_paths() -> Vec<std::path::PathBuf> {
    vec![
        #[cfg(target_os = "windows")]
        {
            let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
            vec![
                std::path::PathBuf::from(&program_files).join("Epic Games").join("UE_5.4"),
                std::path::PathBuf::from(&program_files).join("Epic Games").join("UE_5.3"),
                std::path::PathBuf::from(&program_files).join("Epic Games").join("UE_5.2"),
                std::path::PathBuf::from(&program_files).join("Epic Games").join("UE_5.1"),
                std::path::PathBuf::from(&program_files).join("Epic Games").join("UE_5.0"),
            ]
        },
        #[cfg(target_os = "macos")]
        vec![
            std::path::PathBuf::from("/Users/Shared/Epic Games/UE_5.4"),
            std::path::PathBuf::from("/Users/Shared/Epic Games/UE_5.3"),
        ],
        #[cfg(target_os = "linux")]
        vec![
            std::path::PathBuf::from("/opt/UnrealEngine"),
            std::path::PathBuf::from("/usr/local/UnrealEngine"),
        ],
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// UE5 Python API 文档 URL
pub fn get_python_api_docs_url(version: Option<(u32, u32)>) -> String {
    match version {
        Some((major, minor)) => {
            format!(
                "https://docs.unrealengine.com/{:.1}/en-US/PythonAPI/",
                major as f32 + minor as f32 / 10.0
            )
        }
        None => "https://docs.unrealengine.com/5.4/en-US/PythonAPI/".to_string(),
    }
}

/// 检查是否安装了远程控制插件
pub async fn check_remote_control_plugin(project_path: &std::path::Path) -> bool {
    let plugins_file = project_path.join("Plugins").join("RemoteControl").join("RemoteControl.uplugin");
    plugins_file.exists()
}
