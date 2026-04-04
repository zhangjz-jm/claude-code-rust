//! DCC 工具连接器
//!
//! 提供多种连接方式与 DCC 工具通信

use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::process::{Child, Command};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use super::types::*;
use super::error::*;

/// DCC 连接接口
#[async_trait::async_trait]
pub trait DCCConnection: Send + Sync {
    /// 发送消息
    async fn send(&mut self, message: &str) -> DCCResult<()>;
    /// 接收消息
    async fn receive(&mut self) -> DCCResult<Option<String>>;
    /// 关闭连接
    async fn close(&mut self) -> DCCResult<()>;
    /// 检查连接状态
    fn is_connected(&self) -> bool;
}

/// 标准输入输出连接
pub struct StdioConnection {
    process: Child,
    stdin: tokio::process::ChildStdin,
    stdout_reader: BufReader<tokio::process::ChildStdout>,
    connected: bool,
}

impl StdioConnection {
    /// 创建新的标准输入输出连接
    pub async fn new(
        executable: &std::path::Path,
        args: &[String],
        working_dir: Option<&std::path::Path>,
        env_vars: &[(String, String)],
    ) -> DCCResult<Self> {
        let mut cmd = Command::new(executable);
        cmd.args(args)
            .stdout(Stdio::piped())
            .stdin(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(dir) = working_dir {
            cmd.current_dir(dir);
        }

        for (key, value) in env_vars {
            cmd.env(key, value);
        }

        let mut process = cmd.spawn().map_err(DCCError::IoError)?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| DCCError::ConnectionError("Failed to open stdin".to_string()))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| DCCError::ConnectionError("Failed to open stdout".to_string()))?;

        let stdout_reader = BufReader::new(stdout);

        Ok(Self {
            process,
            stdin,
            stdout_reader,
            connected: true,
        })
    }

    /// 获取进程 ID
    pub fn pid(&self) -> Option<u32> {
        Some(self.process.id().unwrap_or(0))
    }

    /// 检查进程是否仍在运行
    pub async fn is_running(&mut self) -> bool {
        match self.process.try_wait() {
            Ok(None) => true,
            _ => false,
        }
    }
}

#[async_trait::async_trait]
impl DCCConnection for StdioConnection {
    async fn send(&mut self, message: &str) -> DCCResult<()> {
        if !self.connected {
            return Err(DCCError::ConnectionError("Not connected".to_string()));
        }

        self.stdin
            .write_all(message.as_bytes())
            .await
            .map_err(DCCError::IoError)?;
        self.stdin
            .write_all(b"\n")
            .await
            .map_err(DCCError::IoError)?;
        self.stdin.flush().await.map_err(DCCError::IoError)?;

        Ok(())
    }

    async fn receive(&mut self) -> DCCResult<Option<String>> {
        if !self.connected {
            return Err(DCCError::ConnectionError("Not connected".to_string()));
        }

        let mut line = String::new();
        match self.stdout_reader.read_line(&mut line).await {
            Ok(0) => Ok(None), // EOF
            Ok(_) => {
                line.pop(); // Remove newline
                Ok(Some(line))
            }
            Err(e) => Err(DCCError::IoError(e)),
        }
    }

    async fn close(&mut self) -> DCCResult<()> {
        if self.connected {
            let _ = self.process.kill().await;
            self.connected = false;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// TCP Socket 连接
pub struct TcpConnection {
    stream: TcpStream,
    connected: bool,
}

impl TcpConnection {
    /// 连接到 TCP 服务器
    pub async fn connect(host: &str, port: u16) -> DCCResult<Self> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr)
            .await
            .map_err(|e| DCCError::ConnectionError(format!("Failed to connect to {}: {}", addr, e)))?;

        Ok(Self {
            stream,
            connected: true,
        })
    }
}

#[async_trait::async_trait]
impl DCCConnection for TcpConnection {
    async fn send(&mut self, message: &str) -> DCCResult<()> {
        if !self.connected {
            return Err(DCCError::ConnectionError("Not connected".to_string()));
        }

        self.stream
            .write_all(message.as_bytes())
            .await
            .map_err(DCCError::IoError)?;
        self.stream.write_all(b"\n").await.map_err(DCCError::IoError)?;

        Ok(())
    }

    async fn receive(&mut self) -> DCCResult<Option<String>> {
        if !self.connected {
            return Err(DCCError::ConnectionError("Not connected".to_string()));
        }

        let mut reader = BufReader::new(&self.stream);
        let mut line = String::new();

        match reader.read_line(&mut line).await {
            Ok(0) => Ok(None),
            Ok(_) => {
                line.pop();
                Ok(Some(line))
            }
            Err(e) => Err(DCCError::IoError(e)),
        }
    }

    async fn close(&mut self) -> DCCResult<()> {
        if self.connected {
            let _ = self.stream.shutdown().await;
            self.connected = false;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// WebSocket 连接
pub struct WebSocketConnection {
    ws: Option<WebSocketStream<MaybeTlsStream<TcpStream>>>,
    connected: bool,
}

impl WebSocketConnection {
    /// 连接到 WebSocket 服务器
    pub async fn connect(url: &str) -> DCCResult<Self> {
        let (ws, _) = connect_async(url)
            .await
            .map_err(|e| DCCError::ConnectionError(format!("WebSocket connection failed: {}", e)))?;

        Ok(Self {
            ws: Some(ws),
            connected: true,
        })
    }
}

#[async_trait::async_trait]
impl DCCConnection for WebSocketConnection {
    async fn send(&mut self, message: &str) -> DCCResult<()> {
        if !self.connected {
            return Err(DCCError::ConnectionError("Not connected".to_string()));
        }

        if let Some(ws) = &mut self.ws {
            use tokio_tungstenite::tungstenite::Message;
            ws.send(Message::Text(message.to_string()))
                .await
                .map_err(|e| DCCError::ConnectionError(format!("WebSocket send failed: {}", e)))?;
        }

        Ok(())
    }

    async fn receive(&mut self) -> DCCResult<Option<String>> {
        if !self.connected {
            return Err(DCCError::ConnectionError("Not connected".to_string()));
        }

        if let Some(ws) = &mut self.ws {
            use futures::StreamExt;
            use tokio_tungstenite::tungstenite::Message;

            match ws.next().await {
                Some(Ok(Message::Text(text))) => Ok(Some(text)),
                Some(Ok(Message::Close(_))) => {
                    self.connected = false;
                    Ok(None)
                }
                Some(Err(e)) => Err(DCCError::ConnectionError(format!("WebSocket receive failed: {}", e))),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn close(&mut self) -> DCCResult<()> {
        if self.connected {
            if let Some(ws) = &mut self.ws {
                use futures::SinkExt;
                use tokio_tungstenite::tungstenite::Message;
                let _ = ws.send(Message::Close(None)).await;
                let _ = ws.close().await;
            }
            self.connected = false;
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }
}

/// 连接工厂
pub struct ConnectionFactory;

impl ConnectionFactory {
    /// 创建标准输入输出连接
    pub async fn create_stdio(
        executable: &std::path::Path,
        args: &[String],
        working_dir: Option<&std::path::Path>,
        env_vars: &[(String, String)],
    ) -> DCCResult<Box<dyn DCCConnection>> {
        let conn = StdioConnection::new(executable, args, working_dir, env_vars).await?;
        Ok(Box::new(conn))
    }

    /// 创建 TCP 连接
    pub async fn create_tcp(host: &str, port: u16) -> DCCResult<Box<dyn DCCConnection>> {
        let conn = TcpConnection::connect(host, port).await?;
        Ok(Box::new(conn))
    }

    /// 创建 WebSocket 连接
    pub async fn create_websocket(url: &str) -> DCCResult<Box<dyn DCCConnection>> {
        let conn = WebSocketConnection::connect(url).await?;
        Ok(Box::new(conn))
    }
}
