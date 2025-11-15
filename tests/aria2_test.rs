//! Aria2 服务功能测试

use netdisk_db::models::config::Aria2Config;
use netdisk_db::services::aria2::Aria2Service;

#[tokio::test]
async fn test_aria2_service_creation() {
    // 测试Aria2配置创建
    let config = Aria2Config {
        enabled: true,
        rpc_host: "127.0.0.1".to_string(),
        rpc_port: 6800,
        rpc_secret: None,
        download_dir: "./test_downloads".to_string(),
    };

    let mut service = Aria2Service::new(config);
    
    // 测试服务启动（如果aria2已安装）
    if Aria2Service::check_aria2_installed() {
        match service.start() {
            Ok(_) => {
                println!("Aria2 service started successfully");
                
                // 测试连接
                if let Some(client) = service.get_client() {
                    let is_connected = client.check_connection().await.unwrap_or(false);
                    println!("Aria2 connection status: {}", is_connected);
                    
                    if is_connected {
                        // 测试获取版本信息
                        match client.get_version().await {
                            Ok(version) => {
                                println!("Aria2 version: {}", version);
                            }
                            Err(e) => {
                                println!("Failed to get Aria2 version: {}", e);
                            }
                        }
                    }
                }
                
                // 停止服务
                let _ = service.stop();
            }
            Err(e) => {
                println!("Failed to start Aria2 service: {}", e);
            }
        }
    } else {
        println!("Aria2 is not installed, skipping service tests");
    }
}

#[test]
fn test_aria2_config_serialization() {
    use serde_json;
    
    let config = Aria2Config {
        enabled: true,
        rpc_host: "127.0.0.1".to_string(),
        rpc_port: 6800,
        rpc_secret: Some("secret123".to_string()),
        download_dir: "./downloads".to_string(),
    };
    
    // 测试序列化
    let serialized = serde_json::to_string(&config).unwrap();
    println!("Serialized config: {}", serialized);
    
    // 测试反序列化
    let deserialized: Aria2Config = serde_json::from_str(&serialized).unwrap();
    assert_eq!(deserialized.enabled, config.enabled);
    assert_eq!(deserialized.rpc_host, config.rpc_host);
    assert_eq!(deserialized.rpc_port, config.rpc_port);
    assert_eq!(deserialized.download_dir, config.download_dir);
    
    println!("Aria2 config serialization test passed");
}