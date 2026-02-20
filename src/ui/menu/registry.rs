//! For tabs/items

use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub static TAB_REGISTRY: Lazy<Mutex<Vec<(String, i32)>>> = Lazy::new(|| Mutex::new(Vec::new()));

/// Adds a tab to the menu
pub fn add_tab(name: &str, page_id: i32) {
    TAB_REGISTRY
        .lock()
        .unwrap()
        .push((name.to_string(), page_id));
}

pub type ToggleCallback = Box<dyn Fn(bool) + Send + Sync>;
pub type SliderCallback = Box<dyn Fn(f32) + Send + Sync>;
pub type InputCallback = Box<dyn Fn(String) + Send + Sync>;
pub type ButtonCallback = Box<dyn Fn() + Send + Sync>;
pub type DropdownCallback = Box<dyn Fn(i32) + Send + Sync>;

/// Represents a single item in the menu
#[derive(Clone)]
pub enum MenuItem {
    Toggle {
        id: i32,
        name: String,
        key: String,
        default: bool,
        callback: Option<Arc<ToggleCallback>>,
    },
    Slider {
        id: i32,
        name: String,
        key: String,
        min: f32,
        max: f32,
        default: f32,
        callback: Option<Arc<SliderCallback>>,
    },
    Input {
        id: i32,
        name: String,
        key: String,
        placeholder: String,
        default: String,
        callback: Option<Arc<InputCallback>>,
    },
    Button {
        id: i32,
        name: String,
        target_page: Option<i32>,
        callback: Option<Arc<ButtonCallback>>,
    },
    Label {
        id: i32,
        text: String,
        font_size: f32,
        is_bold: bool,
        color: Option<String>,
    },
    ActionButton {
        id: i32,
        name: String,
        callback: Option<Arc<ButtonCallback>>,
    },
    SectionHeader {
        id: i32,
        title: String,
    },
    Dropdown {
        id: i32,
        name: String,
        key: String,
        options: Vec<String>,
        default: i32,
        callback: Option<Arc<DropdownCallback>>,
    },
}

/// Registry for all menu items and pages
pub struct MenuRegistry {
    /// Map of page IDs to a list of items on that page
    pub pages: HashMap<i32, Vec<MenuItem>>,
    /// Map of page IDs to page titles
    pub page_titles: HashMap<i32, String>,
    /// Lookup map of all items by their unique ID
    pub items_by_id: HashMap<i32, MenuItem>,
    /// Counter for generating unique item IDs
    pub next_id: i32,
}

pub static REGISTRY: Lazy<Mutex<MenuRegistry>> = Lazy::new(|| {
    Mutex::new(MenuRegistry {
        pages: HashMap::new(),
        page_titles: HashMap::new(),
        items_by_id: HashMap::new(),
        next_id: 1000,
    })
});

macro_rules! register_item {
    ($page_id:expr, $item:expr) => {{
        let mut reg = REGISTRY.lock().unwrap();
        let id = reg.next_id;
        reg.next_id += 1;
        let mut item = $item;
        match &mut item {
            MenuItem::Toggle { id: item_id, .. }
            | MenuItem::Slider { id: item_id, .. }
            | MenuItem::Input { id: item_id, .. }
            | MenuItem::Button { id: item_id, .. }
            | MenuItem::Label { id: item_id, .. }
            | MenuItem::ActionButton { id: item_id, .. }
            | MenuItem::SectionHeader { id: item_id, .. }
            | MenuItem::Dropdown { id: item_id, .. } => *item_id = id,
        }
        reg.pages.entry($page_id).or_default().push(item.clone());
        reg.items_by_id.insert(id, item);
    }};
}

/// Adds a toggle switch to the menu
pub fn add_toggle(
    page_id: i32,
    name: &str,
    key: &str,
    default: bool,
    callback: Option<impl Fn(bool) + Send + Sync + 'static>,
) {
    register_item!(
        page_id,
        MenuItem::Toggle {
            id: 0,
            name: name.into(),
            key: key.into(),
            default,
            callback: callback.map(|f| Arc::new(Box::new(f) as ToggleCallback))
        }
    );
}

/// Adds a slider to the menu
pub fn add_slider(
    page_id: i32,
    name: &str,
    key: &str,
    min: f32,
    max: f32,
    default: f32,
    callback: Option<impl Fn(f32) + Send + Sync + 'static>,
) {
    register_item!(
        page_id,
        MenuItem::Slider {
            id: 0,
            name: name.into(),
            key: key.into(),
            min,
            max,
            default,
            callback: callback.map(|f| Arc::new(Box::new(f) as SliderCallback))
        }
    );
}

/// Adds a text input field to the menu
pub fn add_input(
    page_id: i32,
    name: &str,
    key: &str,
    placeholder: &str,
    default: &str,
    callback: Option<impl Fn(String) + Send + Sync + 'static>,
) {
    register_item!(
        page_id,
        MenuItem::Input {
            id: 0,
            name: name.into(),
            key: key.into(),
            placeholder: placeholder.into(),
            default: default.into(),
            callback: callback.map(|f| Arc::new(Box::new(f) as InputCallback))
        }
    );
}

/// Adds a button that navigates to another page
pub fn add_button_with_nav(
    page_id: i32,
    name: &str,
    target_page: i32,
    callback: Option<impl Fn() + Send + Sync + 'static>,
) {
    let mut reg = REGISTRY.lock().unwrap();
    let id = 200 + target_page;
    let item = MenuItem::Button {
        id,
        name: name.into(),
        target_page: Some(target_page),
        callback: callback.map(|f| Arc::new(Box::new(f) as ButtonCallback)),
    };
    reg.pages.entry(page_id).or_default().push(item.clone());
    reg.items_by_id.insert(id, item);
}

/// Adds a general button to the menu
pub fn add_button(page_id: i32, name: &str, callback: Option<impl Fn() + Send + Sync + 'static>) {
    register_item!(
        page_id,
        MenuItem::Button {
            id: 0,
            name: name.into(),
            target_page: None,
            callback: callback.map(|f| Arc::new(Box::new(f) as ButtonCallback))
        }
    );
}

/// Adds a text label to the menu
pub fn add_label(page_id: i32, text: &str, font_size: f32, is_bold: bool, color: Option<&str>) {
    register_item!(
        page_id,
        MenuItem::Label {
            id: 0,
            text: text.into(),
            font_size,
            is_bold,
            color: color.map(|s| s.into())
        }
    );
}

/// Adds an action button (triggers a callback, usually for one-off actions)
pub fn add_action_button(
    page_id: i32,
    name: &str,
    callback: Option<impl Fn() + Send + Sync + 'static>,
) {
    register_item!(
        page_id,
        MenuItem::ActionButton {
            id: 0,
            name: name.into(),
            callback: callback.map(|f| Arc::new(Box::new(f) as ButtonCallback))
        }
    );
}

/// Adds a section header
pub fn add_section_header(page_id: i32, title: &str) {
    register_item!(
        page_id,
        MenuItem::SectionHeader {
            id: 0,
            title: title.into(),
        }
    );
}

/// Adds a dropdown menu to the menu
pub fn add_dropdown(
    page_id: i32,
    name: &str,
    key: &str,
    options: Vec<String>,
    default: i32,
    callback: Option<impl Fn(i32) + Send + Sync + 'static>,
) {
    register_item!(
        page_id,
        MenuItem::Dropdown {
            id: 0,
            name: name.into(),
            key: key.into(),
            options,
            default,
            callback: callback.map(|f| Arc::new(Box::new(f) as DropdownCallback))
        }
    );
}

/// Creates a new menu page and returns its ID
pub fn add_page(name: &str) -> i32 {
    let mut reg = REGISTRY.lock().unwrap();
    let mut page_id = 10;
    while reg.pages.contains_key(&page_id) || page_id < 10 {
        page_id += 1;
    }
    reg.pages.insert(page_id, Vec::new());
    reg.page_titles.insert(page_id, name.to_string());
    let btn_id = reg.next_id;
    reg.next_id += 1;
    let button = MenuItem::Button {
        id: btn_id,
        name: name.to_string(),
        target_page: Some(page_id),
        callback: None,
    };
    reg.pages.entry(0).or_default().push(button.clone());
    reg.items_by_id.insert(btn_id, button);
    page_id
}
