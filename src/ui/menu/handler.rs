//! Handler for items
use objc2::rc::Retained;
use objc2::{define_class, msg_send, ClassType};
use objc2_core_foundation::CGPoint;
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use objc2_ui_kit::{
    UIButton, UIGestureRecognizerState, UIPanGestureRecognizer, UISlider, UITapGestureRecognizer,
    UITextField, UIView,
};

use super::registry::{MenuItem, REGISTRY};
use super::utils::trigger_feedback;
use crate::ui::menu::view::{render_content, update_card_toggle_ui, update_toggle_ui};
use crate::ui::pref::Preferences;
use crate::ui::window::hide_menu;

thread_local! {
    pub static ACTION_HANDLER: std::cell::RefCell<Option<Retained<MenuActionHandler>>> = const { std::cell::RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "RustMenuActionHandler"]
    /// Helper class for handling UI control events (buttons, sliders, gestures)
    pub struct MenuActionHandler;

    impl MenuActionHandler {
        #[unsafe(method(handleAction:))]
        /// Handles button taps and toggle switches
        fn handle_action(&self, sender: &UIButton) {
            trigger_feedback();
            let tag = sender.tag();
            let registry = REGISTRY.lock();
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
                         Preferences::set_bool(&key, selected);
                         update_toggle_ui(sender, selected);
                         if let Some(cb) = callback { cb(selected); }
                         return;
                    }
                    MenuItem::Slider { toggle: Some(toggle), .. }
                    | MenuItem::Input { toggle: Some(toggle), .. } => {
                        let selected = !sender.isSelected();
                        sender.setSelected(selected);
                        Preferences::set_bool(&toggle.key, selected);
                        update_card_toggle_ui(sender, selected);
                        if let Some(cb) = &toggle.callback { cb(selected); }
                        return;
                    }
                    MenuItem::ActionButton { callback, .. } => {
                        trigger_feedback();
                        crate::ui::utils::animations::pulse(sender, 0.96, 0.2);
                        if let Some(cb) = callback { cb(); }
                        return;
                    }
                    MenuItem::Dropdown { id: _, name, key, options, default: _, callback } => {
                        let current_idx = Preferences::get_int(&key);
                        let key_clone = key.clone();
                        let callback_clone = callback.clone();
                        super::view::show_dropdown_selection(
                            &name,
                            options,
                            current_idx,
                            move |idx| {
                                Preferences::set_int(&key_clone, idx);
                                if let Some(cb) = &callback_clone { cb(idx); }
                                render_content(Preferences::get_int("last_page"));
                            }
                        );
                        return;
                    }

                    _ => {},
                }
            } else { drop(registry); }
            if tag == 99 { hide_menu(); }
            else if (200..300).contains(&tag) { render_content((tag - 200) as i32); }
            else if (400..500).contains(&tag) { render_content((tag - 400) as i32); }
        }

        #[unsafe(method(handleSlider:))]
        /// Handles slider value changes
        fn handle_slider(&self, sender: &UISlider) {
             let (value, tag) = (sender.value(), sender.tag());
             unsafe {
                 let container: Option<Retained<UIView>> = msg_send![sender, superview];
                 if let Some(cont) = container {
                     if let Some(label) = cont.viewWithTag(5) {
                         let _: () = msg_send![&label, setText: &*NSString::from_str(&format!("{:.0}", value))];
                     }
                 }
             }
             if let Some(MenuItem::Slider { callback, key, .. }) = REGISTRY.lock().items_by_id.get(&(tag as i32)) {
                 Preferences::set_float(key, value);
                 if let Some(cb) = callback { cb(value); }
             }
        }

        #[unsafe(method(handleTextChange:))]
        /// Handles text input changes
        fn handle_text_change(&self, sender: &UITextField) {
             let (tag, text) = (sender.tag(), sender.text().map(|t| t.to_string()).unwrap_or_default());
             if let Some(MenuItem::Input { callback, key, .. }) = REGISTRY.lock().items_by_id.get(&(tag as i32)) {
                 Preferences::set_string(key, &text);
                 if let Some(cb) = callback { cb(text); }
             }
        }

        #[unsafe(method(handleTap:))]
        /// Handles background taps (dismisses keyboard)
        fn handle_tap(&self, recognizer: &UITapGestureRecognizer) {
            if let Some(view) = recognizer.view() { view.endEditing(true); }
        }

        #[unsafe(method(handlePan:))]
        /// Handles standard pan gestures (dragging the menu window)
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
    pub fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![MenuActionHandler::class(), new] }
    }
}
