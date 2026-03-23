//! Animation utilities
use block2::RcBlock;
use objc2::rc::Retained;
use objc2::runtime::AnyObject;
use objc2::runtime::Bool;
use objc2::{msg_send, ClassType};
use objc2_core_foundation::CGAffineTransform;
use objc2_foundation::{NSNumber, NSString};
use objc2_quartz_core::{CABasicAnimation, CALayer, CAMediaTiming, CAMediaTimingFunction};
use objc2_ui_kit::UIView;

fn retain_view(view: &UIView) -> Retained<UIView> {
    unsafe {
        let ptr: *mut UIView = msg_send![view, retain];
        Retained::from_raw(ptr).unwrap()
    }
}

/// Start a basic animation block
pub fn animate(
    duration: f64,
    animations: impl Fn() + 'static,
    completion: Option<impl Fn(bool) + 'static>,
) {
    let anim_block = RcBlock::new(animations);
    let completion_block = completion.map(|c| {
        RcBlock::new(move |finished: Bool| {
            c(finished.is_true());
        })
    });

    unsafe {
        let _: () = msg_send![
            UIView::class(),
            animateWithDuration: duration,
            animations: &*anim_block,
            completion: completion_block.as_deref().map(|c| c as *const _).unwrap_or(std::ptr::null())
        ];
    }
}

/// Start a spring animation
pub fn animate_spring(
    duration: f64,
    damping: f64,
    velocity: f64,
    animations: impl Fn() + 'static,
    completion: Option<impl Fn(bool) + 'static>,
) {
    let anim_block = RcBlock::new(animations);
    let completion_block = completion.map(|c| {
        RcBlock::new(move |finished: Bool| {
            c(finished.is_true());
        })
    });

    unsafe {
        let _: () = msg_send![
            UIView::class(),
            animateWithDuration: duration,
            delay: 0.0f64,
            usingSpringWithDamping: damping,
            initialSpringVelocity: velocity,
            options: 0usize,
            animations: &*anim_block,
            completion: completion_block.as_deref().map(|c| c as *const _).unwrap_or(std::ptr::null())
        ];
    }
}

/// Animate a view's alpha to 1.0
pub fn fade_in(view: &UIView, duration: f64) {
    let view = retain_view(view);
    animate(
        duration,
        move || {
            view.setAlpha(1.0);
        },
        None::<fn(bool)>,
    );
}

/// Animate a view's alpha to 0.0
pub fn fade_out(view: &UIView, duration: f64) {
    let view = retain_view(view);
    animate(
        duration,
        move || {
            view.setAlpha(0.0);
        },
        None::<fn(bool)>,
    );
}

/// Scale a view with a spring animation
pub fn scale_spring(view: &UIView, scale: f64, duration: f64) {
    let view = retain_view(view);
    animate_spring(
        duration,
        0.7,
        0.5,
        move || {
            view.setTransform(CGAffineTransform {
                a: scale,
                b: 0.0,
                c: 0.0,
                d: scale,
                tx: 0.0,
                ty: 0.0,
            });
        },
        None::<fn(bool)>,
    );
}

/// Scale a view with a standard animation
pub fn scale(view: &UIView, scale: f64, duration: f64) {
    let view = retain_view(view);
    animate(
        duration,
        move || {
            view.setTransform(CGAffineTransform {
                a: scale,
                b: 0.0,
                c: 0.0,
                d: scale,
                tx: 0.0,
                ty: 0.0,
            });
        },
        None::<fn(bool)>,
    );
}

/// Animate a layer's strokeEnd property from 0.0 to 1.0
pub fn animate_stroke_end(layer: &CALayer, duration: f64, ease_in_out: bool) {
    let anim = CABasicAnimation::animationWithKeyPath(Some(&NSString::from_str("strokeEnd")));

    let from_val = NSNumber::numberWithFloat(0.0);
    let to_val = NSNumber::numberWithFloat(1.0);

    unsafe {
        let from_obj: &AnyObject = std::mem::transmute(&*from_val);
        anim.setFromValue(Some(from_obj));

        let to_obj: &AnyObject = std::mem::transmute(&*to_val);
        anim.setToValue(Some(to_obj));
    }

    anim.setDuration(duration);

    let timing_name = if ease_in_out {
        unsafe { objc2_quartz_core::kCAMediaTimingFunctionEaseInEaseOut }
    } else {
        unsafe { objc2_quartz_core::kCAMediaTimingFunctionEaseOut }
    };

    anim.setTimingFunction(Some(&CAMediaTimingFunction::functionWithName(timing_name)));

    // Keep state after animation
    anim.setFillMode(unsafe { objc2_quartz_core::kCAFillModeForwards });
    anim.setRemovedOnCompletion(false);

    layer.addAnimation_forKey(&anim, Some(&NSString::from_str("drawStroke")));
}

/// Start a spring animation with delay and options
pub fn animate_spring_with_delay(
    duration: f64,
    delay: f64,
    damping: f64,
    velocity: f64,
    options: usize,
    animations: impl Fn() + 'static,
    completion: Option<impl Fn(bool) + 'static>,
) {
    let anim_block = RcBlock::new(animations);
    let completion_block = completion.map(|c| {
        RcBlock::new(move |finished: Bool| {
            c(finished.is_true());
        })
    });

    unsafe {
        let _: () = msg_send![
            UIView::class(),
            animateWithDuration: duration,
            delay: delay,
            usingSpringWithDamping: damping,
            initialSpringVelocity: velocity,
            options: options,
            animations: &*anim_block,
            completion: completion_block.as_deref().map(|c| c as *const _).unwrap_or(std::ptr::null())
        ];
    }
}

/// Start a basic animation with delay and options
pub fn animate_with_delay(
    duration: f64,
    delay: f64,
    options: usize,
    animations: impl Fn() + 'static,
    completion: Option<impl Fn(bool) + 'static>,
) {
    let anim_block = RcBlock::new(animations);
    let completion_block = completion.map(|c| {
        RcBlock::new(move |finished: Bool| {
            c(finished.is_true());
        })
    });

    unsafe {
        let _: () = msg_send![
            UIView::class(),
            animateWithDuration: duration,
            delay: delay,
            options: options,
            animations: &*anim_block,
            completion: completion_block.as_deref().map(|c| c as *const _).unwrap_or(std::ptr::null())
        ];
    }
}

/// Pulse a view (scale down and back up)
pub fn pulse(view: &UIView, scale: f64, duration: f64) {
    let view = retain_view(view);
    let view_anim = view.clone();

    animate_spring(
        duration / 2.0,
        0.7,
        0.5,
        move || {
            view_anim.setTransform(CGAffineTransform {
                a: scale,
                b: 0.0,
                c: 0.0,
                d: scale,
                tx: 0.0,
                ty: 0.0,
            });
        },
        Some(move |_| {
            let view_restore = view.clone();
            animate_spring(
                duration / 2.0,
                0.7,
                0.5,
                move || {
                    view_restore.setTransform(CGAffineTransform {
                        a: 1.0,
                        b: 0.0,
                        c: 0.0,
                        d: 1.0,
                        tx: 0.0,
                        ty: 0.0,
                    });
                },
                None::<fn(bool)>,
            );
        }),
    );
}
