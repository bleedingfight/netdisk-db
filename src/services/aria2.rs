//! Aria2 服务模块 - 集成Aria2下载器
//!
//! 提供Aria2 RPC客户端功能，用于管理下载任务

use crate::models::config::Aria2Config;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

/// Aria2 RPC 客户端
pub struct Aria2Client {
    config: Aria2Config,
    client: reqwest::Client,
    base_url: String,
}

/// Aria2 RPC 响应结构
#[derive(Debug, Deserialize)]
pub struct Aria2Response {
    pub id: Option<String>,
    pub jsonrpc: String,
    pub result: Option<Value>,
    pub error: Option<Aria2Error>,
}

/// Aria2 错误结构
#[derive(Debug, Deserialize)]
pub struct Aria2Error {
    pub code: i32,
    pub message: String,
}

/// Aria2 下载任务信息
#[derive(Debug, Serialize)]
pub struct DownloadTask {
    pub uris: Vec<String>,
    pub options: DownloadOptions,
}

/// Aria2 下载选项
#[derive(Debug, Serialize)]
pub struct DownloadOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dir: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub out: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub header: Option<Vec<String>>,
}

impl Aria2Client {
    /// 创建新的Aria2客户端
    pub fn new(config: Aria2Config) -> Self {
        let base_url = format!("http://{}:{}", config.rpc_host, config.rpc_port);
        Self {
            config,
            client: reqwest::Client::new(),
            base_url,
        }
    }

    /// 发送RPC请求到Aria2
    async fn send_rpc_request(&self, method: &str, params: Vec<Value>) -> Result<Aria2Response> {
        let mut request_body = json!({
            "jsonrpc": "2.0",
            "id": "netdisk_db",
            "method": method,
            "params": params
        });

        // 如果有RPC密钥，添加到参数中
        if let Some(ref secret) = self.config.rpc_secret {
            if let Some(params_array) = request_body["params"].as_array_mut() {
                params_array.insert(0, Value::String(format!("token:{}", secret)));
            }
        }

        debug!("Sending Aria2 RPC request: {}", request_body);

        let response = self
            .client
            .post(&self.base_url)
            .json(&request_body)
            .send()
            .await
            .context("Failed to send RPC request to Aria2")?;

        let response_text = response.text().await.context("Failed to read response text")?;
        debug!("Aria2 RPC response: {}", response_text);

        let rpc_response: Aria2Response = serde_json::from_str(&response_text)
            .context("Failed to parse Aria2 RPC response")?;

        if let Some(error) = rpc_response.error {
            return Err(anyhow::anyhow!("Aria2 RPC error: {} (code: {})", error.message, error.code));
        }

        Ok(rpc_response)
    }

    /// 检查Aria2服务是否可用
    pub async fn check_connection(&self) -> Result<bool> {
        match self.get_version().await {
            Ok(_) => Ok(true),
            Err(e) => {
                debug!("Aria2 connection check failed: {}", e);
                Ok(false)
            }
        }
    }

    /// 获取Aria2版本信息
    pub async fn get_version(&self) -> Result<String> {
        let response = self.send_rpc_request("aria2.getVersion", vec![]).await?;
        
        if let Some(result) = response.result {
            if let Some(version) = result["version"].as_str() {
                Ok(version.to_string())
            } else {
                Err(anyhow::anyhow!("Version information not found in response"))
            }
        } else {
            Err(anyhow::anyhow!("No result in response"))
        }
    }

    /// 添加下载任务
    pub async fn add_download(&self, url: &str, filename: Option<&str>) -> Result<String> {
        let mut options = DownloadOptions {
            dir: Some(self.config.download_dir.clone()),
            out: filename.map(|f| f.to_string()),
            header: None,
        };

        let task = DownloadTask {
            uris: vec![url.to_string()],
            options,
        };

        let params = vec![
            json!(task.uris),
            json!(task.options),
        ];

        let response = self.send_rpc_request("aria2.addUri", params).await?;
        
        if let Some(result) = response.result {
            if let Some(gid) = result.as_str() {
                info!("Download task added successfully with GID: {}", gid);
                Ok(gid.to_string())
            } else {
                Err(anyhow::anyhow!("GID not found in response"))
            }
        } else {
            Err(anyhow::anyhow!("No result in response"))
        }
    }

    /// 获取下载状态
    pub async fn get_status(&self, gid: &str) -> Result<Value> {
        let response = self.send_rpc_request("aria2.tellStatus", vec![json!(gid)]).await?;
        
        if let Some(result) = response.result {
            Ok(result)
        } else {
            Err(anyhow::anyhow!("No result in response"))
        }
    }
}

/// Aria2 服务管理器
pub struct Aria2Service {
    client: Option<Aria2Client>,
    process: Option<Child>,
    config: Aria2Config,
}

impl Aria2Service {
    /// 创建新的Aria2服务管理器
    pub fn new(config: Aria2Config) -> Self {
        Self {
            client: None,
            process: None,
            config,
        }
    }

    /// 启动Aria2服务
    pub fn start(&mut self) -> Result<()> {
        if !self.config.enabled {
            info!("Aria2 service is disabled in configuration");
            return Ok(());
        }

        // 检查是否已安装aria2c
        if !Self::check_aria2_installed() {
            warn!("Aria2 is not installed or not in PATH. Please install aria2 to enable download functionality.");
            return Ok(());
        }

        info!("Starting Aria2 service...");

        // 创建下载目录
        if let Err(e) = std::fs::create_dir_all(&self.config.download_dir) {
            warn!("Failed to create download directory: {}, using current directory", e);
            self.config.download_dir = ".".to_string();
        }

        // 启动aria2c进程
        let mut command = Command::new("aria2c");
        
        command
            .arg("--enable-rpc")
            .arg("--rpc-listen-all=false")
            .arg("--rpc-listen-port")
            .arg(self.config.rpc_port.to_string())
            .arg("--rpc-allow-origin-all")
            .arg("--file-allocation=none")
            .arg("--max-connection-per-server=16")
            .arg("--split=16")
            .arg("--min-split-size=1M")
            .arg("--continue=true")
            .arg("--dir")
            .arg(&self.config.download_dir)
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        // 如果有RPC密钥，添加认证
        if let Some(ref secret) = self.config.rpc_secret {
            command.arg("--rpc-secret").arg(secret);
        }

        let process = command.spawn().context("Failed to start aria2c process")?;
        
        self.process = Some(process);
        self.client = Some(Aria2Client::new(self.config.clone()));

        info!("Aria2 service started on {}:{}", self.config.rpc_host, self.config.rpc_port);
        
        Ok(())
    }

    /// 停止Aria2服务
    pub fn stop(&mut self) -> Result<()> {
        if let Some(mut process) = self.process.take() {
            info!("Stopping Aria2 service...");
            if let Err(e) = process.kill() {
                warn!("Failed to kill Aria2 process: {}", e);
            }
            let _ = process.wait();
            info!("Aria2 service stopped");
        }
        self.client = None;
        Ok(())
    }

    /// 检查Aria2是否已安装
    pub fn check_aria2_installed() -> bool {
        Command::new("aria2c")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// 获取Aria2客户端
    pub fn get_client(&self) -> Option<&Aria2Client> {
        self.client.as_ref()
    }

    /// 等待Aria2服务就绪
    pub async fn wait_until_ready(&self, timeout_secs: u64) -> bool {
        if self.client.is_none() {
            return false;
        }

        let client = self.client.as_ref().unwrap();
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        while start_time.elapsed() < timeout {
            if client.check_connection().await.unwrap_or(false) {
                info!("Aria2 service is ready");
                return true;
            }
            sleep(Duration::from_millis(500)).await;
        }

        warn!("Aria2 service did not become ready within {} seconds", timeout_secs);
        false
    }
}

impl Drop for Aria2Service {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

/// 全局Aria2服务实例
pub type SharedAria2Service = Arc<Mutex<Aria2Service>>;

/// 创建共享的Aria2服务实例
pub fn create_shared_aria2_service(config: Aria2Config) -> SharedAria2Service {
    Arc::new(Mutex::new(Aria2Service::new(config)))
}