//! View

use objc2::rc::{Allocated, Retained};
use objc2::{define_class, msg_send, sel, ClassType, MainThreadOnly};
use objc2_core_foundation::{CGAffineTransform, CGPoint, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSObject, NSString};
use objc2_ui_kit::{
    UIButton, UIControlEvents, UIControlState, UIPanGestureRecognizer, UIScrollView,
    UITapGestureRecognizer, UIView,
};
use std::cell::RefCell;

use super::handler::{MenuActionHandler, ACTION_HANDLER};
use super::items::{
    create_action_button_item, create_button_item, create_dropdown_item, create_slider_item,
    create_text_input_item, create_toggle_item,
};
use super::registry::{MenuItem, REGISTRY, TAB_REGISTRY};
use super::utils::hex_to_color;
use crate::ui::components::{create_label, create_section_header};
use crate::ui::pref::Preferences;
use crate::ui::theme::Theme;
use crate::ui::utils::animations;
use crate::ui::utils::wrappers::{CAGradientLayer, UIBlurEffect, UIVisualEffectView};

thread_local! {
    static SCROLL_VIEW: RefCell<Option<Retained<UIScrollView>>> = const { RefCell::new(None) };
    #[allow(clippy::type_complexity)]
    static DROPDOWN_CALLBACK: RefCell<Option<Box<dyn Fn(i32)>>> = const { RefCell::new(None) };
    static DROPDOWN_DELEGATE: RefCell<Option<Retained<DropdownDelegate>>> = const { RefCell::new(None) };
}

define_class!(
    #[unsafe(super(NSObject))]
    #[name = "RustDropdownDelegate"]
    struct DropdownDelegate;

    impl DropdownDelegate {
        #[unsafe(method(handleOption:))]
        fn handle_option(&self, sender: &UIButton) {
            DROPDOWN_CALLBACK.with(|cb| {
                if let Some(f) = cb.borrow().as_ref() {
                    f(sender.tag() as i32);
                }
            });
            unsafe { let _: () = msg_send![self, dismissOverlay]; }
        }

        #[unsafe(method(handleDismiss:))]
        fn handle_dismiss(&self, _sender: &UITapGestureRecognizer) {
            unsafe { let _: () = msg_send![self, dismissOverlay]; }
        }

        #[unsafe(method(dismissOverlay))]
        fn dismiss_overlay(&self) {
            let mtm = unsafe { MainThreadMarker::new_unchecked() };
            #[allow(deprecated)]
            let app = objc2_ui_kit::UIApplication::sharedApplication(mtm);
            #[allow(deprecated)]
            if let Some(window) = app.keyWindow() {
                if let Some(overlay) = window.viewWithTag(9999) {
                    let overlay_ptr = overlay.clone();
                    animations::animate(0.2, move || { overlay_ptr.setAlpha(0.0); },
                        Some(move |_: bool| { overlay.removeFromSuperview(); }));
                }
            }
        }
    }
);

impl DropdownDelegate {
    fn new(_mtm: MainThreadMarker) -> Retained<Self> {
        unsafe { msg_send![DropdownDelegate::class(), new] }
    }
}

// Helper: create gesture recognizer
fn create_gesture<T: ClassType>(
    _mtm: MainThreadMarker,
    target: &impl objc2::Message,
    action: objc2::runtime::Sel,
) -> Retained<T> {
    unsafe {
        let gesture: Allocated<T> = msg_send![T::class(), alloc];
        msg_send![gesture, initWithTarget: target, action: action]
    }
}

// Helper: setup layer styling
fn setup_layer(layer: &objc2_quartz_core::CALayer, shadow_offset: CGSize) {
    layer.setCornerRadius(16.0);
    unsafe {
        layer.setShadowColor(Some(&Theme::shadow().CGColor()));
        layer.setBorderColor(Some(&Theme::menu_border().CGColor()));
        layer.setBorderWidth(1.0);
        layer.setShadowOffset(shadow_offset);
        layer.setShadowRadius(24.0);
        layer.setShadowOpacity(0.6);
    }
}

// Helper: create button with frame
fn create_btn(mtm: MainThreadMarker, frame: CGRect) -> Retained<UIButton> {
    UIButton::initWithFrame(UIButton::alloc(mtm), frame)
}

pub fn create_menu_view(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIView> {
    let menu = UIView::initWithFrame(UIView::alloc(mtm), frame);
    menu.setBackgroundColor(Some(&objc2_ui_kit::UIColor::clearColor()));

    // Gradient
    let gradient = CAGradientLayer::new();
    gradient.setFrame(menu.bounds());
    let colors = objc2_foundation::NSArray::from_retained_slice(&[
        unsafe { Retained::cast_unchecked::<objc2_foundation::NSObject>(Theme::gradient_start()) },
        unsafe { Retained::cast_unchecked::<objc2_foundation::NSObject>(Theme::gradient_end()) },
    ]);
    gradient.setColors(&colors);
    menu.layer().insertSublayer_atIndex(&gradient, 0);

    // Blur effect
    let blur_effect = unsafe {
        let effect: Retained<UIBlurEffect> =
            msg_send![UIBlurEffect::class(), effectWithStyle: 2i64];
        effect
    };
    let effect_view = UIVisualEffectView::new(&blur_effect, mtm);
    effect_view.setFrame(menu.bounds());
    effect_view.setAutoresizingMask(
        objc2_ui_kit::UIViewAutoresizing::FlexibleWidth
            | objc2_ui_kit::UIViewAutoresizing::FlexibleHeight,
    );
    effect_view.layer().setCornerRadius(16.0);
    effect_view.setClipsToBounds(true);
    effect_view.setUserInteractionEnabled(false);
    menu.addSubview(&effect_view);

    // Layer styling
    setup_layer(&menu.layer(), CGSize::new(0.0, 8.0));
    menu.setClipsToBounds(false);
    menu.setUserInteractionEnabled(true);

    // Gesture handlers
    let handler = MenuActionHandler::new(mtm);
    menu.addGestureRecognizer(&create_gesture::<UIPanGestureRecognizer>(
        mtm,
        &*handler,
        sel!(handlePan:),
    ));
    let tap_gesture: Retained<UITapGestureRecognizer> =
        create_gesture(mtm, &*handler, sel!(handleTap:));
    tap_gesture.setCancelsTouchesInView(false);
    menu.addGestureRecognizer(&tap_gesture);

    // Close button
    let close_btn = create_btn(
        mtm,
        CGRect::new(
            CGPoint::new((frame.size.width - 40.0) / 2.0, frame.size.height - 44.0),
            CGSize::new(40.0, 40.0),
        ),
    );
    close_btn.setTitle_forState(Some(&NSString::from_str("✕")), UIControlState::Normal);
    close_btn.setTitleColor_forState(Some(&Theme::text_secondary()), UIControlState::Normal);
    close_btn.setUserInteractionEnabled(true);
    close_btn.setTag(99);
    unsafe {
        close_btn.addTarget_action_forControlEvents(
            Some(&handler),
            sel!(handleAction:),
            UIControlEvents::TouchUpInside,
        );
    }

    // Tab bar
    let tabs = TAB_REGISTRY.lock().unwrap().clone();
    let tab_bar_height = if tabs.is_empty() { 0.0 } else { 50.0 };

    if !tabs.is_empty() {
        let tab_bar = UIView::initWithFrame(
            UIView::alloc(mtm),
            CGRect::new(
                CGPoint::new(0.0, 0.0),
                CGSize::new(frame.size.width, tab_bar_height),
            ),
        );
        tab_bar.setTag(999);

        // Separator
        let separator = UIView::initWithFrame(
            UIView::alloc(mtm),
            CGRect::new(
                CGPoint::new(0.0, tab_bar_height - 0.5),
                CGSize::new(frame.size.width, 0.5),
            ),
        );
        separator.setBackgroundColor(Some(&Theme::menu_border()));
        tab_bar.addSubview(&separator);

        // Indicator
        let btn_width = frame.size.width / tabs.len() as f64;
        let indicator = UIView::initWithFrame(
            UIView::alloc(mtm),
            CGRect::new(
                CGPoint::new(0.0, tab_bar_height - 3.0),
                CGSize::new(btn_width, 3.0),
            ),
        );
        indicator.setBackgroundColor(Some(&Theme::accent()));
        indicator.layer().setCornerRadius(1.5);
        indicator.setTag(888);
        tab_bar.addSubview(&indicator);

        // Tab buttons
        for (i, (name, target_page)) in tabs.iter().enumerate() {
            let btn = create_btn(
                mtm,
                CGRect::new(
                    CGPoint::new(btn_width * i as f64, 0.0),
                    CGSize::new(btn_width, tab_bar_height),
                ),
            );
            btn.setTitle_forState(Some(&NSString::from_str(name)), UIControlState::Normal);
            unsafe {
                btn.setTitleColor_forState(Some(&Theme::text_secondary()), UIControlState::Normal);
                btn.setTitleColor_forState(Some(&Theme::text()), UIControlState::Selected);
                if let Some(label) = btn.titleLabel() {
                    label.setFont(Some(&objc2_ui_kit::UIFont::systemFontOfSize(13.0)));
                }
                btn.addTarget_action_forControlEvents(
                    Some(&handler),
                    sel!(handleAction:),
                    UIControlEvents::TouchUpInside,
                );
            }
            btn.setTag(400 + *target_page as isize);
            if *target_page == 0 {
                btn.setSelected(true);
            }
            tab_bar.addSubview(&btn);
        }
        menu.addSubview(&tab_bar);
    }

    menu.addSubview(&close_btn);

    // Scroll view
    let scroll_view: Retained<UIScrollView> = unsafe {
        let view: Allocated<UIScrollView> = msg_send![objc2::class!(UIScrollView), alloc];
        msg_send![view, initWithFrame: CGRect::new(CGPoint::new(0.0, tab_bar_height),
            CGSize::new(frame.size.width, frame.size.height - tab_bar_height - 44.0))]
    };
    menu.addSubview(&scroll_view);

    ACTION_HANDLER.with(|h| *h.borrow_mut() = Some(handler.clone()));
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

// Helper: update tab button state
fn update_tab_button(btn: &UIButton, is_selected: bool) {
    btn.setSelected(is_selected);
    unsafe {
        let font = if is_selected {
            objc2_ui_kit::UIFont::boldSystemFontOfSize(13.0)
        } else {
            objc2_ui_kit::UIFont::systemFontOfSize(13.0)
        };
        if let Some(label) = btn.titleLabel() {
            label.setFont(Some(&font));
        }
    }
}

pub fn render_content(page_id: i32) {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    SCROLL_VIEW.with(|s| {
        if let Some(scroll_view) = s.borrow().as_ref() {
            // Update tab bar
            if let Some(superview) = scroll_view.superview() {
                if let Some(tab_bar) = superview.viewWithTag(999) {
                    let subviews = tab_bar.subviews();
                    for i in 0..subviews.count() {
                        let subview = subviews.objectAtIndex(i);
                        let tag = subview.tag();
                        if (400..500).contains(&tag) {
                            let btn: Retained<UIButton> = unsafe { Retained::cast_unchecked(subview) };
                            update_tab_button(&btn, tag == (400 + page_id) as isize);
                        }
                    }

                    // Animate indicator
                    if let Some(indicator) = tab_bar.viewWithTag(888) {
                        let tabs = TAB_REGISTRY.lock().unwrap();
                        if let Some(index) = tabs.iter().position(|(_, id)| *id == page_id) {
                            let width = tab_bar.frame().size.width / tabs.len() as f64;
                            let new_x = width * index as f64;
                            let indicator = indicator.clone();
                            animations::animate(0.3, move || {
                                let mut frame = indicator.frame();
                                frame.origin.x = new_x;
                                indicator.setFrame(frame);
                            }, None::<fn(bool)>);
                        }
                    }
                }
            }

            clear_content(scroll_view);
            let frame = scroll_view.frame();
            let mut y_offset = 16.0;
            let (item_height, padding) = (50.0, 16.0);
            let handler = ACTION_HANDLER.with(|h| h.borrow().clone()).unwrap();
            let registry = REGISTRY.lock().unwrap();

            if let Some(items) = registry.pages.get(&page_id) {
                for item in items {
                    let item_frame = CGRect::new(CGPoint::new(padding, y_offset),
                        CGSize::new(frame.size.width - padding * 2.0, item_height));

                    match item {
                        MenuItem::Button { id, name, .. } => {
                            let btn = create_button_item(item_frame, name, mtm);
                            btn.setTag(*id as isize);
                            unsafe { btn.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                            scroll_view.addSubview(&btn);
                            y_offset += item_height + 12.0;
                        }

                        MenuItem::Toggle { id, name, key, default, callback, .. } => {
                            let toggle_height = item_height - 5.0;
                            let toggle = create_toggle_item(
                                CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, toggle_height)),
                                name, mtm);
                            toggle.setTag(*id as isize);
                            let mut current = Preferences::get_bool(key);
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

                            if current { if let Some(cb) = callback { cb(true); } }
                            unsafe { toggle.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                            scroll_view.addSubview(&toggle);
                            y_offset += toggle_height + 12.0;
                        }

                        MenuItem::Slider { id, name, key, min, max, default, .. } => {
                            let mut current = Preferences::get_float(key);
                            if current == 0.0 && *default != 0.0 { current = *default; }
                            let slider_height = item_height - 5.0;
                            let slider_item = create_slider_item(
                                CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, slider_height)),
                                name, current, *min, *max, mtm);
                            if let Some(slider) = slider_item.viewWithTag(4) {
                                slider.setTag(*id as isize);
                                unsafe { let _: () = msg_send![&slider, addTarget: &*handler, action: sel!(handleSlider:), forControlEvents: UIControlEvents::ValueChanged]; }
                            }
                            scroll_view.addSubview(&slider_item);
                            y_offset += slider_height + 12.0;
                        }

                        MenuItem::Input { id, name, key, placeholder, default, .. } => {
                            let input_item = create_text_input_item(
                                CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, 70.0)),
                                name, placeholder, mtm);
                            if let Some(input) = input_item.viewWithTag(6) {
                                input.setTag(*id as isize);
                                let current = Preferences::get_string(key);
                                let val = if current.is_empty() { default } else { &current };
                                if !val.is_empty() { let _: () = unsafe { msg_send![&input, setText: &*NSString::from_str(val)] }; }
                                unsafe { let _: () = msg_send![&input, addTarget: &*handler, action: sel!(handleTextChange:), forControlEvents: UIControlEvents::EditingDidEnd]; }
                            }
                            scroll_view.addSubview(&input_item);
                            y_offset += 82.0;
                        }

                        MenuItem::ActionButton { id, name, .. } => {
                            let action_height = item_height - 5.0;
                            let action_btn = create_action_button_item(
                                CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, action_height)),
                                name, mtm);
                            action_btn.setTag(*id as isize);
                            unsafe { action_btn.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                            scroll_view.addSubview(&action_btn);
                            y_offset += action_height + 12.0;
                        }

                        MenuItem::Label { text, font_size, is_bold, color, .. } => {
                            let label = create_label(
                                CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, 20.0)),
                                text, *font_size as f64, *is_bold, mtm);
                            if let Some(hex) = color {
                                if let Some(ui_color) = hex_to_color(hex) {
                                    unsafe { label.setTextColor(Some(&ui_color)); }
                                }
                            }
                            scroll_view.addSubview(&label);
                            y_offset += 28.0;
                        }

                        MenuItem::SectionHeader { title, .. } => {
                            let header = create_section_header(
                                CGRect::new(CGPoint::new(padding, y_offset + 10.0), CGSize::new(frame.size.width - padding * 2.0, 22.0)),
                                title, mtm);
                            scroll_view.addSubview(&header);
                            y_offset += 48.0;
                        }

                        MenuItem::Dropdown { id, name, key, options, default, .. } => {
                            let dropdown_height = item_height - 5.0;
                            let current_idx = Preferences::get_int(key);
                            let idx = if (current_idx as usize) < options.len() { current_idx } else { *default };
                            let selected_text = if (idx as usize) < options.len() {
                                options[idx as usize].clone()
                            } else {
                                "Select...".to_string()
                            };
                            let dropdown = create_dropdown_item(
                                CGRect::new(CGPoint::new(padding, y_offset), CGSize::new(frame.size.width - padding * 2.0, dropdown_height)),
                                name, &selected_text, mtm);
                            dropdown.setTag(*id as isize);
                            unsafe { dropdown.addTarget_action_forControlEvents(Some(&handler), sel!(handleAction:), UIControlEvents::TouchUpInside); }
                            scroll_view.addSubview(&dropdown);
                            y_offset += dropdown_height + 12.0;
                        }
                    }
                }
                unsafe { let _: () = msg_send![scroll_view, setContentSize: CGSize::new(frame.size.width, y_offset + 20.0)]; }
            }
        }
    });
}

pub fn update_toggle_ui(sender: &UIButton, selected: bool) {
    if let Some(bg) = sender.viewWithTag(2) {
        let bg = bg.clone();
        animations::animate_spring(
            0.4,
            0.6,
            0.8,
            move || {
                bg.setBackgroundColor(Some(&*if selected {
                    Theme::accent()
                } else {
                    Theme::toggle_off()
                }));
                if let Some(knob) = bg.viewWithTag(3) {
                    let mut frame = knob.frame();
                    frame.origin.x = if selected { 22.0 } else { 2.0 };
                    knob.setFrame(frame);
                    knob.setBackgroundColor(Some(&*if selected {
                        Theme::knob_on()
                    } else {
                        Theme::accent()
                    }));
                }
            },
            None::<fn(bool)>,
        );
    }
}

// Helper: create transform
fn scale_transform(scale: f64) -> CGAffineTransform {
    CGAffineTransform {
        a: scale,
        b: 0.0,
        c: 0.0,
        d: scale,
        tx: 0.0,
        ty: 0.0,
    }
}

pub fn show_menu(menu: &UIView) {
    menu.setHidden(false);
    menu.setAlpha(0.0);
    menu.setTransform(scale_transform(0.8));

    let menu_ptr: Retained<UIView> = unsafe {
        let ptr: *mut UIView = msg_send![menu, retain];
        Retained::from_raw(ptr).unwrap()
    };
    animations::animate_spring(
        0.3,
        0.7,
        0.0,
        move || {
            menu_ptr.setAlpha(1.0);
            menu_ptr.setTransform(scale_transform(1.0));
        },
        None::<fn(bool)>,
    );
}

pub fn hide_menu(menu: &UIView) {
    let menu_ptr: Retained<UIView> = unsafe {
        let ptr: *mut UIView = msg_send![menu, retain];
        Retained::from_raw(ptr).unwrap()
    };
    let menu_ptr_anim = menu_ptr.clone();
    animations::animate(
        0.2,
        move || {
            menu_ptr_anim.setAlpha(0.0);
            menu_ptr_anim.setTransform(scale_transform(0.8));
        },
        Some(move |finished: bool| {
            if finished {
                menu_ptr.setHidden(true);
            }
        }),
    );
}

pub fn show_dropdown_selection(
    _title: &str,
    options: Vec<String>,
    current_idx: i32,
    callback: impl Fn(i32) + 'static,
) {
    let mtm = unsafe { MainThreadMarker::new_unchecked() };
    DROPDOWN_CALLBACK.with(|cb| *cb.borrow_mut() = Some(Box::new(callback)));

    let delegate = DROPDOWN_DELEGATE.with(|d| {
        let mut borrow = d.borrow_mut();
        if borrow.is_none() {
            *borrow = Some(DropdownDelegate::new(mtm));
        }
        borrow.as_ref().unwrap().clone()
    });

    let window = {
        #[allow(deprecated)]
        let app = objc2_ui_kit::UIApplication::sharedApplication(mtm);
        #[allow(deprecated)]
        let w = app.keyWindow().unwrap();
        w
    };
    let bounds = window.bounds();

    // Create overlay
    let overlay = UIView::initWithFrame(UIView::alloc(mtm), bounds);
    unsafe {
        overlay.setBackgroundColor(Some(&objc2_ui_kit::UIColor::clearColor()));
        let blur_effect: Retained<UIBlurEffect> =
            msg_send![UIBlurEffect::class(), effectWithStyle: 2isize];
        let visual_effect_view = UIVisualEffectView::new(&blur_effect, mtm);
        visual_effect_view.setFrame(bounds);
        visual_effect_view.setUserInteractionEnabled(false);
        overlay.addSubview(&visual_effect_view);
    }
    overlay.setTag(9999);
    overlay.setUserInteractionEnabled(true);

    let tap: Retained<UITapGestureRecognizer> = unsafe {
        let t = UITapGestureRecognizer::alloc(mtm);
        msg_send![t, initWithTarget: &*delegate, action: sel!(handleDismiss:)]
    };
    overlay.addGestureRecognizer(&tap);

    // Create container
    let (container_width, item_height, max_height) = (280.0, 44.0, 400.0);
    let content_height = (options.len() as f64) * item_height;
    let container_height = f64::min(content_height, max_height);
    let container = UIView::initWithFrame(
        UIView::alloc(mtm),
        CGRect::new(
            CGPoint::new(
                (bounds.size.width - container_width) / 2.0,
                (bounds.size.height - container_height) / 2.0,
            ),
            CGSize::new(container_width, container_height),
        ),
    );

    let container_tap: Retained<UITapGestureRecognizer> = unsafe {
        let t = UITapGestureRecognizer::alloc(mtm);
        msg_send![t, init]
    };
    container.addGestureRecognizer(&container_tap);

    // Content view
    let content_view = UIView::initWithFrame(UIView::alloc(mtm), container.bounds());
    unsafe {
        container.setBackgroundColor(Some(&objc2_ui_kit::UIColor::clearColor()));
        content_view.setBackgroundColor(Some(&Theme::background().colorWithAlphaComponent(0.85)));
        content_view.layer().setCornerRadius(14.0);
        content_view.layer().setBorderWidth(0.5);
        content_view
            .layer()
            .setBorderColor(Some(&Theme::menu_border().CGColor()));
        content_view.setClipsToBounds(true);
    }
    container.addSubview(&content_view);

    // Scroll view with options
    let scroll_view: Retained<UIScrollView> = unsafe {
        let view = UIScrollView::alloc(mtm);
        msg_send![view, initWithFrame: CGRect::new(CGPoint::new(0.0, 0.0), CGSize::new(container_width, container_height))]
    };

    for (i, option) in options.iter().enumerate() {
        let btn = create_btn(
            mtm,
            CGRect::new(
                CGPoint::new(0.0, (i as f64) * item_height),
                CGSize::new(container_width, item_height),
            ),
        );
        let is_selected = i as i32 == current_idx;
        let display_text = if is_selected {
            format!("✓ {}", option)
        } else {
            format!("  {}", option)
        };

        btn.setTitle_forState(
            Some(&NSString::from_str(&display_text)),
            UIControlState::Normal,
        );
        unsafe {
            let (normal_color, highlight_color) = if is_selected {
                (Theme::accent(), Theme::accent())
            } else {
                (Theme::text(), Theme::text_secondary())
            };
            btn.setTitleColor_forState(Some(&normal_color), UIControlState::Normal);
            btn.setTitleColor_forState(Some(&highlight_color), UIControlState::Highlighted);

            if let Some(label) = btn.titleLabel() {
                label.setFont(Some(&objc2_ui_kit::UIFont::systemFontOfSize(15.0)));
                label.setTextAlignment(objc2_ui_kit::NSTextAlignment::Left);
            }
            btn.setContentHorizontalAlignment(
                objc2_ui_kit::UIControlContentHorizontalAlignment::Left,
            );
            #[allow(deprecated)]
            btn.setContentEdgeInsets(objc2_ui_kit::UIEdgeInsets {
                top: 0.0,
                left: 16.0,
                bottom: 0.0,
                right: 16.0,
            });
            btn.addTarget_action_forControlEvents(
                Some(&delegate),
                sel!(handleOption:),
                UIControlEvents::TouchUpInside,
            );
        }
        btn.setTag(i as isize);

        // Separator
        if i > 0 {
            let sep = UIView::initWithFrame(
                UIView::alloc(mtm),
                CGRect::new(
                    CGPoint::new(16.0, (i as f64) * item_height),
                    CGSize::new(container_width - 32.0, 0.5),
                ),
            );
            sep.setBackgroundColor(Some(&Theme::container_border()));
            scroll_view.addSubview(&sep);
        }
        scroll_view.addSubview(&btn);
    }

    unsafe {
        let _: () =
            msg_send![&*scroll_view, setContentSize: CGSize::new(container_width, content_height)];
    }
    content_view.addSubview(&scroll_view);
    overlay.addSubview(&container);
    window.addSubview(&overlay);

    // Animate in
    overlay.setAlpha(0.0);
    container.setTransform(scale_transform(0.8));
    let container_ptr = container.clone();
    animations::animate(
        0.2,
        move || {
            overlay.setAlpha(1.0);
            container_ptr.setTransform(scale_transform(1.0));
        },
        None::<fn(bool)>,
    );
}
