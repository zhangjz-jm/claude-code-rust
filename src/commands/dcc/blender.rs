//! Blender 命令实现

use crate::commands::types::{CommandContext, CommandResult, CommandResultDisplay};
use anyhow::Result;

/// 执行 Blender 命令
pub async fn execute(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    // 移除 "blender " 前缀
    let args = args.strip_prefix("blender ").unwrap_or(args);
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.is_empty() || parts[0] == "--help" || parts[0] == "-h" {
        return Ok(CommandResult {
            content: get_help_text(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    match parts[0] {
        "connect" => connect(context).await,
        "disconnect" => disconnect(context).await,
        "status" => status(context).await,
        "open" | "load" => open_file(context).await,
        "save" => save_file(context).await,
        "scene" => scene_command(context).await,
        "object" => object_command(context).await,
        "render" => render_command(context).await,
        "python" | "py" => execute_python(context).await,
        "import" => import_file(context).await,
        "export" => export_file(context).await,
        _ => Ok(CommandResult {
            content: format!("Unknown Blender command: {}\n\nUse '/dcc blender --help' for available commands.", parts[0]),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        }),
    }
}

async fn connect(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    let mut executable_path: Option<String> = None;
    let mut working_dir: Option<String> = None;

    let mut i = 1;
    while i < parts.len() {
        match parts[i] {
            "--path" | "-p" => {
                if i + 1 < parts.len() {
                    executable_path = Some(parts[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            "--workdir" | "-w" => {
                if i + 1 < parts.len() {
                    working_dir = Some(parts[i + 1].to_string());
                    i += 2;
                } else {
                    i += 1;
                }
            }
            _ => i += 1,
        }
    }

    let path_msg = if let Some(path) = executable_path {
        format!(" using executable at: {}", path)
    } else {
        " (auto-detecting Blender installation)".to_string()
    };

    Ok(CommandResult {
        content: format!("Connecting to Blender{}...\n\n[This would initialize the Blender adapter and establish connection]", path_msg),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.connect".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

async fn disconnect(_context: CommandContext) -> Result<CommandResult> {
    Ok(CommandResult {
        content: "Disconnecting from Blender...\n\n[This would close the connection to Blender]".to_string(),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.disconnect".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

async fn status(_context: CommandContext) -> Result<CommandResult> {
    Ok(CommandResult {
        content: r#"Blender Status:

Connection: Disconnected
Version: Not detected
Scene: None
Objects: 0

Use '/dcc blender connect' to establish a connection.
"#
        .to_string(),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec![],
        next_input: None,
        submit_next_input: false,
    })
}

async fn open_file(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender open <file_path>\n\nOpens a Blender file.".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    let file_path = parts[1];

    Ok(CommandResult {
        content: format!("Opening Blender file: {}\n\n[This would open the file in the connected Blender instance]", file_path),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.open".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

async fn save_file(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    let file_path = if parts.len() > 1 {
        Some(parts[1])
    } else {
        None
    };

    let msg = if let Some(path) = file_path {
        format!("Saving Blender scene to: {}", path)
    } else {
        "Saving Blender scene (current file)".to_string()
    };

    Ok(CommandResult {
        content: format!("{}\n\n[This would save the current scene]", msg),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.save".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

async fn scene_command(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender scene <subcommand>\n\nSubcommands:\n  info    - Get scene information\n  clear   - Clear all objects\n  stats   - Get scene statistics".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    match parts[1] {
        "info" => {
            Ok(CommandResult {
                content: r#"Scene Information:

Name: Untitled
File: (not saved)
Objects: 0
Materials: 0
Textures: 0

[Connect to Blender to see actual scene info]"#
                    .to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec![],
                next_input: None,
                submit_next_input: false,
            })
        }
        "clear" => {
            Ok(CommandResult {
                content: "Clearing all objects from scene...\n\n[This would remove all objects from the current scene]".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec!["blender.scene.clear".to_string()],
                next_input: None,
                submit_next_input: false,
            })
        }
        "stats" => {
            Ok(CommandResult {
                content: r#"Scene Statistics:

Vertices: 0
Edges: 0
Faces: 0
Triangles: 0
Objects: 0
Memory: 0 MB

[Connect to Blender to see actual statistics]"#
                    .to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec![],
                next_input: None,
                submit_next_input: false,
            })
        }
        _ => Ok(CommandResult {
            content: format!("Unknown scene subcommand: {}", parts[1]),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        }),
    }
}

async fn object_command(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender object <subcommand> [options]\n\nSubcommands:\n  create <type> [name]  - Create a new object\n  list                   - List all objects\n  select <name>          - Select object(s)\n  delete [name]          - Delete object(s)\n  move <name>            - Move/transform object\n  duplicate <name>       - Duplicate object\n\nObject Types: CUBE, SPHERE, CYLINDER, CONE, TORUS, PLANE, CAMERA, LIGHT, EMPTY\n\nTransform Options:\n  --location x,y,z   (-l)\n  --rotation x,y,z   (-r)\n  --scale x,y,z      (-s)".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    match parts[1] {
        "create" => {
            if parts.len() < 3 {
                return Ok(CommandResult {
                    content: "Usage: /dcc blender object create <type> [name] [options]\n\nExample: /dcc blender object create cube MyCube --location 0,0,5".to_string(),
                    display: CommandResultDisplay::User,
                    should_query: false,
                    meta_messages: vec![],
                    next_input: None,
                    submit_next_input: false,
                });
            }

            let obj_type = parts[2].to_uppercase();
            let name = parts.get(3).map(|s| *s).unwrap_or("NewObject");

            Ok(CommandResult {
                content: format!("Creating {} object named '{}'...\n\n[This would create the object in Blender]", obj_type, name),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec!["blender.object.create".to_string()],
                next_input: None,
                submit_next_input: false,
            })
        }
        "list" => {
            Ok(CommandResult {
                content: "Objects in scene:\n\n(No objects - connect to Blender to see actual objects)".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec![],
                next_input: None,
                submit_next_input: false,
            })
        }
        "select" => {
            if parts.len() < 3 {
                return Ok(CommandResult {
                    content: "Usage: /dcc blender object select <name_pattern>\n\nUse 'all' to select all objects.".to_string(),
                    display: CommandResultDisplay::User,
                    should_query: false,
                    meta_messages: vec![],
                    next_input: None,
                    submit_next_input: false,
                });
            }

            Ok(CommandResult {
                content: format!("Selecting object(s) matching '{}'...", parts[2]),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec!["blender.object.select".to_string()],
                next_input: None,
                submit_next_input: false,
            })
        }
        "delete" => {
            let target = parts.get(2).map(|s| *s).unwrap_or("selected");
            Ok(CommandResult {
                content: format!("Deleting '{}' object(s)...", target),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec!["blender.object.delete".to_string()],
                next_input: None,
                submit_next_input: false,
            })
        }
        _ => Ok(CommandResult {
            content: format!("Unknown object subcommand: {}", parts[1]),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        }),
    }
}

async fn render_command(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender render <subcommand> [options]\n\nSubcommands:\n  settings              - Show/change render settings\n  start                 - Start rendering\n  frame <number>        - Render specific frame\n  animation             - Render animation\n\nOptions:\n  --engine <engine>     - Render engine (CYCLES, EEVEE, WORKBENCH)\n  --samples <n>         - Sample count\n  --resolution w,h      - Output resolution\n  --output <path>       - Output file path".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    match parts[1] {
        "settings" => {
            Ok(CommandResult {
                content: r#"Render Settings:

Engine: CYCLES
Resolution: 1920x1080
Samples: 128
Output: /tmp/render.png

Use '/dcc blender render start' to begin rendering."#
                    .to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec![],
                next_input: None,
                submit_next_input: false,
            })
        }
        "start" | "frame" => {
            let frame_info = if parts[1] == "frame" && parts.len() > 2 {
                format!(" frame {}", parts[2])
            } else {
                "".to_string()
            };

            Ok(CommandResult {
                content: format!("Starting render{}...\n\n[This would initiate the render process in Blender]", frame_info),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec!["blender.render.start".to_string()],
                next_input: None,
                submit_next_input: false,
            })
        }
        "animation" => {
            Ok(CommandResult {
                content: "Starting animation render...\n\n[This would render the full animation sequence]".to_string(),
                display: CommandResultDisplay::User,
                should_query: false,
                meta_messages: vec!["blender.render.animation".to_string()],
                next_input: None,
                submit_next_input: false,
            })
        }
        _ => Ok(CommandResult {
            content: format!("Unknown render subcommand: {}", parts[1]),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        }),
    }
}

async fn execute_python(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender python <script_or_file>\n\nExecute Python script in Blender.\n\nExamples:\n  /dcc blender python 'print(bpy.data.objects)'\n  /dcc blender python /path/to/script.py".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    // 获取脚本（可能是引号包裹的多词字符串）
    let script = args.strip_prefix("python ").unwrap_or(args);

    Ok(CommandResult {
        content: format!("Executing Python script in Blender:\n```python\n{}\n```\n\n[This would execute the script]", script),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.python".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

async fn import_file(context: CommandContext) -> Result> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender import <file_path> [options]\n\nSupported formats: OBJ, FBX, GLTF/GLB, USD, STL, PLY\n\nOptions:\n  --scale <factor>     - Scale factor\n  --location x,y,z    - Import location".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    let file_path = parts[1];

    Ok(CommandResult {
        content: format!("Importing file: {}\n\n[This would import the file into the current scene]", file_path),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.import".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

async fn export_file(context: CommandContext) -> Result<CommandResult> {
    let args = context.args.trim();
    let parts: Vec<&str> = args.split_whitespace().collect();

    if parts.len() < 2 {
        return Ok(CommandResult {
            content: "Usage: /dcc blender export <file_path> [options]\n\nExport formats: OBJ, FBX, GLTF/GLB, USD, STL, PLY\n\nOptions:\n  --selected            - Export only selected objects\n  --format <format>    - Force specific format".to_string(),
            display: CommandResultDisplay::User,
            should_query: false,
            meta_messages: vec![],
            next_input: None,
            submit_next_input: false,
        });
    }

    let file_path = parts[1];

    Ok(CommandResult {
        content: format!("Exporting to file: {}\n\n[This would export the scene/objects to the file]", file_path),
        display: CommandResultDisplay::User,
        should_query: false,
        meta_messages: vec!["blender.export".to_string()],
        next_input: None,
        submit_next_input: false,
    })
}

fn get_help_text() -> String {
    r#"Blender Commands

USAGE:
    /dcc blender <command> [options]

COMMANDS:
    connect [options]       Connect to Blender
    disconnect              Disconnect from Blender
    status                  Show connection status
    open <path>             Open a .blend file
    save [path]             Save current scene
    scene <subcommand>      Scene operations (info, clear, stats)
    object <subcommand>     Object operations (create, list, select, delete)
    render <subcommand>     Rendering (settings, start, frame, animation)
    python <script>         Execute Python script
    import <path>           Import file (OBJ, FBX, GLTF, etc.)
    export <path>           Export file

CONNECT OPTIONS:
    --path, -p <path>       Path to Blender executable
    --workdir, -w <path>    Working directory

OBJECT CREATE OPTIONS:
    --location, -l x,y,z    Object position
    --rotation, -r x,y,z    Object rotation (degrees)
    --scale, -s x,y,z       Object scale

EXAMPLES:
    /dcc blender connect
    /dcc blender open /path/to/scene.blend
    /dcc blender object create cube MyCube --location 0,0,5
    /dcc blender object create sphere --scale 2,2,2
    /dcc blender render start --engine CYCLES --samples 256
    /dcc blender python 'bpy.ops.mesh.primitive_monkey_add()'
"#
    .to_string()
}
