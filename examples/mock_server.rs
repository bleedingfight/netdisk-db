//! 模拟服务器 - 用于测试 HTTP 请求功能
//! 
//! 这个示例创建一个简单的 HTTP 服务器来接收文件上传请求，
//! 用于验证 send_to_aria2 功能的正确性。

use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UploadFileItemPayload {
    #[serde(alias = "parentFileID")]
    parent_file_id: i64,
    filename: String,
    etag: String,
    size: u64,
}

// 存储接收到的请求数据
struct AppState {
    received_requests: Mutex<Vec<UploadFileItemPayload>>,
}

async fn handle_file_upload(
    data: web::Json<UploadFileItemPayload>,
    state: web::Data<AppState>,
) -> impl Responder {
    println!("收到文件上传请求:");
    println!("  父文件ID: {}", data.parent_file_id);
    println!("  文件名: {}", data.filename);
    println!("  ETag: {}", data.etag);
    println!("  文件大小: {} bytes", data.size);
    
    // 存储接收到的请求
    let mut requests = state.received_requests.lock().unwrap();
    requests.push(data.into_inner());
    
    println!("✅ 请求处理成功");
    
    HttpResponse::Ok().json(serde_json::json!({
        "status": "success",
        "message": "文件上传信息接收成功"
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("启动模拟服务器...");
    println!("服务器将在 http://127.0.0.1:8080 运行");
    println!("POST /file/upload 端点已准备就绪");
    println!();
    println!("你可以运行以下命令来测试:");
    println!("cargo run --example http_request_example");
    println!();
    
    let app_state = web::Data::new(AppState {
        received_requests: Mutex::new(Vec::new()),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone())
            .service(
                web::resource("/file/upload")
                    .route(web::post().to(handle_file_upload))
            )
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}