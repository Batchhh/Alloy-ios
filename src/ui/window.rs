use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, sel, ClassType};
use objc2::runtime::NSObject;
use objc2_core_foundation::{CGPoint, CGRect, CGSize};
use objc2_foundation::MainThreadMarker;
use objc2_ui_kit::{UIApplication, UIButton, UIControlEvents, UIGestureRecognizerState, UIPanGestureRecognizer, UIView};
use std::cell::RefCell;
use dispatch::Queue;
use std::time::Duration;
use super::components::create_toggle_button;
use super::menu::create_menu_view;
use crate::utils::logger;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "RustMenuHandler"]
    pub struct MenuHandler;

    impl MenuHandler {
        #[unsafe(method(handleToggle))]
        fn handle_toggle(&self) {
            logger::info("Toggle button tapped!");
            toggle_menu();
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

                    // Constrain to superview bounds
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
    }
);

impl MenuHandler {
    fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![MenuHandler::class(), new] }
    }
}

thread_local! {
    static MENU_VIEW: RefCell<Option<Retained<UIView>>> = const { RefCell::new(None) };
    static TOGGLE_BTN: RefCell<Option<Retained<UIButton>>> = const { RefCell::new(None) };
    static MENU_HANDLER: RefCell<Option<Retained<MenuHandler>>> = const { RefCell::new(None) };
}

pub fn init_overlay(mtm: MainThreadMarker) {
    logger::info("Creating native iOS overlay...");
    let app = UIApplication::sharedApplication(mtm);
    
    // Try to get the window
    #[allow(deprecated)]
    let window_opt = if let Some(w) = app.keyWindow() {
        Some(w)
    } else {
        #[allow(deprecated)]
        let windows = app.windows();
        if windows.count() > 0 {
             logger::info("keyWindow was nil, using first window in array");
             Some(windows.objectAtIndex(0))
        } else {
            None
        }
    };

    let window = match window_opt {
        Some(w) => w,
        None => {
            logger::warning("Failed to get windows! Retrying in 1 second...");
            Queue::main().exec_after(Duration::from_secs(1), || {
                if let Some(mtm) = MainThreadMarker::new() {
                    init_overlay(mtm);
                }
            });
            return;
        }
    };
    
    let toggle_btn = create_toggle_button(CGRect::new(CGPoint::new(20.0, 100.0), CGSize::new(56.0, 56.0)), mtm);
    let handler = MenuHandler::new(mtm);
    unsafe {
        toggle_btn.addTarget_action_forControlEvents(Some(&handler), sel!(handleToggle), UIControlEvents::TouchUpInside);
    }
    let pan_gesture = unsafe {
        let gesture: Allocated<UIPanGestureRecognizer> = msg_send![UIPanGestureRecognizer::class(), alloc];
        let gesture: Retained<UIPanGestureRecognizer> = msg_send![gesture, initWithTarget: &*handler, action: sel!(handlePan:)];
        gesture
    };
    toggle_btn.addGestureRecognizer(&pan_gesture);
    MENU_HANDLER.with(|h| *h.borrow_mut() = Some(handler));
    logger::info(&format!("Toggle button created with userInteractionEnabled: {}", toggle_btn.isUserInteractionEnabled()));
    window.addSubview(&toggle_btn);
    
    let bounds = window.bounds();
    let menu_width = 300.0;
    let menu_height = 360.0;
    let x = (bounds.size.width - menu_width) / 2.0;
    let y = (bounds.size.height - menu_height) / 2.0;
    
    let menu = create_menu_view(CGRect::new(CGPoint::new(x, y), CGSize::new(menu_width, menu_height)), mtm);
    menu.setHidden(true);
    menu.setUserInteractionEnabled(true);
    window.addSubview(&menu);
    
    MENU_VIEW.with(|m| *m.borrow_mut() = Some(menu));
    TOGGLE_BTN.with(|b| *b.borrow_mut() = Some(toggle_btn));
    logger::info("Native iOS overlay attached to main window!");
}

pub fn toggle_menu() {
    MENU_VIEW.with(|m| {
        if let Some(menu) = m.borrow().as_ref() {
            if menu.isHidden() { show_menu(); } else { hide_menu(); }
        }
    });
}


pub fn show_menu() {
    MENU_VIEW.with(|m| { if let Some(menu) = m.borrow().as_ref() { super::menu::show_menu(menu); } });
    TOGGLE_BTN.with(|b| { if let Some(btn) = b.borrow().as_ref() { btn.setHidden(true); } });
}


pub fn hide_menu() {
    MENU_VIEW.with(|m| { if let Some(menu) = m.borrow().as_ref() { super::menu::hide_menu(menu); } });
    TOGGLE_BTN.with(|b| { if let Some(btn) = b.borrow().as_ref() { btn.setHidden(false); } });
}


pub fn is_menu_visible() -> bool {
    MENU_VIEW.with(|m| m.borrow().as_ref().map(|menu| !menu.isHidden()).unwrap_or(false))
}
