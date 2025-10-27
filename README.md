# 文件搜索工具

一个使用 Rust + Slint 实现的文件搜索应用程序，支持通过 SQLite 数据库进行文件搜索。

## 功能特点

- **图形用户界面**: 使用 Slint 框架构建的现代 GUI
- **数据库搜索**: 基于 SQLite 数据库的高效文件搜索
- **可扩展架构**: 支持未来添加其他数据库（MySQL、PostgreSQL 等）
- **实时搜索**: 输入搜索内容后实时显示结果
- **文件信息展示**: 显示文件名、路径、大小、修改时间等详细信息

## 项目结构

```
file_search_app/
├── Cargo.toml          # Rust 项目配置
├── build.rs            # 构建脚本
├── config.json         # 配置文件（自动生成）
├── file_search.db      # SQLite 数据库（自动生成）
├── src/
│   ├── main.rs         # 主程序入口
│   ├── config.rs       # 配置管理
│   ├── ui/
│   │   └── mod.rs      # UI 界面定义
│   └── database/
│       ├── mod.rs      # 数据库抽象接口
│       └── sqlite.rs   # SQLite 实现
└── ui/
    └── app_window.slint # Slint UI 文件（已移除，使用 Rust 内嵌）
```

## 技术栈

- **Rust**: 系统编程语言
- **Slint**: 跨平台 GUI 框架
- **SQLite**: 轻量级数据库
- **r2d2**: 数据库连接池
- **rusqlite**: SQLite Rust 驱动

## 快速开始

### 1. 安装依赖

确保已安装 Rust 工具链：
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 2. 克隆和运行

```bash
cd file_search_app
cargo run
```

### 3. 使用应用

1. 程序启动后会自动创建配置文件和数据库
2. 在搜索框中输入文件名或路径关键词
3. 点击"搜索"按钮或按回车键进行搜索
4. 搜索结果会显示在下方列表中

## 配置说明

应用会自动创建 `config.json` 配置文件：

```json
{
  "database": {
    "db_type": "sqlite",
    "connection_string": "file_search.db"
  },
  "window_width": 800,
  "window_height": 600
}
```

## 数据库架构

SQLite 数据库包含以下表结构：

```sql
CREATE TABLE files (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    path TEXT NOT NULL,
    size INTEGER NOT NULL,
    modified_time TEXT NOT NULL,
    file_type TEXT NOT NULL
);

CREATE INDEX idx_files_name ON files(name);
CREATE INDEX idx_files_path ON files(path);
```

## 扩展开发

### 添加新的数据库支持

1. 在 `src/database/` 目录下创建新的实现文件（如 `mysql.rs`）
2. 实现 `Database` trait
3. 在 `src/main.rs` 中添加对应的数据库类型匹配

示例：
```rust
// src/database/mysql.rs
use crate::database::{Database, FileRecord};

pub struct MySqlDatabase {
    // MySQL 连接配置
}

impl Database for MySqlDatabase {
    fn search_files(&self, query: &str) -> Result<Vec<FileRecord>> {
        // MySQL 搜索实现
    }
    
    fn init_database(&self) -> Result<()> {
        // MySQL 初始化实现
    }
}
```

### 修改 UI 界面

UI 界面定义在 `src/ui/mod.rs` 文件中，使用 Slint 的 `slint!` 宏内嵌定义。可以修改：

- 窗口大小和标题
- 搜索框样式
- 结果列表布局
- 文件信息显示方式

## 注意事项

- 首次运行时会自动创建示例数据
- 搜索支持模糊匹配（LIKE 查询）
- 结果限制为最多 100 条记录
- 程序使用同步方式处理数据库操作，避免线程安全问题

## 许可证

MIT License