//! Floating button
use objc2::rc::Retained;
use objc2::{msg_send, ClassType, MainThreadOnly};
use objc2_core_foundation::{CGAffineTransform, CGRect, CGSize};
use objc2_foundation::{MainThreadMarker, NSString};
use objc2_ui_kit::{UIButton, UIColor, UIControlState, UIFont};

use crate::ui::theme::Theme;
use crate::ui::utils::animations;
use crate::ui::utils::wrappers::{UIBlurEffect, UIVisualEffectView};

/// Creates the main floating toggle button that opens/closes the menu
///
/// Returns a round button with a blur effect background and a dragon symbol.
///
/// # Arguments
/// * `frame` - The size and position of the button
/// * `mtm` - Main thread marker
pub fn create_toggle_button(frame: CGRect, mtm: MainThreadMarker) -> Retained<UIButton> {
    let button = UIButton::buttonWithType(objc2_ui_kit::UIButtonType::Custom, mtm);
    button.setFrame(frame);

    let blur_effect = unsafe {
        let effect: Retained<UIBlurEffect> =
            msg_send![UIBlurEffect::class(), effectWithStyle: 2i64];
        effect
    };
    let effect_view = UIVisualEffectView::new(&blur_effect, mtm);
    effect_view.setFrame(button.bounds());
    effect_view.setUserInteractionEnabled(false);
    effect_view.layer().setCornerRadius(frame.size.width / 2.0);
    effect_view.setClipsToBounds(true);

    button.addSubview(&effect_view);
    button.sendSubviewToBack(&effect_view);

    button.setBackgroundColor(Some(&UIColor::clearColor()));

    let layer = button.layer();
    layer.setCornerRadius(frame.size.width / 2.0);

    layer.setBorderWidth(0.5);
    unsafe {
        let border_color = Theme::text().colorWithAlphaComponent(0.2).CGColor();
        layer.setBorderColor(Some(&border_color));
    }

    let icon_layer = objc2_quartz_core::CAShapeLayer::new();
    icon_layer.setFrame(button.bounds()); // 50x50
    icon_layer.setFillColor(None);
    icon_layer.setLineWidth(2.5);
    unsafe {
        icon_layer.setLineCap(objc2_quartz_core::kCALineCapRound);
        icon_layer.setLineJoin(objc2_quartz_core::kCALineJoinRound);

        let color = Theme::text();
        icon_layer.setStrokeColor(Some(std::mem::transmute(color.CGColor())));
    }

    let path = crate::ui::assets::icons::menu_path();

    let bounds = path.bounds();
    let current_width = bounds.size.width;
    let current_height = bounds.size.height;

    let button_width = frame.size.width;
    let button_height = frame.size.height;

    let scale = 0.7;

    let center_src_x = bounds.origin.x + current_width / 2.0;
    let center_src_y = bounds.origin.y + current_height / 2.0;

    let tx = (button_width / 2.0) - (center_src_x * scale);
    let ty = (button_height / 2.0) - (center_src_y * scale);

    let transform = CGAffineTransform {
        a: scale,
        b: 0.0,
        c: 0.0,
        d: scale,
        tx,
        ty,
    };

    unsafe {
        path.applyTransform(transform);
        icon_layer.setPath(Some(std::mem::transmute(path.CGPath())));
    }

    layer.addSublayer(&icon_layer);

    unsafe {
        button.setTitle_forState(None, UIControlState::Normal);
        button.setImage_forState(None, UIControlState::Normal);
    }
    button.setUserInteractionEnabled(true);

    // Initial State (Scaled Down)
    button.setTransform(CGAffineTransform {
        a: 0.1,
        b: 0.0,
        c: 0.0,
        d: 0.1,
        tx: 0.0,
        ty: 0.0,
    });
    button.setAlpha(0.0);

    // Animate In
    let button_clone = button.clone();
    animations::animate_spring(
        0.6,
        0.6,
        0.3,
        move || {
            button_clone.setTransform(CGAffineTransform {
                a: 1.0,
                b: 0.0,
                c: 0.0,
                d: 1.0,
                tx: 0.0,
                ty: 0.0,
            });
            button_clone.setAlpha(1.0);
        },
        None::<fn(bool)>,
    );

    button
}
