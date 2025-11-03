
use anyhow::Result;
use std::collections::HashMap;

/// 菜单项类型枚举
#[derive(Debug, Clone, PartialEq)]
pub enum MenuItemType {
    Normal,
    Separator,
    Submenu,
}

/// 菜单项结构
#[derive(Debug, Clone)]
pub struct MenuItem {
    pub id: String,
    pub text: String,
    pub action: String,
    pub item_type: MenuItemType,
    pub enabled: bool,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub category: Option<String>, // 用于分组
    pub color: Option<String>,    // 用于视觉区分
}

/// 上下文菜单管理器
pub struct ContextMenuManager {
    items: Vec<MenuItem>,
    handlers: HashMap<String, Box<dyn Fn(&str) -> Result<()> + Send + Sync>>,
}

impl ContextMenuManager {
    pub fn new() -> Self {
        let mut manager = Self {
            items: Vec::new(),
            handlers: HashMap::new(),
        };
        
        // 添加默认菜单项
        manager.add_default_items();
        manager
    }

    fn add_default_items(&mut self) {
        // 文件传输组 - 蓝色主题
        self.add_item(MenuItem {
            id: "download".to_string(),
            text: "Download File".to_string(),
            action: "download".to_string(),
            item_type: MenuItemType::Normal,
            enabled: true,
            icon: Some("⬇".to_string()),
            shortcut: None,
            category: Some("transfer".to_string()),
            color: Some("#007acc".to_string()),
        });

        self.add_item(MenuItem {
            id: "send_to_server".to_string(),
            text: "Send to Server".to_string(),
            action: "send_to_server".to_string(),
            item_type: MenuItemType::Normal,
            enabled: true,
            icon: Some("📤".to_string()),
            shortcut: None,
            category: Some("transfer".to_string()),
            color: Some("#007acc".to_string()),
        });

        // 分隔符
        self.add_separator();

        // 文件操作组 - 绿色主题
        self.add_item(MenuItem {
            id: "update_content".to_string(),
            text: "Update Content".to_string(),
            action: "update_content".to_string(),
            item_type: MenuItemType::Normal,
            enabled: true,
            icon: Some("✏".to_string()),
            shortcut: None,
            category: Some("operation".to_string()),
            color: Some("#28a745".to_string()),
        });

        self.add_item(MenuItem {
            id: "open_location".to_string(),
            text: "Open Location".to_string(),
            action: "open_location".to_string(),
            item_type: MenuItemType::Normal,
            enabled: true,
            icon: Some("📁".to_string()),
            shortcut: None,
            category: Some("operation".to_string()),
            color: Some("#28a745".to_string()),
        });
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn add_separator(&mut self) {
        let separator_count = self.items.iter()
            .filter(|item| item.item_type == MenuItemType::Separator)
            .count();
        
        self.add_item(MenuItem {
            id: format!("separator{}", separator_count + 1),
            text: "-".to_string(),
            action: "".to_string(),
            item_type: MenuItemType::Separator,
            enabled: false,
            icon: None,
            shortcut: None,
            category: None,
            color: None,
        });
    }

    pub fn register_handler<F>(&mut self, action: &str, handler: F)
    where
        F: Fn(&str) -> Result<()> + Send + Sync + 'static,
    {
        self.handlers.insert(action.to_string(), Box::new(handler));
    }

    pub fn get_items(&self) -> &[MenuItem] {
        &self.items
    }

    pub fn get_enabled_items(&self) -> Vec<&MenuItem> {
        self.items.iter()
            .filter(|item| item.enabled && item.item_type == MenuItemType::Normal)
            .collect()
    }

    pub fn execute_action(&self, action: &str, file_path: &str) -> Result<()> {
        if let Some(handler) = self.handlers.get(action) {
            handler(file_path)
        } else {
            anyhow::bail!("No handler registered for action: {}", action)
        }
    }

    pub fn enable_item(&mut self, id: &str, enabled: bool) {
        if let Some(item) = self.items.iter_mut().find(|i| i.id == id) {
            item.enabled = enabled;
        }
    }

    pub fn add_aria2_support(&mut self) {
        // 添加Aria2支持作为示例扩展功能
        self.add_separator();
        
        self.add_item(MenuItem {
            id: "send_to_aria2".to_string(),
            text: "Send to Aria2".to_string(),
            action: "send_to_aria2".to_string(),
            item_type: MenuItemType::Normal,
            enabled: true,
            icon: Some("🚀".to_string()),
            shortcut: Some("Ctrl+A".to_string()),
            category: Some("advanced".to_string()),
            color: Some("#ff6b35".to_string()),
        });
    }

    pub fn create_slint_menu_items(&self) -> Vec<slint::SharedString> {
        self.items.iter()
            .filter(|item| item.enabled)
            .map(|item| {
                match item.item_type {
                    MenuItemType::Normal => {
                        let display_text = if let (Some(icon), Some(shortcut)) = (&item.icon, &item.shortcut) {
                            format!("{} {}  {}", icon, item.text, shortcut)
                        } else if let Some(icon) = &item.icon {
                            format!("{} {}", icon, item.text)
                        } else if let Some(shortcut) = &item.shortcut {
                            format!("{}  {}", item.text, shortcut)
                        } else {
                            item.text.clone()
                        };
                        slint::SharedString::from(display_text)
                    },
                    MenuItemType::Separator => {
                        slint::SharedString::from("──────────────────────────────") // 分隔线
                    },
                    MenuItemType::Submenu => {
                        slint::SharedString::from(format!("{} ▶", item.text))
                    },
                }
            })
            .collect()
    }

    pub fn get_item_by_index(&self, index: usize) -> Option<&MenuItem> {
        let enabled_items: Vec<&MenuItem> = self.items.iter()
            .filter(|item| item.enabled)
            .collect();
        
        enabled_items.get(index).copied()
    }

    pub fn get_slint_struct_items(&self) -> Vec<crate::ui::ContextMenuItem> {
        self.items.iter()
            .filter(|item| item.enabled && item.item_type == MenuItemType::Normal)
            .map(|item| crate::ui::ContextMenuItem {
                text: item.text.clone().into(),
                action: item.action.clone().into(),
                color: item.color.clone().unwrap_or_default().into(),
                icon: item.icon.clone().unwrap_or_default().into(),
                category: item.category.clone().unwrap_or_default().into(),
                shortcut: item.shortcut.clone().unwrap_or_default().into(),
            })
            .collect()
    }
}

impl Default for ContextMenuManager {
    fn default() -> Self {
        Self::new()
    }
}
