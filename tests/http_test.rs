//! HTTP 请求功能测试

use netdisk_db::controllers::handlers::{send_file_upload_request, UploadFileItemPayload};
use reqwest::Client;

#[tokio::test]
async fn test_send_file_upload_request() {
    // 创建测试数据
    let payload = UploadFileItemPayload {
        parent_file_id: 0,
        filename: "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv".to_string(),
        etag: "e325c611ea19f1bc3bef16f0eac7cb92".to_string(),
        size: 59570941009,
    };

    // 发送请求（注意：这需要本地服务器运行在 127.0.0.1:8080）
    let client = Client::new();
    match send_file_upload_request(&client, payload).await {
        Ok(_) => println!("请求发送成功"),
        Err(e) => println!("请求发送失败: {}", e),
    }
}

#[tokio::test]
async fn test_send_to_aria2_integration() {
    use netdisk_db::controllers::handlers::send_to_aria2;

    // 测试 send_to_aria2 函数
    let result = send_to_aria2(
        "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv",
        "e325c611ea19f1bc3bef16f0eac7cb92",
        59570941009,
    ).await;

    match result {
        Ok(_) => println!("send_to_aria2 执行成功"),
        Err(e) => println!("send_to_aria2 执行失败: {}", e),
    }
}