//! Window control

use super::components::create_toggle_button;
use super::menu::create_menu_view;
use crate::ui::utils::animations;
use crate::utils::logger;
use dispatch::Queue;
use objc2::{
    define_class, msg_send,
    rc::{Allocated, Retained},
    runtime::{AnyObject, Class, NSObject},
    sel, ClassType,
};
use objc2_core_foundation::{CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_ui_kit::{
    UIApplication, UIButton, UIControlEvents, UIGestureRecognizerState, UIPanGestureRecognizer,
    UIView, UIWindow,
};
use std::cell::RefCell;
use std::time::Duration;

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "RustMenuHandler"]
    /// Handler for menu interaction events (toggle, pan gesture)
    pub struct MenuHandler;

    impl MenuHandler {
        #[unsafe(method(handleToggle))]
        fn handle_toggle(&self) {
            toggle_menu();
        }

        #[unsafe(method(handlePan:))]
        fn handle_pan(&self, recognizer: &UIPanGestureRecognizer) {
            use block2::RcBlock;
            let state = recognizer.state();
            if let Some(view) = recognizer.view() {
                let translation = recognizer.translationInView(Some(&view));
                let mut center = view.center();
                center.x += translation.x;
                center.y += translation.y;

                if state == UIGestureRecognizerState::Began {
                    // Scale up when dragging begins
                    let view_ptr = view.clone();
                    animations::animate_spring(
                        0.2,
                        0.7,
                        0.5,
                        move || unsafe {
                            view_ptr.setTransform(CGAffineTransform {
                                a: 1.15,
                                b: 0.0,
                                c: 0.0,
                                d: 1.15,
                                tx: 0.0,
                                ty: 0.0,
                            });
                        },
                        None::<fn(bool)>,
                    );
                } else if state == UIGestureRecognizerState::Changed {
                    // Allow free dragging during the gesture
                    view.setCenter(center);
                    recognizer.setTranslation_inView(CGPoint::new(0.0, 0.0), Some(&view));
                } else if state == UIGestureRecognizerState::Ended || state == UIGestureRecognizerState::Cancelled {
                    view.setCenter(center);
                    recognizer.setTranslation_inView(CGPoint::new(0.0, 0.0), Some(&view));

                    // Check if near screen bounds and animate to half-visible position
                    if let Some(superview) = view.superview() {
                        let super_bounds = superview.bounds();
                        let view_bounds = view.bounds();
                        let half_w = view_bounds.size.width * 0.5;
                        let half_h = view_bounds.size.height * 0.5;
                        let edge_threshold = 20.0; // Distance from edge to trigger snap

                        let mut final_center = center;
                        let mut needs_snap = false;

                        // Left edge: if within threshold of left edge, snap to half visible
                        if center.x < half_w + edge_threshold {
                            final_center.x = 0.0;
                            needs_snap = true;
                        }
                        // Right edge: if within threshold of right edge, snap to half visible
                        else if center.x > super_bounds.size.width - half_w - edge_threshold {
                            final_center.x = super_bounds.size.width;
                            needs_snap = true;
                        }

                        // Top edge: if within threshold of top edge, snap to half visible
                        if center.y < half_h + edge_threshold {
                            final_center.y = 0.0;
                            needs_snap = true;
                        }
                        // Bottom edge: if within threshold of bottom edge, snap to half visible
                        else if center.y > super_bounds.size.height - half_h - edge_threshold {
                            final_center.y = super_bounds.size.height;
                            needs_snap = true;
                        }

                        // Animate scale back to normal + snap to edge if needed
                        let view_ptr = view.clone();
                        animations::animate_spring(
                            0.3,
                            0.8,
                            0.5,
                            move || unsafe {
                                view_ptr.setTransform(CGAffineTransform {
                                    a: 1.0,
                                    b: 0.0,
                                    c: 0.0,
                                    d: 1.0,
                                    tx: 0.0,
                                    ty: 0.0,
                                });
                                if needs_snap {
                                    view_ptr.setCenter(final_center);
                                }
                            },
                            None::<fn(bool)>,
                        );
                    }
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
    static ACTIVE_ALERT: RefCell<Option<Retained<AnyObject>>> = const { RefCell::new(None) };
}

/// Helper function to find the key window or the first available window
pub fn get_window(mtm: MainThreadMarker) -> Option<Retained<UIWindow>> {
    let app = UIApplication::sharedApplication(mtm);
    app.keyWindow().or_else(|| {
        let windows = app.windows();
        if windows.count() > 0 {
            Some(windows.objectAtIndex(0))
        } else {
            None
        }
    })
}

/// Initializes the native iOS overlay and attaches it to the application window
///
/// This creates the hidden menu view, the floating toggle button, and sets up
/// the necessary gesture recognizers. It must be called on the main thread.
pub fn init_overlay() {
    let mtm = MainThreadMarker::new().expect("init_overlay must be called on the main thread");
    #[cfg(dev_release)]
    logger::info("Creating native iOS overlay...");
    let app = UIApplication::sharedApplication(mtm);

    let window_opt = get_window(mtm);

    let window = match window_opt {
        Some(w) => w,
        None => {
            #[cfg(dev_release)]
            logger::warning("Failed to get windows! Retrying in 1 second...");
            Queue::main().exec_after(Duration::from_secs(1), || {
                init_overlay();
            });
            return;
        }
    };

    let bounds = window.bounds();
    let menu_width = 300.0;
    let menu_height = 360.0;
    let x = (bounds.size.width - menu_width) / 2.0;
    let y = (bounds.size.height - menu_height) / 2.0;

    let menu = create_menu_view(
        CGRect::new(CGPoint::new(x, y), CGSize::new(menu_width, menu_height)),
        mtm,
    );
    menu.setHidden(true);
    menu.setUserInteractionEnabled(true);
    window.addSubview(&menu);

    MENU_VIEW.with(|m| *m.borrow_mut() = Some(menu));

    let window_ptr = Retained::into_raw(window.clone()) as usize;

    super::components::toast::show_welcome(
        crate::config::MENU_NAME,
        crate::config::VERSION,
        crate::config::WELCOME_MESSAGE,
        Box::new(move || {
            if let Some(mtm) = MainThreadMarker::new() {
                let window =
                    unsafe { Retained::<UIWindow>::from_raw(window_ptr as *mut _).unwrap() };
                init_floating_button(window, mtm);
            }
        }),
    );

    #[cfg(dev_release)]
    logger::info("Native iOS overlay attached to main window!");
}

fn init_floating_button(window: Retained<UIWindow>, mtm: MainThreadMarker) {
    let toggle_btn = create_toggle_button(
        CGRect::new(CGPoint::new(20.0, 100.0), CGSize::new(50.0, 50.0)),
        mtm,
    );
    let handler = MenuHandler::new(mtm);
    unsafe {
        toggle_btn.addTarget_action_forControlEvents(
            Some(&handler),
            sel!(handleToggle),
            UIControlEvents::TouchUpInside,
        );
    }
    let pan_gesture = unsafe {
        let gesture: Allocated<UIPanGestureRecognizer> =
            msg_send![UIPanGestureRecognizer::class(), alloc];
        let gesture: Retained<UIPanGestureRecognizer> =
            msg_send![gesture, initWithTarget: &*handler, action: sel!(handlePan:)];
        gesture
    };
    toggle_btn.addGestureRecognizer(&pan_gesture);
    MENU_HANDLER.with(|h| *h.borrow_mut() = Some(handler));

    // Scale animation
    unsafe {
        // Initial Position: Center of Screen
        let window_bounds = window.bounds();
        let center_x = window_bounds.size.width / 2.0;
        let center_y = window_bounds.size.height / 2.0;
        toggle_btn.setCenter(CGPoint::new(center_x, center_y));

        window.addSubview(&toggle_btn);

        use block2::RcBlock;
        let btn_ptr = toggle_btn.clone();

        // Target Position: Right side, 50% inside screen (half visible)
        let target_x = window_bounds.size.width; // Half visible on right edge
        let target_y = center_y;

        animations::animate_spring(
            1.5,
            0.8,
            0.0,
            move || {
                btn_ptr.setCenter(CGPoint::new(target_x, target_y));
            },
            None::<fn(bool)>,
        );
    }

    TOGGLE_BTN.with(|b| *b.borrow_mut() = Some(toggle_btn));
}

/// Toggles the menu visibility (show if hidden, hide if shown)
pub fn toggle_menu() {
    MENU_VIEW.with(|m| {
        if let Some(menu) = m.borrow().as_ref() {
            if menu.isHidden() {
                show_menu();
            } else {
                hide_menu();
            }
        }
    });
}

/// Shows the menu and hides the floating button
///
/// Triggers the "open" animation sequence.
pub fn show_menu() {
    MENU_VIEW.with(|m| {
        if let Some(menu) = m.borrow().as_ref() {
            super::menu::show_menu(menu);
        }
    });
    TOGGLE_BTN.with(|b| {
        if let Some(btn) = b.borrow().as_ref() {
            btn.setHidden(true);
        }
    });
}

/// Hides the menu and shows the floating button
///
/// Triggers the "close" animation sequence.
pub fn hide_menu() {
    MENU_VIEW.with(|m| {
        if let Some(menu) = m.borrow().as_ref() {
            super::menu::hide_menu(menu);
        }
    });
    TOGGLE_BTN.with(|b| {
        if let Some(btn) = b.borrow().as_ref() {
            btn.setHidden(false);
        }
    });
}

/// Checks if the menu is currently visible (not hidden)
///
/// # Returns
/// * `bool` - `true` if visible, `false` otherwise
pub fn is_menu_visible() -> bool {
    MENU_VIEW.with(|m| {
        m.borrow()
            .as_ref()
            .map(|menu| !menu.isHidden())
            .unwrap_or(false)
    })
}

/// Displays a native iOS alert dialog
///
/// # Arguments
/// * `title` - The alert title
/// * `text` - The alert message
/// * `with_ok` - If true, adds an "OK" button (otherwise it's just a non-dismissible message, useful for panic screens)
pub fn alert(title: &str, text: &str, with_ok: bool) {
    let text = text.to_string();
    let title = title.to_string();
    Queue::main().exec_async(move || {
        if let Some(mtm) = MainThreadMarker::new() {
            unsafe {
                let title = NSString::from_str(&title);
                let message: Retained<NSString> = NSString::from_str(&text);

                let alert_cls =
                    Class::get(c"UIAlertController").expect("UIAlertController class not found");
                let alert: Retained<AnyObject> = msg_send![
                    alert_cls,
                    alertControllerWithTitle: &*title,
                    message: &*message,
                    preferredStyle: 1usize // UIAlertControllerStyleAlert
                ];

                if with_ok {
                    let action_cls =
                        Class::get(c"UIAlertAction").expect("UIAlertAction class not found");
                    let ok_title = NSString::from_str("OK");
                    let action: Retained<AnyObject> = msg_send![
                        action_cls,
                        actionWithTitle: &*ok_title,
                        style: 0usize, // UIAlertActionStyleDefault
                        handler: 0 as *const std::ffi::c_void
                    ];

                    let _: () = msg_send![&alert, addAction: &*action];
                }

                let dismissed = ACTIVE_ALERT.with(|a| {
                   if let Some(old) = a.borrow().as_ref() {
                       let _: () = msg_send![old, dismissViewControllerAnimated: false completion: 0 as *const std::ffi::c_void];
                       true
                   } else {
                       false
                   }
                });

                ACTIVE_ALERT.with(|a| *a.borrow_mut() = Some(alert.clone()));

                let alert_ptr = Retained::into_raw(alert) as usize;

                let present = move || {
                    let alert = unsafe { Retained::from_raw(alert_ptr as *mut AnyObject).unwrap() };
                    if let Some(mtm) = MainThreadMarker::new() {
                        if let Some(window) = get_window(mtm) {
                            if let Some(root) = window.rootViewController() {
                                unsafe {
                                    let _: () = msg_send![
                                        &root,
                                        presentViewController: &*alert,
                                        animated: true,
                                        completion: 0 as *const std::ffi::c_void
                                    ];
                                }
                            }
                        }
                    }
                };

                if dismissed {
                    Queue::main().exec_after(Duration::from_millis(100), present);
                } else {
                    present();
                }
            }
        }
    });
}
