# 文件搜索工具

一个使用 Rust + Slint 实现的文件搜索应用程序，支持多种数据库类型（SQLite、MySQL）进行文件搜索。

## 功能特点

- **图形用户界面**: 使用 Slint 框架构建的现代 GUI
- **多数据库支持**: 支持 SQLite 和 MySQL 数据库
- **动态数据库发现**: 自动扫描并发现可用的数据库
- **下拉列表选择**: 通过下拉列表选择数据库，替代原有的按钮切换
- **实时搜索**: 输入搜索内容后实时显示结果
- **文件信息展示**: 显示文件名、路径、大小、修改时间等详细信息
- **数据库刷新**: 支持手动刷新数据库列表

## 项目结构

```
file_search_app/
├── Cargo.toml          # Rust 项目配置
├── build.rs            # 构建脚本
├── config.json         # 配置文件（自动生成）
├── file_search.db      # SQLite 数据库（自动生成）
├── src/
│   ├── main.rs         # 主程序入口
│   ├── lib.rs          # 库入口
│   ├── prelude.rs      # 统一导入模块
│   ├── models/         # 数据模型
│   │   ├── config.rs   # 配置模型
│   │   └── database.rs # 数据库模型
│   ├── views/          # 视图层
│   │   └── ui.rs       # UI 数据转换
│   ├── controllers/    # 控制器层
│   │   └── handlers.rs # 事件处理
│   ├── services/       # 服务层
│   │   ├── database_manager.rs # 数据库管理器
│   │   └── database/   # 数据库服务
│   │       ├── mod.rs  # 数据库模块
│   │       ├── connector.rs # 数据库连接器抽象
│   │       └── sqlite.rs # SQLite 实现
│   └── utils/          # 工具函数
│       └── common.rs   # 通用工具
└── ui/
    └── app_window.slint # Slint UI 文件
```

## 技术栈

- **Rust**: 系统编程语言
- **Slint**: 跨平台 GUI 框架
- **SQLite**: 轻量级数据库
- **MySQL**: 关系型数据库（支持连接）
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

1. 程序启动后会自动扫描当前目录下的数据库文件
2. 使用下拉列表选择要搜索的数据库
3. 点击刷新按钮可以重新扫描数据库
4. 在搜索框中输入文件名或路径关键词
5. 搜索结果会实时显示在下方列表中

## 配置说明

应用会自动创建 `config.json` 配置文件，支持多数据库配置：

```json
{
  "database": {
    "db_type": "sqlite",
    "connection_string": "file_search.db",
    "name": "Default Database",
    "description": "Main file search database"
  },
  "multi_database": {
    "databases": [
      {
        "db_type": "sqlite",
        "connection_string": "file_search.db",
        "name": "File Search DB",
        "description": "Main file search database"
      },
      {
        "db_type": "mysql",
        "connection_string": "mysql://user:pass@localhost:3306/files",
        "name": "MySQL Files",
        "description": "MySQL file database"
      }
    ],
    "default_database": 0
  },
  "window_width": 800,
  "window_height": 600
}
```

## 数据库架构

### SQLite 数据库结构
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

### MySQL 数据库支持
支持连接 MySQL 数据库，会自动发现服务器上的数据库列表。

## 核心功能实现

### 数据库连接器架构
- **抽象接口**: `DatabaseConnector` trait 提供统一的数据库发现接口
- **类型支持**: 支持 SQLite（文件扫描）和 MySQL（服务器连接）
- **动态发现**: 根据数据库类型使用不同的发现策略

### UI 改进
- **下拉列表**: 替代原有的按钮切换方式
- **字符串模型**: 使用字符串数组供 ComboBox 使用
- **刷新功能**: 支持手动刷新数据库列表

## 扩展开发

### 添加新的数据库支持

1. 在 `src/services/database/` 目录下创建新的连接器实现
2. 实现 `DatabaseConnector` trait
3. 在连接器工厂中注册新的数据库类型

示例：
```rust
// src/services/database/postgres.rs
pub struct PostgresConnector;

impl DatabaseConnector for PostgresConnector {
    fn get_db_type(&self) -> &str {
        "postgres"
    }
    
    fn get_database_list(&self, connection_info: &HashMap<String, String>) -> Result<Vec<DatabaseConnectionInfo>> {
        // PostgreSQL 数据库发现实现
    }
}
```

### 修改 UI 界面

UI 界面定义在 `ui/app_window.slint` 文件中，可以修改：
- 窗口大小和标题
- 搜索框样式
- 下拉列表样式
- 结果列表布局
- 文件信息显示方式

## 注意事项

- 首次运行时会自动创建示例数据
- 搜索支持模糊匹配（LIKE 查询）
- 结果限制为最多 100 条记录
- SQLite 数据库会自动扫描当前目录下的 `.db` 文件
- MySQL 数据库需要正确的连接字符串格式

## 许可证

MIT License