//! HTTP 请求功能使用示例
//! 
//! 这个示例展示了如何使用 send_to_aria2 函数发送 HTTP POST 请求。
//! 运行前请确保本地服务器在 127.0.0.1:8080 上运行。

use netdisk_db::controllers::handlers::send_to_aria2;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("HTTP 请求功能示例");
    println!("==================");
    
    // 示例数据，对应 curl 命令中的参数
    let filename = "Skyfall.2012.2160p.BluRay.REMUX.HEVC.DTS-HD.MA.5.1-FGT.mkv";
    let etag = "e325c611ea19f1bc3bef16f0eac7cb92";
    let size = 59570941009u64;
    
    println!("准备发送文件上传请求:");
    println!("文件名: {}", filename);
    println!("ETag: {}", etag);
    println!("文件大小: {} bytes", size);
    
    // 调用 send_to_aria2 函数
    match send_to_aria2(filename, etag, size).await {
        Ok(_) => {
            println!("✅ 文件上传请求发送成功！");
        }
        Err(e) => {
            println!("❌ 文件上传请求发送失败: {}", e);
            println!("请确保本地服务器在 127.0.0.1:8080 上运行");
        }
    }
    
    Ok(())
}