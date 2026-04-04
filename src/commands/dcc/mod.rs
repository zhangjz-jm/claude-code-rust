//! DCC 工具命令模块
//!
//! 提供与 Blender、Unreal Engine 5 等 DCC 工具的 CLI 交互

pub mod blender;
pub mod ue5;

use crate::commands::types::{CommandContext, CommandResult, CommandResultDisplay};
use anyhow::Result;

/// DCC 命令总入口
pub async fn execute(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(CommandResult {
            content: "Usage: /dcc <tool> <command> [options]\n\nAvailable tools:\n  blender - Blender 3D\n  ue5     - Unreal Engine 5\n\nUse '/dcc <tool> --help' for tool-specific help.".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    match parts[0] {
        "blender" => blender::execute(context).await,
        "ue5" | "unreal" => ue5::execute(context).await,
        "--help" | "-h" => Ok(CommandResult {
            content: get_help_text(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        }),
        _ => Ok(CommandResult {
            content: format!("Unknown DCC tool: {}\n\nAvailable tools:\n  blender - Blender 3D\n  ue5     - Unreal Engine 5", parts[0]),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        }),
    }
}

fn get_help_text() -> String {
    r#"DCC Tools - Digital Content Creation Integration

USAGE:
    /dcc <tool> <command> [options]

TOOLS:
    blender    Blender 3D integration
    ue5        Unreal Engine 5 integration

EXAMPLES:
    /dcc blender connect                    # Connect to Blender
    /dcc blender scene info                 # Get scene information
    /dcc blender object create cube         # Create a cube
    /dcc blender render                     # Render current scene

    /dcc ue5 connect                        # Connect to UE5
    /dcc ue5 level info                     # Get level information
    /dcc ue5 blueprint create MyBP          # Create a blueprint
    /dcc ue5 blueprint edit MyBP            # Edit blueprint nodes

Use '/dcc <tool> --help' for detailed tool-specific commands.
"#
    .to_string()
}

/// 解析位置参数
pub fn parse_transform(args: &str) -> Option<([f32; 3], [f32; 3], [f32; 3])> {
    // 格式: --location x,y,z --rotation x,y,z --scale x,y,z
    let mut location = [0.0f32; 3];
    let mut rotation = [0.0f32; 3];
    let mut scale = [1.0f32; 3];

    let parts: Vec<&str> = args.split_whitespace().collect();
    let mut i = 0;

    while i < parts.len() {
        match parts[i] {
            "--location" | "-l" => {
                if i + 1 < parts.len() {
                    let coords: Vec<f32> = parts[i + 1]
                        .split(',')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if coords.len() >= 3 {
                        location = [coords[0], coords[1], coords[2]];
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--rotation" | "-r" => {
                if i + 1 < parts.len() {
                    let coords: Vec<f32> = parts[i + 1]
                        .split(',')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if coords.len() >= 3 {
                        rotation = [coords[0], coords[1], coords[2]];
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--scale" | "-s" => {
                if i + 1 < parts.len() {
                    let coords: Vec<f32> = parts[i + 1]
                        .split(',')
                        .filter_map(|s| s.parse().ok())
                        .collect();
                    if coords.len() >= 3 {
                        scale = [coords[0], coords[1], coords[2]];
                    }
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    Some((location, rotation, scale))
}
