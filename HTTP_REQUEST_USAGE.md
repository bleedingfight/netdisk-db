# HTTP 请求功能使用说明

## 功能概述

本项目已经实现了通过 Rust 的 reqwest 库发送 HTTP POST 请求的功能，可以将文件信息发送到指定的服务器端点。

## 主要函数

### `send_file_upload_request`

发送文件上传请求到服务器。

```rust
pub async fn send_file_upload_request(data: UploadFileItemPayload) -> Result<(), Box<dyn std::error::Error>>
```

**参数：**
- `data` - 文件上传数据，包含以下字段：
  - `parent_file_id: i64` - 父文件ID
  - `filename: String` - 文件名
  - `etag: String` - 文件ETag
  - `size: u64` - 文件大小

**返回值：**
- `Ok(())` - 请求发送成功
- `Err(Box<dyn std::error::Error>)` - 请求发送失败

### `send_to_aria2`

将文件信息发送到 Aria2 处理函数，内部调用 `send_file_upload_request`。

```rust
pub async fn send_to_aria2<T>(path: T, etag: T, size: u64) -> Result<(), Box<dyn std::error::Error>>
where
    T: AsRef<str> + std::fmt::Debug,
```

**参数：**
- `path` - 文件路径或名称
- `etag` - 文件ETag
- `size` - 文件大小

## 使用示例

### 基本用法

```rust
use netdisk_db::controllers::handlers::send_to_aria2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 发送文件信息
    send_to_aria2(
        "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv",
        "e325c611ea19f1bc3bef16f0eac7cb92",
        59570941009,
    ).await?;
    
    Ok(())
}
```

### 等效的 curl 命令

你提供的 curl 命令：

```bash
curl -X POST -H 'Content-Type: application/json' -d '{
    "parentFileID": 0,
    "filename": "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv",
    "etag": "e325c611ea19f1bc3bef16f0eac7cb92",
    "size": 59570941009
}' http://127.0.0.1:8080/file/upload
```

等效的 Rust 代码：

```rust
use netdisk_db::controllers::handlers::{send_file_upload_request, UploadFileItemPayload};

let payload = UploadFileItemPayload {
    parent_file_id: 0,
    filename: "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv".to_string(),
    etag: "e325c611ea19f1bc3bef16f0eac7cb92".to_string(),
    size: 59570941009,
};

send_file_upload_request(payload).await?;
```

## 运行示例

### 1. 启动模拟服务器（用于测试）

```bash
cargo run --example mock_server
```

### 2. 运行 HTTP 请求示例

在另一个终端中运行：

```bash
cargo run --example http_request_example
```

## 测试

运行单元测试：

```bash
cargo test test_send_file_upload_request -- --nocapture
cargo test test_send_to_aria2_integration -- --nocapture
```

## 错误处理

函数会返回详细的错误信息，包括：
- 网络连接错误
- HTTP 状态码错误
- 服务器返回的错误信息

## 依赖项

本项目使用以下依赖项：
- `reqwest = "0.12"` - HTTP 客户端
- `serde = { version = "1.0", features = ["derive"] }` - 序列化/反序列化
- `tokio = { version = "1", features = ["full"] }` - 异步运行时

## 注意事项

1. 确保目标服务器在 `http://127.0.0.1:8080/file/upload` 上运行
2. 函数是异步的，需要在异步上下文中调用
3. 错误处理已经内置，会返回详细的错误信息
4. 请求头会自动设置为 `Content-Type: application/json`