//! Blender 集成模块
//!
//! 提供与 Blender 的完整集成支持

pub mod types;
pub mod adapter;

pub use types::*;
pub use adapter::BlenderAdapter;

use std::path::PathBuf;

/// Blender 默认安装路径
pub fn get_default_blender_paths() -> Vec<PathBuf> {
    vec![
        #[cfg(target_os = "windows")]
        {
            let program_files = std::env::var("PROGRAMFILES").unwrap_or_default();
            vec![
                PathBuf::from(program_files).join("Blender Foundation").join("Blender 4.2"),
                PathBuf::from(program_files).join("Blender Foundation").join("Blender 4.1"),
                PathBuf::from(program_files).join("Blender Foundation").join("Blender 4.0"),
                PathBuf::from(program_files).join("Blender Foundation").join("Blender 3.6"),
            ]
        },
        #[cfg(target_os = "macos")]
        vec![
            PathBuf::from("/Applications/Blender.app"),
        ],
        #[cfg(target_os = "linux")]
        vec![
            PathBuf::from("/usr/share/blender"),
            PathBuf::from("/usr/local/share/blender"),
        ],
    ]
    .into_iter()
    .flatten()
    .collect()
}

/// 获取 Blender Python API 文档 URL
pub fn get_python_api_docs_url(version: Option<(u32, u32)>) -> String {
    match version {
        Some((major, minor)) => {
            format!(
                "https://docs.blender.org/api/{}.{}/",
                major, minor
            )
        }
        None => "https://docs.blender.org/api/current/".to_string(),
    }
}
