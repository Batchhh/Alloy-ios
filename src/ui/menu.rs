use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, sel, ClassType, MainThreadOnly};
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use objc2_ui_kit::{UIButton, UIControlEvents, UIControlState, UIGestureRecognizerState, UIPanGestureRecognizer, UISlider, UITextField, UIView, UITapGestureRecognizer};
use super::components::{
    create_button_item, create_label, create_toggle_item, create_slider_item, 
    create_text_input_item, create_header, UIBlurEffect, UIVisualEffectView
};
use super::theme::Theme;
use crate::utils::logger;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;

type ToggleCallback = Box<dyn Fn(bool) + Send + Sync>;
type SliderCallback = Box<dyn Fn(f32) + Send + Sync>;
type InputCallback = Box<dyn Fn(String) + Send + Sync>;
type ButtonCallback = Box<dyn Fn() + Send + Sync>;

#[derive(Clone)]
pub enum MenuItem {
    Toggle { id: i32, name: String, key: String, default: bool, callback: Option<Arc<ToggleCallback>> },
    Slider { id: i32, name: String, key: String, min: f32, max: f32, default: f32, callback: Option<Arc<SliderCallback>> },
    Input { id: i32, name: String, key: String, placeholder: String, default: String, callback: Option<Arc<InputCallback>> },
    Button { id: i32, name: String, target_page: Option<i32>, callback: Option<Arc<ButtonCallback>> },
    Label { id: i32, text: String, font_size: f32, is_bold: bool, color: Option<String> },
    ActionButton { id: i32, name: String, callback: Option<Arc<ButtonCallback>> },
}

struct MenuRegistry {
    pages: HashMap<i32, Vec<MenuItem>>,
    page_titles: HashMap<i32, String>,
    items_by_id: HashMap<i32, MenuItem>,
    next_id: i32,
}

static REGISTRY: Lazy<Mutex<MenuRegistry>> = Lazy::new(|| {
    Mutex::new(MenuRegistry { pages: HashMap::new(), page_titles: HashMap::new(), items_by_id: HashMap::new(), next_id: 1000 })
});

macro_rules! register_item {
    ($page_id:expr, $item:expr) => {{
        let mut reg = REGISTRY.lock().unwrap();
        let id = reg.next_id;
        reg.next_id += 1;
        let mut item = $item;
        match &mut item {
            MenuItem::Toggle { id: item_id, .. } | MenuItem::Slider { id: item_id, .. } |
            MenuItem::Input { id: item_id, .. } | MenuItem::Button { id: item_id, .. } |
            MenuItem::Label { id: item_id, .. } | MenuItem::ActionButton { id: item_id, .. } => *item_id = id,
        }
        reg.pages.entry($page_id).or_default().push(item.clone());
        reg.items_by_id.insert(id, item);
    }};
}

pub fn add_toggle(page_id: i32, name: &str, key: &str, default: bool, callback: Option<impl Fn(bool) + Send + Sync + 'static>) {
    register_item!(page_id, MenuItem::Toggle { id: 0, name: name.into(), key: key.into(), default, callback: callback.map(|f| Arc::new(Box::new(f) as ToggleCallback)) });
}

pub fn add_slider(page_id: i32, name: &str, key: &str, min: f32, max: f32, default: f32, callback: Option<impl Fn(f32) + Send + Sync + 'static>) {
    register_item!(page_id, MenuItem::Slider { id: 0, name: name.into(), key: key.into(), min, max, default, callback: callback.map(|f| Arc::new(Box::new(f) as SliderCallback)) });
}

pub fn add_input(page_id: i32, name: &str, key: &str, placeholder: &str, default: &str, callback: Option<impl Fn(String) + Send + Sync + 'static>) {
    register_item!(page_id, MenuItem::Input { id: 0, name: name.into(), key: key.into(), placeholder: placeholder.into(), default: default.into(), callback: callback.map(|f| Arc::new(Box::new(f) as InputCallback)) });
}

pub fn add_button_with_nav(page_id: i32, name: &str, target_page: i32, callback: Option<impl Fn() + Send + Sync + 'static>) {
    let mut reg = REGISTRY.lock().unwrap();
    let id = 200 + target_page;
    let item = MenuItem::Button { id, name: name.into(), target_page: Some(target_page), callback: callback.map(|f| Arc::new(Box::new(f) as ButtonCallback)) };
    reg.pages.entry(page_id).or_default().push(item.clone());
    reg.items_by_id.insert(id, item);
}


pub fn add_button(page_id: i32, name: &str, callback: Option<impl Fn() + Send + Sync + 'static>) {
    register_item!(page_id, MenuItem::Button { id: 0, name: name.into(), target_page: None, callback: callback.map(|f| Arc::new(Box::new(f) as ButtonCallback)) });
}

pub fn add_label(page_id: i32, text: &str, font_size: f32, is_bold: bool, color: Option<&str>) {
    register_item!(page_id, MenuItem::Label { id: 0, text: text.into(), font_size, is_bold, color: color.map(|s| s.into()) });
}

pub fn add_action_button(page_id: i32, name: &str, callback: Option<impl Fn() + Send + Sync + 'static>) {
    register_item!(page_id, MenuItem::ActionButton { id: 0, name: name.into(), callback: callback.map(|f| Arc::new(Box::new(f) as ButtonCallback)) });
}


pub fn add_page(name: &str) -> i32 {
    let mut reg = REGISTRY.lock().unwrap();
    let mut page_id = 10;
    while reg.pages.contains_key(&page_id) || page_id < 10 { page_id += 1; }
    reg.pages.insert(page_id, Vec::new());
    reg.page_titles.insert(page_id, name.to_string());
    let btn_id = reg.next_id;
    reg.next_id += 1;
    let button = MenuItem::Button { id: btn_id, name: name.to_string(), target_page: Some(page_id), callback: None };
    reg.pages.entry(0).or_default().push(button.clone());
    reg.items_by_id.insert(btn_id, button);
    page_id
}





fn update_toggle_ui(sender: &UIButton, selected: bool) {
    if let Some(bg) = sender.viewWithTag(2) {
        let color = if selected { Theme::accent() } else { Theme::toggle_off() };
        bg.setBackgroundColor(Some(&color));
        if let Some(knob) = bg.viewWithTag(3) {
            let mut frame = knob.frame();
            frame.origin.x = if selected { 22.0 } else { 2.0 };
            knob.setFrame(frame);
            let knob_color = if selected { Theme::knob_on() } else { Theme::accent() };
            knob.setBackgroundColor(Some(&knob_color));
        }
    }
}

thread_local! {
    static ACTION_HANDLER: std::cell::RefCell<Option<Retained<MenuActionHandler>>> = const { std::cell::RefCell::new(None) };
    static SCROLL_VIEW: std::cell::RefCell<Option<Retained<UIView>>> = const { std::cell::RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "RustMenuActionHandler"]
    pub struct MenuActionHandler;

impl MenuActionHandler {
        #[unsafe(method(handleAction:))]
        fn handle_action(&self, sender: &UIButton) {
            let tag = sender.tag();
            let registry = REGISTRY.lock().unwrap();
            if let Some(item) = registry.items_by_id.get(&(tag as i32)).cloned() {
                drop(registry);
                match item {
                    MenuItem::Button{ callback, target_page, .. } => {
                        if let Some(cb) = callback { cb(); }
                        if let Some(page_id) = target_page { render_content(page_id); }
                        return;
                    }
                    MenuItem::Toggle { callback, key, .. } => {
                         let selected = !sender.isSelected();
                         sender.setSelected(selected);
                         super::pref::Preferences::set_bool(&key, selected);
                         update_toggle_ui(sender, selected);
                         if let Some(cb) = callback { cb(selected); }
                         return;
                    }
                    MenuItem::ActionButton { callback, .. } => {
                        sender.setAlpha(0.6);
                        unsafe {
                            let _: () = msg_send![self, performSelector: sel!(resetButtonAlpha:), withObject: sender, afterDelay: 0.15f64];
                        }
                        if let Some(cb) = callback { cb(); }
                        return;
                    }
                    _ => {},
                }
            } else { drop(registry); }
            if tag == 99 { crate::ui::window::hide_menu(); }
            else if tag >= 200 && tag < 300 { render_content((tag - 200) as i32); }
        }
        
        #[unsafe(method(handleSlider:))]
        fn handle_slider(&self, sender: &UISlider) {
             let (value, tag) = (sender.value(), sender.tag());
             unsafe {
                 let container: Option<Retained<UIView>> = msg_send![sender, superview];
                 if let Some(cont) = container {
                     if let Some(label) = cont.viewWithTag(5) {
                         let _: () = msg_send![&label, setText: &*NSString::from_str(&format!("{:.2}", value))];
                     }
                 }
             }
             if let Some(MenuItem::Slider { callback, key, .. }) = REGISTRY.lock().unwrap().items_by_id.get(&(tag as i32)) {
                 super::pref::Preferences::set_float(key, value);
                 if let Some(cb) = callback { cb(value); }
             }
        }
        
        #[unsafe(method(handleTextChange:))]
        fn handle_text_change(&self, sender: &UITextField) {
             let (tag, text) = (sender.tag(), sender.text().map(|t| t.to_string()).unwrap_or_default());
             if let Some(MenuItem::Input { callback, key, .. }) = REGISTRY.lock().unwrap().items_by_id.get(&(tag as i32)) {
                 super::pref::Preferences::set_string(key, &text);
                 if let Some(cb) = callback { cb(text); }
             }
        }

        #[unsafe(method(handleTap:))]
        fn handle_tap(&self, recognizer: &UITapGestureRecognizer) {
            if let Some(view) = recognizer.view() { view.endEditing(true); }
        }

        #[unsafe(method(handlePan:))]
        fn handle_pan(&self, recognizer: &UIPanGestureRecognizer) {
            let state = recognizer.state();
            if state == UIGestureRecognizerState::Changed || state == UIGestureRecognizerState::Ended {
                 if let Some(view) = recognizer.view() {
                    let translation = recognizer.translationInView(Some(&view));
                    let mut center = view.center();
                    center.x += translation.x;
                    center.y += translation.y;

                    if let Some(superview) = view.superview() {
                        let super_bounds = superview.bounds();
                        let view_bounds = view.bounds();
                        let half_w = view_bounds.size.width * 0.5;
                        let half_h = view_bounds.size.height * 0.5;

                        if center.x < half_w { center.x = half_w; }
                        if center.x > super_bounds.size.width - half_w { center.x = super_bounds.size.width - half_w; }
                        
                        if center.y < half_h { center.y = half_h; }
                        if center.y > super_bounds.size.height - half_h { center.y = super_bounds.size.height - half_h; }
                    }

                    view.setCenter(center);
                    recognizer.setTranslation_inView(CGPoint::new(0.0, 0.0), Some(&view));
                 }
            }
        }

        #[unsafe(method(resetButtonAlpha:))]
        fn reset_button_alpha(&self, sender: &UIButton) {
            sender.setAlpha(1.0);
        }
    }
);

impl MenuActionHandler {
    fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![MenuActionHandler::class(), new] }
    }
}

pub fn create_menu_view(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIView> {
    let menu: Retained<UIView> = UIView::initWithFrame(UIView::alloc(mtm), frame);
    menu.setBackgroundColor(Some(&Theme::background()));
    let blur_effect = unsafe {
        let effect: Retained<UIBlurEffect> = msg_send![UIBlurEffect::class(), effectWithStyle: 2i64];
        effect
    };
    let effect_view = UIVisualEffectView::new(&blur_effect, mtm);
    effect_view.setFrame(menu.bounds());
    effect_view.setAutoresizingMask(objc2_ui_kit::UIViewAutoresizing::FlexibleWidth | objc2_ui_kit::UIViewAutoresizing::FlexibleHeight);
    effect_view.layer().setCornerRadius(16.0);
    effect_view.setClipsToBounds(true);
    effect_view.setUserInteractionEnabled(false);
    menu.addSubview(&effect_view);

    let layer = menu.layer();
    layer.setCornerRadius(16.0);
    unsafe {
        let shadow_color = Theme::shadow().CGColor();
        layer.setShadowColor(Some(&shadow_color));
        let border_color = Theme::menu_border().CGColor();
        layer.setBorderColor(Some(&border_color));
        layer.setBorderWidth(1.0);
    }
    layer.setShadowOffset(CGSize::new(0.0, 8.0));
    layer.setShadowRadius(24.0);
    layer.setShadowOpacity(0.6);
    menu.setClipsToBounds(false);
    menu.setUserInteractionEnabled(true);

    let handler = MenuActionHandler::new(mtm);
    let header: Retained<UIView> = UIView::initWithFrame(UIView::alloc(mtm), CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(frame.size.width, 50.0)));
    header.setBackgroundColor(Some(&Theme::header()));
    header.setClipsToBounds(true);
    header.layer().setCornerRadius(16.0);
    menu.addSubview(&header);

    let pan_gesture = unsafe {
        let gesture: Allocated<UIPanGestureRecognizer> = msg_send![UIPanGestureRecognizer::class(), alloc];
        let gesture: Retained<UIPanGestureRecognizer> = msg_send![gesture, initWithTarget: &*handler, action: sel!(handlePan:)];
        gesture
    };
    menu.addGestureRecognizer(&pan_gesture);

    let tap_gesture = unsafe {
        let gesture: Allocated<UITapGestureRecognizer> = msg_send![UITapGestureRecognizer::class(), alloc];
        let gesture: Retained<UITapGestureRecognizer> = msg_send![gesture, initWithTarget: &*handler, action: sel!(handleTap:)];
        gesture.setCancelsTouchesInView(false);
        gesture
    };
    menu.addGestureRecognizer(&tap_gesture);

    header.addSubview(&create_label(CGRect::new(CGPoint::new(16.0, 12.0), CGSize::new(200.0, 26.0)), "RGG - v0.2.0", 18.0, true, mtm));
    let close_btn = UIButton::initWithFrame(UIButton::alloc(mtm), CGRect::new(CGPoint::new(frame.size.width - 44.0, 8.0), CGSize::new(36.0, 36.0)));
    close_btn.setTitle_forState(Some(&NSString::from_str("✕")), UIControlState::Normal);
    close_btn.setTitleColor_forState(Some(&Theme::text_secondary()), UIControlState::Normal);
    close_btn.setUserInteractionEnabled(true);
    close_btn.setTag(99);
    unsafe { close_btn.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
    header.addSubview(&close_btn);

    let scroll_view: Retained<UIView> = unsafe {
        let view: Allocated<UIView> = msg_send![objc2::class!(UIScrollView), alloc];
        let view: Retained<UIView> = msg_send![view, initWithFrame: CGRect::new(CGPoint::new(0.0, 50.0), CGSize::new(frame.size.width, frame.size.height - 50.0))];
        view
    };
    menu.addSubview(&scroll_view);
    ACTION_HANDLER.with(|h| *h.borrow_mut() = Some(handler));
    SCROLL_VIEW.with(|s| *s.borrow_mut() = Some(scroll_view));
    render_content(0);
    menu
}

fn clear_content(view: &UIView) {
    let subviews = view.subviews();
    for i in 0..subviews.count() {
        subviews.objectAtIndex(i).removeFromSuperview();
    }
}

fn hex_to_color(hex: &str) -> Option<Retained<objc2_ui_kit::UIColor>> {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 { return None; }
    let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
    let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
    let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
    Some(unsafe { msg_send![objc2_ui_kit::UIColor::class(), colorWithRed: r, green: g, blue: b, alpha: 1.0] })
}

pub fn render_content(page_id: i32) {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    SCROLL_VIEW.with(|s| {
        if let Some(scroll_view) = s.borrow().as_ref() {
            clear_content(scroll_view);
            let frame = scroll_view.frame();
            let mut y_offset: f64 = 10.0;
            let (item_height, padding) = (44.0, 16.0);
            let handler = ACTION_HANDLER.with(|h| h.borrow().clone()).unwrap();
            
            if page_id != 0 {
                let title = REGISTRY.lock().unwrap().page_titles.get(&page_id).cloned().unwrap_or_else(|| "Menu".to_string());
                let header = create_header(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, 40.0)), &title, mtm);
                header.setTag(200);
                unsafe { header.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                scroll_view.addSubview(&header);
                y_offset += 50.0;
            }

            let registry = REGISTRY.lock().unwrap();
            if let Some(items) = registry.pages.get(&page_id) {
                for item in items {
                    match item {
                        MenuItem::Button { id, name, .. } => {
                             let btn = create_button_item(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, item_height)), name, mtm);
                             btn.setTag(*id as isize);
                             unsafe { btn.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                             scroll_view.addSubview(&btn);
                             y_offset += item_height + 10.0;
                        }
                        MenuItem::Toggle { id, name, key, default, .. } => {
                             let toggle = create_toggle_item(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, item_height)), name, mtm);
                             toggle.setTag(*id as isize);
                             let mut current = super::pref::Preferences::get_bool(key);
                             if !current && *default { current = *default; }
                             toggle.setSelected(current);
                             if let Some(bg) = toggle.viewWithTag(2) {
                                 let color = if current { Theme::accent() } else { Theme::toggle_off() };
                                 bg.setBackgroundColor(Some(&color));
                                 if let Some(knob) = bg.viewWithTag(3) {
                                     let mut f = knob.frame();
                                     f.origin.x = if current { 22.0 } else { 2.0 };
                                     knob.setFrame(f);
                                     if current { knob.setBackgroundColor(Some(&Theme::knob_on())); }
                                 }
                             }
                             unsafe { toggle.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                             scroll_view.addSubview(&toggle);
                             y_offset += item_height + 10.0;
                        }
                        MenuItem::Slider { id, name, key, min, max, default, .. } => {
                             let mut current = super::pref::Preferences::get_float(key);
                             if current == 0.0 && *default != 0.0 { current = *default; }
                             let slider_item = create_slider_item(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, 70.0)), name, current, *min, *max, mtm);
                             if let Some(slider) = slider_item.viewWithTag(4) {
                                   slider.setTag(*id as isize);
                                   unsafe { let _: () = msg_send![&slider, addTarget: &*handler, action: sel!(handleSlider:), forControlEvents: UIControlEvents::ValueChanged]; }
                             }
                             scroll_view.addSubview(&slider_item);
                             y_offset += 80.0;
                        }
                        MenuItem::Input { id, name, key, placeholder, default, .. } => {
                             let input_item = create_text_input_item(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, 70.0)), name, placeholder, mtm);
                             if let Some(input) = input_item.viewWithTag(6) {
                                   input.setTag(*id as isize);
                                   let current = super::pref::Preferences::get_string(key);
                                   let val = if current.is_empty() { default } else { &current };
                                   if !val.is_empty() { let _: () = unsafe { msg_send![&input, setText: &*NSString::from_str(val)] }; }
                                   unsafe { let _: () = msg_send![&input, addTarget: &*handler, action: sel!(handleTextChange:), forControlEvents: UIControlEvents::EditingDidEnd]; }
                             }
                             scroll_view.addSubview(&input_item);
                             y_offset += 80.0;
                        }
                        MenuItem::ActionButton { id, name, .. } => {
                            let action_btn = super::components::create_action_button_item(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, item_height)), name, mtm);
                            action_btn.setTag(*id as isize);
                            unsafe { action_btn.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                            scroll_view.addSubview(&action_btn);
                            y_offset += item_height + 10.0;
                        }
                        MenuItem::Label { text, font_size, is_bold, color, .. } => {
                            let label = create_label(CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, 20.0)), text, *font_size as f64, *is_bold, mtm);
                            if let Some(hex) = color {
                                if let Some(ui_color) = hex_to_color(hex) {
                                    unsafe { label.setTextColor(Some(&ui_color)); }
                                }
                            }
                            scroll_view.addSubview(&label);
                            y_offset += 28.0;
                        }
                    }
                }
            }
            unsafe { let _: () = msg_send![scroll_view, setContentSize: CGSize::new(frame.size.width, y_offset + 20.0)]; }
        }
    });
}


pub fn toggle_menu(menu: &UIView) {
    let is_hidden = menu.isHidden();
    menu.setHidden(!is_hidden);
}


pub fn show_menu(menu: &UIView) {
    menu.setHidden(false);
}


pub fn hide_menu(menu: &UIView) {
    menu.setHidden(true);
}
